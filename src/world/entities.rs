#[derive(Copy, Clone)]
pub struct Entity {
    pub x: i16,
    pub y: i16,
    pub w: i16,
    pub h: i16,
    pub typ: i16,
    pub del: bool,
    pub state: u8,
    pub sprite_x: u8,
    pub sprite_y: u8,
    pub speed: u16,
}

impl Entity {
    pub fn new() -> Entity {
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