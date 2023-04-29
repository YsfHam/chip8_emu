use super::chip8_errors::Chip8ErrorKind;
use super::cpu::CPU;
use super::memory::Ram;
use super::{FONT_START_ADDRESS, PROGRAM_START_ADDRESS};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use crate::timer::Timer;
use super::keypad::KeyPad;
use super::graphics::{Screen, FrameBuffer};
use super::memory::RAM_SIZE;

const CLOCK_FREQ: f64 = 1.0 / 60.0;

const FONT_DATA:[u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,//0
    0x20, 0x60, 0x20, 0x20, 0x70,//1
    0xF0, 0x10, 0xF0, 0x80, 0xF0,//2
    0xF0, 0x10, 0xF0, 0x10, 0xF0,//3
    0x90, 0x90, 0xF0, 0x10, 0x10,//4
    0xF0, 0x80, 0xF0, 0x10, 0xF0,//5
    0xF0, 0x80, 0xF0, 0x90, 0xF0,//6
    0xF0, 0x10, 0x20, 0x40, 0x40,//7
    0xF0, 0x90, 0xF0, 0x90, 0xF0,//8
    0xF0, 0x90, 0xF0, 0x10, 0xF0,//9
    0xF0, 0x90, 0xF0, 0x90, 0x90,//A
    0xE0, 0x90, 0xE0, 0x90, 0xE0,//B
    0xF0, 0x80, 0x80, 0x80, 0xF0,//C
    0xE0, 0x90, 0x90, 0x90, 0xE0,//D
    0xF0, 0x80, 0xF0, 0x80, 0xF0,//E
    0xF0, 0x80, 0xF0, 0x80, 0x80,//F
];

pub(crate) struct Devices {
    pub ram: Ram,
    pub keypad: KeyPad,
    pub screen: Screen,
}

impl Devices {
    fn new() -> Self {

        let mut ram = Ram::new();
        
        ram.write_bytes(FONT_START_ADDRESS as u16, &FONT_DATA).unwrap();

        Self {
            ram,
            keypad: KeyPad::new(),
            screen: Screen::new(),
        }
    }
}

pub struct Chip8 {
    cpu: CPU,
    devices: Devices,

    read_program_length: usize,

    clock_update_timer: Timer,
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            cpu: CPU::new(),
            devices: Devices::new(),
            read_program_length: 0,
            clock_update_timer: Timer::new(),
        }
    }

    pub fn get_cpu(&self) -> &CPU {
        &self.cpu
    }

    pub fn get_ram(&self) -> &Ram {
        &self.devices.ram
    }

    pub fn get_program_length(&self) -> usize {
        self.read_program_length
    }

    pub fn get_screen(&self) -> &Screen {
        &self.devices.screen
    }

    pub fn get_keypad_mut(&mut self) -> &mut KeyPad {
        &mut self.devices.keypad
    }

    pub fn get_keypad(&self) -> &KeyPad {
        &self.devices.keypad
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.devices.ram.reset();
        self.devices.keypad.reset();
        self.devices.screen.clear();
    }

    pub fn reload_program(&mut self) {
        self.cpu.reset();
        self.devices.ram.reset_range(PROGRAM_START_ADDRESS + self.read_program_length, RAM_SIZE);
        self.devices.keypad.reset();
        self.devices.screen.clear();
    }

    pub fn load_program(&mut self, program_file: &PathBuf) -> std::io::Result<()> {
        let mut file = File::open(program_file)?;
        let mut buf = Vec::new();
        self.read_program_length = file.read_to_end(&mut buf)?;

        self.devices.ram.write_bytes(PROGRAM_START_ADDRESS as u16, buf.as_slice()).unwrap();

        Ok(())
    }

    pub fn can_play_sound(&self) -> bool {
        self.cpu.sound_timer > 1
    }

    pub fn update_framebuffer(&mut self, framebuffer: &mut dyn FrameBuffer) {
        if self.devices.screen.updated {
            framebuffer.update(&self.devices.screen);
            self.devices.screen.updated = false;
        }
    }

    pub fn run_instruction(&mut self) -> Result<(), Chip8ErrorKind> {
        let mut elapsed = self.clock_update_timer.elapsed().as_secs_f64();
        let mut restart = false;
        while elapsed >= CLOCK_FREQ {
            self.cpu.update_timers();
            elapsed -= CLOCK_FREQ;
            restart = true;
        }
        if restart {
            self.clock_update_timer.restart();
        }
        self.cpu.clock(&mut self.devices)
    }
}