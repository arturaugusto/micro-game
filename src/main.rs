// src/main.rs

// std and main are not available for bare metal software
#![no_std]
#![no_main]

use cortex_m_rt::entry; // The runtime

use nb::block;

use ssd1306::{prelude::*, Builder};

use embedded_hal::digital::v2::InputPin; // the `set_high/low`function

//use embedded_hal::digital::v2::{InputPin, OutputPin};

use pac::interrupt;

// use cortex_m::asm;

#[allow(unused_imports)]
use panic_halt; // When a panic occurs, stop the microcontroller

//use stm32f1xx_hal::pac::{interrupt, Interrupt};
//use _micromath::F32Ext;

mod world;
pub use crate::world::*;

#[interrupt]
fn EXTI9_5() {
    let led = unsafe { &mut *LED.as_mut_ptr() };
    let int_pin = unsafe { &mut *INT_PIN.as_mut_ptr() };
    let rxs = unsafe { &mut *RX.as_mut_ptr() };
    let txs = unsafe { &mut *TX.as_mut_ptr() };

    if int_pin.check_interrupt() {
        match block!(rxs.read()) {
            Ok(_received) => {
                led.toggle().unwrap();
                unsafe {
                    BLAST = true;
                };

                // block!(txs.write(received + 1)).ok();
                block!(txs.write(66)).ok(); // B
                block!(txs.write(76)).ok(); // L
                block!(txs.write(65)).ok(); // A
                block!(txs.write(83)).ok(); // S
                block!(txs.write(84)).ok(); // T
                block!(txs.write(33)).ok(); // !
                block!(txs.write(10)).ok(); // \n
            }
            _ => {}
        }
        // if we don't clear this bit, the ISR would trigger indefinitely
        int_pin.clear_interrupt_pending_bit();
    }
}

// This marks the entrypoint of our application. The cortex_m_rt creates some
// startup code before this, but we don't need to worry about this
#[entry]
fn main() -> ! {
    // Get handles to the hardware objects.
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Get a handle to the RCC peripheral:
    let mut rcc = dp.RCC.constrain();

    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    //let _led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // Get a handle to the FLASH peripheral first:
    let mut flash = dp.FLASH.constrain();

    let clocks = rcc.cfgr.sysclk(8.mhz()).freeze(&mut flash.acr);

    // Setup GPIOB
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    // configure pa0 as an analog input

    // configure pb10 as an digital input
    let mut a_btn = gpiob.pb10.into_dynamic(&mut gpiob.crh);
    a_btn.make_floating_input(&mut gpiob.crh);

    // let mut b_btn = gpiob.pb11.into_dynamic(&mut gpiob.crh);
    // b_btn.make_floating_input(&mut gpiob.crh);

    // configure analog input
    let mut adc1 = adc::Adc::adc1(dp.ADC1, &mut rcc.apb2, clocks);
    let mut adc2 = adc::Adc::adc2(dp.ADC2, &mut rcc.apb2, clocks);

    let mut ch1 = gpioa.pa1.into_analog(&mut gpioa.crl);
    let mut ch2 = gpioa.pa2.into_analog(&mut gpioa.crl);

    let mut delay = Delay::new(cp.SYST, clocks);

    // Prepare the alternate function I/O registers
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    // let channels = dp.DMA1.split(&mut rcc.ahb);

    let pins = gpiob.pb9.into_alternate_push_pull(&mut gpiob.crh);

    let mut pwm = Timer::tim4(dp.TIM4, &clocks, &mut rcc.apb1).pwm::<Tim4NoRemap, _, _, _>(
        pins,
        &mut afio.mapr,
        1.khz(),
    );

    let max = pwm.get_max_duty();
    pwm.set_duty(Channel::C4, max / 10);

    let tx = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
    let rx = gpiob.pb7;

    let serial = Serial::usart1(
        dp.USART1,
        (tx, rx),
        &mut afio.mapr,
        Config::default().baudrate(9600.bps()),
        clocks,
        &mut rcc.apb2,
    );

    let (tx, rx) = serial.split();
    let rxs = unsafe { &mut *RX.as_mut_ptr() };
    let txs = unsafe { &mut *TX.as_mut_ptr() };
    *rxs = rx;
    *txs = tx;

    // // Set up the usart device. Taks ownership over the USART register and tx/rx pins. The rest of
    // // the registers are used to enable and configure the device.

    {
        let led = unsafe { &mut *LED.as_mut_ptr() };
        *led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

        let int_pin = unsafe { &mut *INT_PIN.as_mut_ptr() };
        *int_pin = gpiob.pb8.into_floating_input(&mut gpiob.crh);
        int_pin.make_interrupt_source(&mut afio);
        int_pin.trigger_on_edge(&dp.EXTI, Edge::RISING_FALLING);
        int_pin.enable_interrupt(&dp.EXTI);
    }

    unsafe {
        pac::NVIC::unmask(pac::Interrupt::EXTI9_5);
    }

    // Display

    // SPI1
    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let miso = gpioa.pa6;
    // let miso = gpioa.pa4;
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

    let mut rst = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);
    let dc = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);

    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        &mut afio.mapr,
        Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        },
        8.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    let interface = display_interface_spi::SPIInterfaceNoCS::new(spi, dc);
    let mut disp: GraphicsMode<_> = Builder::new().connect(interface).into();

    disp.reset(&mut rst, &mut delay).unwrap();
    disp.init().unwrap();

    let mut world = World::new(adc1.read(&mut ch1).unwrap());

    let mut input = PlayerInput {
        x_move: 0,
        y_move: 0,
        a_btn_on: false,
        a_btn_changed: false,
    };

    if a_btn.is_high().unwrap() {
        input.a_btn_on = true;
    }

    // prog 0 = game
    // prog 1 = ...
    let prog = 0;

    loop {
        // read analog control

        // yaxis
        let adc1_data: u16 = adc1.read(&mut ch1).unwrap();
        if adc1_data < 400 || adc1_data > 2500 {
            if adc1_data > 2000u16 {
                input.y_move = -1;
            }
            if adc1_data < 2000u16 {
                input.y_move = 1;
            }
        } else {
            input.y_move = 0;
        }

        // xaxis
        let adc2_data: u16 = adc2.read(&mut ch2).unwrap();
        if adc2_data < 400 || adc2_data > 2500 {
            if adc2_data > 2000u16 {
                input.x_move = 1;
            }
            if adc2_data < 2000u16 {
                input.x_move = -1;
            }
        } else {
            input.x_move = 0;
        }

        // handle button
        if a_btn.is_high().unwrap() {
            if input.a_btn_on == false {
                input.a_btn_changed = true
            } else {
                input.a_btn_changed = false
            }
            input.a_btn_on = true;
        } else {
            if input.a_btn_on == true {
                input.a_btn_changed = true
            } else {
                input.a_btn_changed = false
            }
            input.a_btn_on = false;
        }

        match prog {
            1 => {}
            0 => {
                // check world update status
                match world.tick(input) {
                    // you loose
                    1 => {
                        delay.delay_ms(2000u16);
                        world = World::new(adc1.read(&mut ch1).unwrap());
                    }
                    // nothing
                    0 => (),
                    // others
                    _ => (),
                }
            }
            _ => {}
        }

        // update sound state machine
        if world.sound.active {
            pwm.enable(Channel::C4);

            if world.sound.counter > world.sound.counter_end {
                world.sound.counter = 0;
                world.sound.active = false;
                pwm.disable(Channel::C4);
            } else {
                pwm.set_period(world.sound.freqs[world.sound.counter as usize]);
                world.sound.counter += 1;
            }
        }

        // clear display
        for i in 0..64 {
            for j in 0..128 {
                disp.set_pixel(j, i, 0);
            }
        }

        // render
        // let data: u16 = adc1.read(&mut ch0).unwrap();
        // world.write_number(110, 30, data as u32);

        for i in 0..POOL_SIZE {
            let entity = world.entities[i];
            if entity.del != true {
                for y in 0..entity.h {
                    let mut bits: u32 = SPRITES[(y + entity.sprite_y as i16) as usize];

                    // starting x bit
                    bits = bits.rotate_left(entity.sprite_x as u32);

                    for x in 0..entity.w {
                        bits = bits.rotate_left(1);
                        let to_paint = bits & 1u32;
                        if to_paint > 0 {
                            // avoid print to non existing display coord
                            let x_pos = x + entity.x;
                            if x_pos < 0 || x_pos > DISP_W {
                                continue;
                            }
                            let y_pos = y + entity.y;
                            if y_pos < 0 || y_pos > DISP_H {
                                continue;
                            }

                            // print pixel
                            disp.set_pixel(x_pos as u32, y_pos as u32, 1);
                        }
                    }
                }
            }
        }

        disp.flush().unwrap();
        delay.delay_us(1u16);
    }
}
