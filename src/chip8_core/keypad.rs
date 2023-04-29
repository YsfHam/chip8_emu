#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Key {
    Num0 = 0x0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    A,
    B,
    C,
    D,
    E,
    F
}
#[derive(Debug)]
pub struct UnknownKeyError(u8);

impl TryFrom<u8> for Key {
    type Error = UnknownKeyError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(Key::Num0),
            0x1 => Ok(Key::Num1),
            0x2 => Ok(Key::Num2),
            0x3 => Ok(Key::Num3),
            0x4 => Ok(Key::Num4),
            0x5 => Ok(Key::Num5),
            0x6 => Ok(Key::Num6),
            0x7 => Ok(Key::Num7),
            0x8 => Ok(Key::Num8),
            0x9 => Ok(Key::Num9),
            0xA => Ok(Key::A),
            0xB => Ok(Key::B),
            0xC => Ok(Key::C),
            0xD => Ok(Key::D),
            0xE => Ok(Key::E),
            0xF => Ok(Key::F),
            _ => Err(UnknownKeyError(value))
        }
    }
}

pub const CHIP8_KEYPAD: [Key; 16] = [
    Key::Num1, Key::Num2, Key::Num3, Key::C,
    Key::Num4, Key::Num5, Key::Num6, Key::D,
    Key::Num7, Key::Num8, Key::Num9, Key::E,
    Key::A, Key::Num0, Key::B, Key::F,
];

pub struct KeyPad {
    keys_status: [bool; 16],
}

impl KeyPad {
    pub fn new() -> Self {
        Self {
            keys_status: [false; 16],
        }
    }

    pub(crate) fn reset(&mut self) {
        self.keys_status = [false; 16];
    }

    pub fn key_pressed(&self, key: Key) -> bool {
        self.keys_status[key as usize]
    }

    pub fn set_key_pressed(&mut self, key: Key, pressed: bool) {
        self.keys_status[key as usize] = pressed;
    }

    pub fn get_key_pressed(&self) -> Option<Key> {
        for (i, pressed) in self.keys_status.iter().enumerate() {
            if *pressed {
                return Some(Key::try_from(i as u8).unwrap());
            }
        }
        None
    }
}