pub const FONT_START_ADDRESS: usize = 0x0;
pub const PROGRAM_START_ADDRESS: usize = 0x200;


pub mod cpu;
pub mod instruction;
pub mod memory;
pub mod keypad;
pub mod graphics;
pub mod chip8;
pub mod chip8_errors;
