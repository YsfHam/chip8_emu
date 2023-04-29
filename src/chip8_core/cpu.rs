use super::chip8_errors::Chip8ErrorKind;
use super::{PROGRAM_START_ADDRESS, FONT_START_ADDRESS};
use super::instruction::Instruction;
use super::chip8::Devices;
use super::graphics::{SCREEN_HEIGHT, SCREEN_WIDTH};
use super::keypad::Key;
pub struct Stack {
    pub sp: u8,
    pub stack: [u16; 16],
}

impl Stack {
    fn new() -> Stack {
        Stack {
            sp: 0,
            stack: [0; 16],
        }
    }

    fn push(&mut self, value: u16) -> Result<(), Chip8ErrorKind> {
        if self.sp >= self.stack.len() as u8 {
            return Err(Chip8ErrorKind::StackOverflow);
        }
        self.stack[self.sp as usize] = value;
        self.sp += 1;

        Ok(())
    }

    fn pop(&mut self) -> Result<u16, Chip8ErrorKind> {
        if self.sp == 0 {
            return Err(Chip8ErrorKind::EmptyStack)
        }
        
        self.sp -= 1;
        Ok(self.stack[self.sp as usize])
    }

    fn reset(&mut self) {
        self.sp = 0;
        self.stack = [0; 16];
    }
}

pub struct CPU {
    pub v: [u8; 16],
    pub i: u16,
    pub pc: u16,
    pub stack: Stack,
    pub sound_timer: u8,
    pub delay_timer: u8,

    key_pressed: Option<Key>,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            v: [0; 16],
            i: 0,
            pc: PROGRAM_START_ADDRESS as u16,
            stack: Stack::new(),
            sound_timer: 0x1,
            delay_timer: 0x0,
            key_pressed: None,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.v = [0; 16];
        self.i = 0;
        self.pc = PROGRAM_START_ADDRESS as u16;
        self.stack.reset();
        self.sound_timer = 0x1;
        self.delay_timer = 0x0;
    }

    pub(crate) fn clock(&mut self, devices: &mut Devices) -> Result<(), Chip8ErrorKind> {
        let instruction = self.fetch(devices)?;

        self.execute(&instruction, devices)
    }

    pub(crate) fn update_timers(&mut self) {
        if self.sound_timer > 1 { // The original device's audio works for sound timer >= 0x2
            self.sound_timer -= 1;
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }

    fn fetch(&mut self, devices: &Devices) -> Result<Instruction, Chip8ErrorKind> {
        let opcode = devices.ram.read16(self.pc)?;
        self.pc += 2;

        Ok(Instruction::new(opcode))
    }

    fn execute(&mut self, instruction: &Instruction, devices: &mut Devices) -> Result<(), Chip8ErrorKind> {
        let nnn = instruction.nnn();
        let nn = instruction.nn();
        let n = instruction.n();
        let x = instruction.x();
        let y = instruction.y();

        let hi_n = instruction.extract(0xF000, 12);

        match (hi_n, x, y, n) {
            (0x0, _, _, 0x0) => self.instr_00e0(devices),
            (0x0, _, _, 0xE) => self.instr_00ee(),
            (0x0, _, _, _) => self.instr_0nnn(nnn),

            (0x1, _, _, _) => self.instr_1nnn(nnn),
            (0x2, _, _, _) => self.instr_2nnn(nnn),
            (0x3, _, _, _) => self.instr_3xnn(x, nn),
            (0x4, _, _, _) => self.instr_4xnn(x, nn),
            (0x5, _, _, _) => self.instr_5xy0(x, y),
            (0x6, _, _, _) => self.instr_6xnn(x, nn),
            (0x7, _, _, _) => self.instr_7xnn(x, nn),

            (0x8, _, _, 0x0) => self.instr_8xy0(x, y),
            (0x8, _, _, 0x1) => self.instr_8xy1(x, y),
            (0x8, _, _, 0x2) => self.instr_8xy2(x, y),
            (0x8, _, _, 0x3) => self.instr_8xy3(x, y),
            (0x8, _, _, 0x4) => self.instr_8xy4(x, y),
            (0x8, _, _, 0x5) => self.instr_8xy5(x, y),
            (0x8, _, _, 0x6) => self.instr_8xy6(x, y),
            (0x8, _, _, 0x7) => self.instr_8xy7(x, y),
            (0x8, _, _, 0xE) => self.instr_8xye(x, y),

            (0x9, _, _, _) => self.instr_9xy0(x, y),
            (0xA, _, _, _) => self.instr_annn(nnn),
            (0xB, _, _, _) => self.instr_bnnn(nnn),
            (0xC, _, _, _) => self.instr_cxnn(x, nn),
            (0xD, _, _, _) => self.instr_dxyn(x, y, n, devices),

            (0xE, _, _, 0xE) => self.instr_ex9e(x, devices),
            (0xE, _, _, 0x1) => self.instr_exa1(x, devices),

            (0xF, _, _, 0x7) => self.instr_fx07(x),
            (0xF, _, _, 0xA) => self.instr_fx0a(x, devices),
            (0xF, _, 0x1, 0x5) => self.instr_fx15(x),
            (0xF, _, _, 0x8) => self.instr_fx18(x),
            (0xF, _, _, 0xE) => self.instr_fx1e(x),
            (0xF, _, _, 0x9) => self.instr_fx29(x),
            (0xF, _, _, 0x3) => self.instr_fx33(x, devices),
            (0xF, _, 0x5, 0x5) => self.instr_fx55(x, devices),
            (0xF, _, 0x6, 0x5) => self.instr_fx65(x, devices),
            (_, _, _, _) => {
                return Err(Chip8ErrorKind::UnknownInstruction(Instruction::new(instruction.opcode())));
            }
        }
    }

    fn instr_0nnn(&mut self, _: u16) -> Result<(), Chip8ErrorKind> {
        Ok(())
    }

    fn instr_00e0(&mut self, devices: &mut Devices) -> Result<(), Chip8ErrorKind> {
        devices.screen.clear();

        Ok(())
    }

    fn instr_00ee(&mut self) -> Result<(), Chip8ErrorKind> {
        self.pc = self.stack.pop()?;

        Ok(())
    }

    fn instr_1nnn(&mut self, nnn: u16) -> Result<(), Chip8ErrorKind> {
        self.pc = nnn;
        Ok(())
    }

    fn instr_2nnn(&mut self, nnn: u16) -> Result<(), Chip8ErrorKind> {
        self.stack.push(self.pc)?;
        self.pc = nnn;
        Ok(())
    }

    fn instr_3xnn(&mut self, x: u8, nn: u8) -> Result<(), Chip8ErrorKind> {
        let vx = self.v[x as usize];
        if vx == nn {
            self.pc += 2;
        }

        Ok(())
    }

    fn instr_4xnn(&mut self, x: u8, nn: u8) -> Result<(), Chip8ErrorKind> {
        let vx = self.v[x as usize];

        if vx != nn {
            self.pc += 2;
        }

        Ok(())
    }

    fn instr_5xy0(&mut self, x: u8, y: u8) -> Result<(), Chip8ErrorKind> {
        let vx = self.v[x as usize];
        let vy = self.v[y as usize];

        if vx == vy {
            self.pc += 2;
        }

        Ok(())
    }

    fn instr_6xnn(&mut self, x: u8, nn: u8) -> Result<(), Chip8ErrorKind> {
        self.v[x as usize] = nn;
        Ok(())
    }

    fn instr_7xnn(&mut self, x: u8, nn: u8) -> Result<(), Chip8ErrorKind> {
        let vx = self.v[x as usize];
        self.v[x as usize] = vx.wrapping_add(nn);
        Ok(())
    }

    fn instr_8xy0(&mut self, x: u8, y: u8) -> Result<(), Chip8ErrorKind> {
        self.v[x as usize] = self.v[y as usize];
        Ok(())
    }

    fn instr_8xy1(&mut self, x: u8, y: u8) -> Result<(), Chip8ErrorKind> {
        self.v[x as usize] |= self.v[y as usize];
        self.v[0xF] = 0;
        Ok(())
    }

    fn instr_8xy2(&mut self, x: u8, y: u8) -> Result<(), Chip8ErrorKind> {
        self.v[x as usize] &= self.v[y as usize];
        self.v[0xF] = 0;

        Ok(())
    }

    fn instr_8xy3(&mut self, x: u8, y: u8) -> Result<(), Chip8ErrorKind> {
        self.v[x as usize] ^= self.v[y as usize];
        self.v[0xF] = 0;

        Ok(())
    }

    fn instr_8xy4(&mut self, x: u8, y: u8) -> Result<(), Chip8ErrorKind> {
        let (res, overflow) = self.v[x as usize].overflowing_add(self.v[y as usize]);
        self.v[x as usize] = res;
        self.v[0xF] = overflow.into();
        Ok(())
    }

    fn instr_8xy5(&mut self, x: u8, y: u8) -> Result<(), Chip8ErrorKind> {
        let vx = self.v[x as usize];
        let vy = self.v[y as usize];

        let not_borrow = vx > vy;
        self.v[x as usize] = vx.wrapping_sub(vy);
        self.v[0xF] = not_borrow.into();

        Ok(())
    }

    fn instr_8xy6(&mut self, x: u8, y: u8) -> Result<(), Chip8ErrorKind> {
        let bit = self.v[y as usize] & 0x1;
        self.v[x as usize] = self.v[y as usize] >> 1;
        self.v[0xF] = bit;

        Ok(())
    }

    fn instr_8xy7(&mut self, x: u8, y: u8) -> Result<(), Chip8ErrorKind> {
        let vx = self.v[x as usize];
        let vy = self.v[y as usize];

        let not_borrow = vx < vy;
        self.v[x as usize] = vy.wrapping_sub(vx);
        self.v[0xF] = not_borrow.into();

        Ok(())
    }

    fn instr_8xye(&mut self, x: u8, y: u8) -> Result<(), Chip8ErrorKind> {
        let bit = (self.v[y as usize] & 0x80) >> 7;
        self.v[x as usize] = self.v[y as usize] << 1;
        self.v[0xF] = bit;

        Ok(())
    }

    fn instr_9xy0(&mut self, x: u8, y: u8) -> Result<(), Chip8ErrorKind> {
        let vx = self.v[x as usize];
        let vy = self.v[y as usize];

        if vx != vy {
            self.pc += 2;
        }

        Ok(())
    }

    fn instr_annn(&mut self, nnn: u16) -> Result<(), Chip8ErrorKind> {
        self.i = nnn;
        Ok(())
    }

    fn instr_bnnn(&mut self, nnn: u16) -> Result<(), Chip8ErrorKind> {
        self.pc = nnn + self.v[0x0] as u16;

        Ok(())
    }

    fn instr_cxnn(&mut self, x: u8, nn: u8) -> Result<(), Chip8ErrorKind> {
        let rand_byte = rand::random::<u8>();
        self.v[x as usize] = rand_byte & nn;

        Ok(())
    }

    fn instr_dxyn(&mut self, x: u8, y: u8, n: u8, devices: &mut Devices) -> Result<(), Chip8ErrorKind> {
        let startx = self.v[x as usize] % SCREEN_WIDTH as u8;
        let starty = self.v[y as usize] % SCREEN_HEIGHT as u8;

        self.v[0xF] = 0;
        for index in 0..n {
            let ypos = starty + index;
            if ypos >= SCREEN_HEIGHT as u8 {
                break;
            }
            let byte = devices.ram.read8(index as u16 + self.i)?;
            for x in 0..8 {
                let xpos = startx as u16 + x;
                if xpos >= SCREEN_WIDTH as u16 {
                    break;
                }
                let previous_pixel = devices.screen.is_pixel_set(
                    xpos as usize,
                    ypos as usize
                );

                devices.screen.set_pixel(
                    xpos as usize,
                    ypos as usize,
                    (byte & (0x80 >> x)) > 0
                );

                let current_pixel = devices.screen.is_pixel_set(
                    xpos as usize,
                    ypos as usize
                );
                if previous_pixel && !current_pixel {
                    self.v[0xF] = 1;
                }
            }
        }

        Ok(())
    }

    fn instr_ex9e(&mut self, x: u8, devices: &Devices) -> Result<(), Chip8ErrorKind> {
        let key = self.v[x as usize].try_into().unwrap();
        if devices.keypad.key_pressed(key) {
            self.pc += 2;
        }

        Ok(())
    }

    fn instr_exa1(&mut self, x: u8, devices: &Devices) -> Result<(), Chip8ErrorKind> {
        let key = self.v[x as usize].try_into().unwrap();
        if !devices.keypad.key_pressed(key) {
            self.pc += 2;
        }

        Ok(())
    }

    fn instr_fx07(&mut self, x: u8) -> Result<(), Chip8ErrorKind> {
        self.v[x as usize] = self.delay_timer;

        Ok(())
    }

    fn instr_fx0a(&mut self, x: u8, devices: &Devices) -> Result<(), Chip8ErrorKind> {

        if let Some(key) = devices.keypad.get_key_pressed() {
            self.key_pressed = Some(key);
            self.pc -= 2;
        }
        else if let Some(key) = self.key_pressed {
            self.v[x as usize] = key as u8;
            self.key_pressed = None;
        }
        else {
            self.pc -= 2;
        }
        
        Ok(())
    }

    fn instr_fx15(&mut self, x: u8) -> Result<(), Chip8ErrorKind> {
        self.delay_timer = self.v[x as usize];

        Ok(())
    }

    fn instr_fx18(&mut self, x: u8) -> Result<(), Chip8ErrorKind> {
        self.sound_timer = self.v[x as usize];

        Ok(())
    }

    fn instr_fx1e(&mut self, x: u8) -> Result<(), Chip8ErrorKind> {
        self.i = self.i.wrapping_add(self.v[x as usize] as u16);

        Ok(())
    }

    fn instr_fx29(&mut self, x: u8) -> Result<(), Chip8ErrorKind> {
        self.i = (FONT_START_ADDRESS as u16)
        .wrapping_add(self.v[x as usize].wrapping_mul(5) as u16);

        Ok(())
    }

    fn instr_fx33(&mut self, x: u8, devices: &mut Devices) -> Result<(), Chip8ErrorKind> {
        let vx = self.v[x as usize];
        devices.ram.write8(self.i.wrapping_add(2), vx % 10)?;

        let vx = vx / 10;
        devices.ram.write8(self.i.wrapping_add(1), vx % 10)?;

        let vx = vx / 10;
        devices.ram.write8(self.i, vx)
    }

    fn instr_fx55(&mut self, x: u8, devices: &mut Devices) -> Result<(), Chip8ErrorKind> {
        for i in 0..=x {
            devices.ram
            .write8(self.i.wrapping_add(i as u16), self.v[i as usize])?; 
        }
        self.i = self.i.wrapping_add(x as u16 + 1);
        Ok(())
    }

    fn instr_fx65(&mut self, x: u8, devices: &Devices) -> Result<(), Chip8ErrorKind> {
        for i in 0..=x {
            self.v[i as usize] =
            devices.ram.read8(self.i.wrapping_add(i as u16))?;
        }
        self.i = self.i.wrapping_add(x as u16 + 1);

        Ok(())
    }

}