pub use stm32f1xx_hal::time::Hertz;

pub struct Sound {
    pub counter: u16,
    pub counter_end: u16,
    pub active: bool,
    pub freqs: [Hertz; 6],
}

impl Sound {
    pub fn new() -> Sound {
        Sound {
            counter: 0u16,
            counter_end: 3,
            active: false,
            freqs: [
                Hertz(800),  // destroy enemy
                Hertz(900),  // destroy enemy
                Hertz(1000), // destroy enemy
                Hertz(3700), // shoot
                Hertz(2900), // shoot
                Hertz(2600), //
            ],
        }
    }
}
