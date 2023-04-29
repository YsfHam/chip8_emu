pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const SCREEN_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

pub struct Screen {
    screen: [bool; SCREEN_SIZE],
    pub updated: bool,
}

impl Screen {
    pub fn new() -> Self {
        Self {
            screen: [false; SCREEN_SIZE],
            updated: false,
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: bool) {
        let index = y * SCREEN_WIDTH + x;
        let old_val = self.screen[index];
        self.screen[index] = (pixel && !old_val) || (!pixel && old_val);

        self.updated = true;
    }

    pub fn is_pixel_set(&self, x: usize, y: usize) -> bool {
        self.screen[y * SCREEN_WIDTH + x]
    }

    pub fn clear(&mut self) {
        self.screen = [false; SCREEN_SIZE];
        self.updated = true;
    }
}

pub trait FrameBuffer {
    fn update(&mut self, screen: &Screen);
}