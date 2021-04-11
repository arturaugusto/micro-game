// src/main.rs

// std and main are not available for bare metal software
#![no_std]
#![no_main]

use core::mem::MaybeUninit;
use cortex_m_rt::entry; // The runtime
                        //use cortex_m_semihosting::hprintln;

// use cortex_m::singleton;

use nb::block;

use ssd1306::{prelude::*, Builder};

use embedded_hal::digital::v2::InputPin; // the `set_high/low`function
                                         //use embedded_hal::digital::v2::{InputPin, OutputPin};
use pac::interrupt;

use stm32f1xx_hal::{
    adc,
    delay::Delay,
    // dma::Half,
    gpio::*,
    pac,
    prelude::*,
    serial::{Config, Serial},
    spi::{Mode, Phase, Polarity, Spi},
}; // STM32F1 specific functions

#[allow(unused_imports)]
use panic_halt; // When a panic occurs, stop the microcontroller

//use stm32f1xx_hal::pac::{interrupt, Interrupt};
//use _micromath::F32Ext;

static mut LED: MaybeUninit<stm32f1xx_hal::gpio::gpioc::PC13<Output<PushPull>>> =
    MaybeUninit::uninit();
static mut INT_PIN: MaybeUninit<stm32f1xx_hal::gpio::gpiob::PB8<Input<Floating>>> =
    MaybeUninit::uninit();

static mut RX: MaybeUninit<stm32f1xx_hal::serial::Rx<stm32f1xx_hal::pac::USART1>> =
    MaybeUninit::uninit();

static mut TX: MaybeUninit<stm32f1xx_hal::serial::Tx<stm32f1xx_hal::pac::USART1>> =
    MaybeUninit::uninit();

#[interrupt]
fn EXTI9_5() {
    let led = unsafe { &mut *LED.as_mut_ptr() };
    let int_pin = unsafe { &mut *INT_PIN.as_mut_ptr() };
    let rxs = unsafe { &mut *RX.as_mut_ptr() };
    let txs = unsafe { &mut *TX.as_mut_ptr() };

    if int_pin.check_interrupt() {
        match block!(rxs.read()) {
            Ok(received) => {
                led.toggle().unwrap();
                block!(txs.write(received + 1)).ok();
            }
            _ => {}
        }
        // if we don't clear this bit, the ISR would trigger indefinitely
        int_pin.clear_interrupt_pending_bit();
    }
}

const POOL_SIZE: usize = 100;

const DISP_H: i16 = 64i16;
const DISP_W: i16 = 128i16;

const SPRITES: [u32; 63] = [
    // numbers
    0b_01111000_00011000_01111000_01111100, //  0
    0b_11000100_01111000_11001100_11000110, //  1
    0b_11100100_00011000_11001100_00000110, //  2
    0b_11010100_00011000_00011000_00111100, //  3
    0b_11001100_00011000_00110000_00000110, //  4
    0b_11000100_00011000_01100000_11000110, //  5
    0b_01111000_01111100_11111100_01111100, //  6
    0b_00000000_00000000_00000000_00000000, //  7
    0b_11001100_11111110_01111100_11111110, //  8
    0b_11001100_11000000_11000110_11000110, //  9
    0b_11001100_11000000_11000000_00000110, // 10
    0b_11001100_11111110_11111100_00011000, // 11
    0b_11111110_00000110_11000110_01100000, // 12
    0b_00001100_11000110_11000110_01100000, // 13
    0b_00001100_11111110_01111100_01100000, // 14
    0b_00000000_00000000_00000000_00000000, // 15
    0b_01111100_01111100_00000000_00000000, // 16
    0b_11000110_11000110_00000000_00000000, // 17
    0b_11000110_11000110_00000000_00000000, // 18
    0b_01111100_01111110_00000000_00000000, // 19
    0b_11000110_00000110_00000000_00000000, // 20
    0b_11000110_11000110_00000000_00000000, // 21
    0b_01111100_01111100_00000000_00000000, // 22
    0b_00000000_00000000_00000000_00000000, // 23
    // enemy
    0b00010001000000000001001000100100, // 24
    0b00001010000000000001000101000100, // 25
    0b00111111100000000001011111110100, // 26
    0b01101110110000000000110111011000, // 27
    0b11111111111000000000111111111000, // 28
    0b10111111101000000000011111110000, // 29
    0b10100000101000000000010000010000, // 30
    0b00011011000000000000001101100000, // 31
    // stars noise
    0b11100110101011111111111110111111, // 32
    // ship
    0b11110000000000000000000000000000, // 33
    0b11001100000000000000000000000000, // 34
    0b11111111000000000000000000000000, // 35
    0b01111111100000000000000000000000, // 36
    0b00111110011100000000000000000000, // 37
    0b00111110011100000000000000000000, // 38
    0b01111111100000000000000000000000, // 39
    0b11111111000000000000000000000000, // 40
    0b11001100000000000000000000000000, // 41
    0b11110000000000000000000000000000, // 42
    0b11110000000000000000000000000000, // 43
    0b11001100000000000000000000000000, // 44
    0b11111111000000000000000000000000, // 45
    0b01111111100000000000000000000000, // 46
    0b01111110011100000000000000000000, // 47
    0b00111110011100000000000000000000, // 48
    0b00111110100000000000000000000000, // 49
    0b11111111000000000000000000000000, // 50
    0b11111100000000000000000000000000, // 51
    0b11100000000000000000000000000000, // 52
    0b11100000000000000000000000000000, // 53
    0b11111100000000000000000000000000, // 54
    0b11111111000000000000000000000000, // 55
    0b00111110100000000000000000000000, // 56
    0b00111110011100000000000000000000, // 57
    0b01111110011100000000000000000000, // 58
    0b01111111100000000000000000000000, // 59
    0b11111111000000000000000000000000, // 60
    0b11001100000000000000000000000000, // 61
    0b11110000000000000000000000000000, // 62
];

#[derive(Copy, Clone)]
struct Entity {
    x: i16,
    y: i16,
    w: i16,
    h: i16,
    typ: i16,
    del: bool,
    state: u8,
    sprite_x: u8,
    sprite_y: u8,
    speed: u16,
}

impl Entity {
    fn new() -> Entity {
        Entity {
            x: 0i16,
            y: 0i16,
            w: 0i16,
            h: 0i16,
            typ: 0i16,
            del: true,
            state: 0u8,
            sprite_x: 0u8,
            sprite_y: 0u8,
            speed: 0u16,
        }
    }
}

struct Xorshift128pState {
    a: u64,
    b: u64,
}

// random number generator
// based on https://en.wikipedia.org/wiki/Xorshift
impl Xorshift128pState {
    fn new(seed: u64) -> Xorshift128pState {
        let b = seed * 34;
        let mut res = Xorshift128pState { a: seed, b: b };
        // drop some samples
        for _i in 0..6 {
            res.gen();
        }
        res
    }

    fn gen(&mut self) -> u64 {
        let mut t: u64 = self.a;
        let s: u64 = self.b;
        self.a = s;
        t ^= t << 23; // a
        t ^= t >> 17; // b
        t ^= s ^ (s >> 26); // c
        self.b = t;
        return t.wrapping_add(s);
    }

    fn gen_min_max(&mut self, min: u64, max: u64) -> u64 {
        let n = self.gen();
        n / (u64::MAX / (max - min)) + min
    }
}

#[derive(Copy, Clone)]
struct PlayerInput {
    x_move: i16,
    y_move: i16,
    a_btn_on: bool,
    a_btn_changed: bool,
}

struct World {
    entities: [Entity; POOL_SIZE],
    random: Xorshift128pState,
    score: u32,
}

impl World {
    fn has_collision(&self, a: Entity, b: Entity) -> bool {
        return a.x + a.w >= b.x && a.x <= b.x + b.w && a.y + a.h >= b.y && a.y <= b.y + b.h;
    }

    fn write_number(&mut self, x: i16, y: i16, mut n: u32) {
        let number_pool_start = 60;
        for i in 0..5 {
            let m: u32 = n % 10u32;
            n = n / 10u32;

            let mut entity = self.entities[i + number_pool_start];
            entity.x = x - i as i16 * 8;
            entity.y = y;
            entity.del = false;
            // get bitflag position of character
            entity.sprite_x = m as u8 * 8;
            entity.sprite_y = (m as u8 / 4) * 8;
            self.entities[i + number_pool_start] = entity;

            if n == 0 {
                break;
            }
        }
    }

    fn new() -> World {
        let mut world = World {
            entities: [Entity::new(); POOL_SIZE],
            random: Xorshift128pState::new(52),
            score: 0u32,
        };

        for i in 60..70 {
            let mut entity = world.entities[i];
            entity.typ = 4;
            entity.w = 8i16;
            entity.h = 8i16;
            entity.y = 15i16;
            entity.del = true;
            entity.sprite_x = 0 * 8;
            entity.sprite_y = 0;
            world.entities[i] = entity;
        }

        // type codes:
        // 0: player
        // 1: enemy
        // 2: bullet
        // 3: stars
        // Position at array define paint order

        // make bullet pool
        for i in 1..5 {
            let mut entity = world.entities[i];
            entity.typ = 2;
            entity.w = 2i16;
            entity.h = 2i16;
            entity.x = 70i16;
            entity.y = 40i16;
            entity.del = true;
            entity.sprite_y = 53;
            world.entities[i] = entity;
        }

        for i in 10..40 {
            // enemy
            let mut entity = world.entities[i];
            entity.typ = 1;
            entity.w = 11i16;
            entity.h = 8i16;
            entity.x = 40i16;
            entity.y = 40i16;
            entity.del = true;
            entity.sprite_y = 24;
            world.entities[i] = entity;
        }

        // stars
        for i in 40..50 {
            let mut entity = world.entities[i];
            entity.typ = 3;
            entity.w = 2i16;
            entity.h = 1i16;
            entity.x = 0i16;
            entity.y = 0i16;
            entity.del = true;
            entity.sprite_y = 32;
            if world.random.gen_min_max(1, 4) == 1 {
                entity.speed = 2;
            } else {
                entity.speed = 1;
            }

            world.entities[i] = entity;
        }

        //player
        let mut entity = world.entities[51];
        entity.typ = 0;
        entity.w = 12i16;
        entity.h = 10i16;
        entity.x = 20i16;
        entity.y = 0i16;
        entity.sprite_y = 33u8;
        entity.del = false;

        entity.y = DISP_H / 2 - entity.h / 2;

        world.entities[51] = entity;

        return world;
    }

    fn tick(&mut self, input: PlayerInput) -> u16 {
        // spawn new enemies
        if self.random.gen() < u64::MAX / 5 + (self.score as u64) / 2 {
            for i in 0..POOL_SIZE {
                if self.entities[i].del == true && self.entities[i].typ == 1 {
                    let mut enemy = self.entities[i];
                    enemy.del = false;
                    enemy.x = DISP_W - enemy.w;
                    enemy.y = self
                        .random
                        .gen_min_max(0u64, DISP_H as u64 - enemy.h as u64)
                        as i16;
                    self.entities[i] = enemy;
                    break;
                }
            }
        }

        // stars
        if self.random.gen() < u64::MAX / 3 {
            for i in 0..POOL_SIZE {
                if self.entities[i].del == true && self.entities[i].typ == 3 {
                    let mut star = self.entities[i];
                    star.del = false;
                    star.x = DISP_W - star.w;
                    star.y = self.random.gen_min_max(0u64, DISP_H as u64 - star.h as u64) as i16;
                    self.entities[i] = star;
                    break;
                }
            }
        }

        // update
        for i in 0..POOL_SIZE {
            if self.entities[i].del != true {
                let mut entity = self.entities[i];

                if entity.x + entity.w < -2 {
                    entity.del = true;
                    self.entities[i] = entity;
                    continue;
                }

                // bullets updates
                if entity.typ == 2 {
                    if entity.x + entity.w * 2 > DISP_W {
                        entity.del = true;
                    } else {
                        entity.x = entity.x + 4;
                    }

                    // bullet-enemy collision
                    for j in 0..POOL_SIZE {
                        if self.entities[j].del == false && self.entities[j].typ == 1 {
                            let mut enemy = self.entities[j];
                            if self.has_collision(entity, enemy) {
                                enemy.del = true;
                                entity.del = true;
                                // update score
                                self.score += 1;
                                self.write_number(110, 1, self.score);
                            }
                            self.entities[j] = enemy;
                        }
                    }
                    self.entities[i] = entity;
                }

                // player updates
                if entity.typ == 0 {
                    for j in 0..POOL_SIZE {
                        if self.entities[j].del == false && self.entities[j].typ == 1 {
                            let enemy = self.entities[j];
                            if self.has_collision(entity, enemy) {
                                self.score = 0;
                                return 1u16;
                            }
                        }
                    }

                    entity.x = entity.x + input.x_move * 2;
                    entity.y = entity.y + input.y_move * 2;

                    if input.y_move > 0 {
                        entity.sprite_y = 43;
                    }
                    if input.y_move < 0 {
                        entity.sprite_y = 53;
                    }
                    if input.y_move == 0 {
                        entity.sprite_y = 33;
                    }

                    // dont allow player move outside canvas
                    if entity.x < 0 {
                        entity.x = 0;
                    }
                    if entity.x + entity.w > DISP_W {
                        entity.x = DISP_W - entity.w;
                    }
                    if entity.y < 0 {
                        entity.y = 0;
                    }
                    if entity.y + entity.h > DISP_H {
                        entity.y = DISP_H - entity.h;
                    }

                    // on key press, shoot a bullet
                    if input.a_btn_on && input.a_btn_changed {
                        // find a deleted bullet on pool
                        for j in 0..POOL_SIZE {
                            if self.entities[j].del == true && self.entities[j].typ == 2 {
                                let mut bullet = self.entities[j];
                                bullet.del = false;
                                bullet.x = entity.x + entity.w + 1;
                                bullet.y = entity.y + entity.h / 2 - bullet.h / 2;
                                self.entities[j] = bullet;
                                break;
                            }
                        }
                    }

                    self.entities[i] = entity;
                }

                // enemy updates
                if entity.typ == 1 {
                    entity.x -= 2;
                    entity.state = entity.state.wrapping_add(1);

                    if entity.state >= 20 {
                        entity.state = 0;
                    }
                    if entity.state < 20 {
                        entity.sprite_x = 19;
                    }
                    if entity.state < 10 {
                        entity.sprite_x = 0;
                    }

                    self.entities[i] = entity;
                }

                // stars update
                if entity.typ == 3 {
                    entity.state = entity.state.wrapping_add(1);
                    entity.sprite_x = self.random.gen_min_max(0, 30) as u8;
                    entity.x -= 3 + entity.speed as i16;
                    self.entities[i] = entity;
                }
            }
        }
        return 0u16;
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

    // configure pa0 as an digital input
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

    // USART1

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

    let mut world = World::new();

    let mut input = PlayerInput {
        x_move: 0,
        y_move: 0,
        a_btn_on: false,
        a_btn_changed: false,
    };

    if a_btn.is_high().unwrap() {
        input.a_btn_on = true;
    }

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

        // check world update status
        match world.tick(input) {
            // you loose
            1 => {
                delay.delay_ms(2000u16);
                world = World::new();
            }
            // nothing
            0 => (),
            // others
            _ => (),
        }

        // clear display
        for i in 0..64 {
            for j in 0..128 {
                disp.set_pixel(j, i, 0);
            }
        }

        // render
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
