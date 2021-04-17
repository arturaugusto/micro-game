use nb::block;

pub use stm32f1xx_hal::{
    adc,
    delay::Delay,
    // dma::Half,
    gpio::*,
    pac,
    prelude::*,
    pwm::Channel,
    serial::{Config, Serial},
    spi::{Mode, Phase, Polarity, Spi},
    time::Hertz,
    timer::{Tim4NoRemap, Timer},
}; // STM32F1 specific functions

pub use core::mem::MaybeUninit;

mod sprites;
pub use self::sprites::*;

mod seed_rand;
pub use self::seed_rand::*;

mod sound;
pub use self::sound::*;

mod entities;
pub use self::entities::*;

pub static mut LED: MaybeUninit<stm32f1xx_hal::gpio::gpioc::PC13<Output<PushPull>>> =
    MaybeUninit::uninit();
pub static mut INT_PIN: MaybeUninit<stm32f1xx_hal::gpio::gpiob::PB8<Input<Floating>>> =
    MaybeUninit::uninit();

pub static mut RX: MaybeUninit<stm32f1xx_hal::serial::Rx<stm32f1xx_hal::pac::USART1>> =
    MaybeUninit::uninit();

pub static mut TX: MaybeUninit<stm32f1xx_hal::serial::Tx<stm32f1xx_hal::pac::USART1>> =
    MaybeUninit::uninit();

pub static mut BLAST: bool = false;

pub const POOL_SIZE: usize = 100;

pub const DISP_H: i16 = 64i16;
pub const DISP_W: i16 = 128i16;

#[derive(Copy, Clone)]
pub struct PlayerInput {
    pub x_move: i16,
    pub y_move: i16,
    pub a_btn_on: bool,
    pub a_btn_changed: bool,
}

pub struct World {
    pub entities: [Entity; POOL_SIZE],
    pub random: Xorshift128pState,
    pub score: u32,
    pub sound: Sound,
}

impl World {
    pub fn has_collision(&self, a: Entity, b: Entity) -> bool {
        return a.x + a.w >= b.x && a.x <= b.x + b.w && a.y + a.h >= b.y && a.y <= b.y + b.h;
    }

    pub fn write_number(&mut self, x: i16, y: i16, mut n: u32) {
        let number_pool_start = 60;

        // clear digits
        for i in number_pool_start..70 {
            let mut entity = self.entities[i];
            entity.typ = 4;
            entity.w = 8i16;
            entity.h = 8i16;
            entity.y = 15i16;
            entity.del = true;
            entity.sprite_x = 0 * 8;
            entity.sprite_y = 0;
            self.entities[i] = entity;
        }

        for i in 0..9 {
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

    pub fn new(seed: u16) -> World {
        let mut world = World {
            entities: [Entity::new(); POOL_SIZE],
            random: Xorshift128pState::new(seed as u64),
            score: 0u32,
            sound: Sound::new(),
        };

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

    pub fn tick(&mut self, input: PlayerInput) -> u16 {
        let txs = unsafe { &mut *TX.as_mut_ptr() };

        // spawn new enemies
        if self.random.gen_min_max(0, 1000) < 100 + (self.score as u64) {
            // if self.random.gen() < u64::MAX / 500000 + (self.score as u64) / 2 {
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

        unsafe {
            if BLAST {
                for i in 0..POOL_SIZE {
                    if self.entities[i].del == false && self.entities[i].typ == 1 {
                        let mut enemy = self.entities[i];
                        enemy.del = true;
                        self.entities[i] = enemy;
                    }
                }
                BLAST = false;
            }
        };

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

                    // remove score if enemy pass
                    if entity.typ == 1 && self.score > 0 {
                        self.score -= 1;
                        self.write_number(110, 1, self.score);
                        entity.del = true;
                    }

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

                                self.sound.counter = 0;
                                self.sound.counter_end = 2;
                                self.sound.active = true;

                                self.write_number(110, 1, self.score);
                                block!(txs.write(self.score as u8 + 48u8)).ok();
                                block!(txs.write(10)).ok();
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

                                self.sound.freqs[3 as usize] =
                                    Hertz(self.random.gen_min_max(3400, 3700) as u32);

                                self.sound.freqs[4 as usize] =
                                    Hertz(self.random.gen_min_max(2800, 3000) as u32);

                                self.sound.counter = 3;
                                self.sound.counter_end = 4;
                                self.sound.active = true;

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

                    // sprite animation
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
