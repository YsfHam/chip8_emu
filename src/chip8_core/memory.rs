use super::PROGRAM_START_ADDRESS;
use super::chip8_errors::Chip8ErrorKind;

pub const RAM_SIZE: usize = 1024 * 4;

pub struct Ram {
    memory: [u8; RAM_SIZE],
}

impl Ram {
    pub fn new() -> Self {
        Self {
            memory: [0; RAM_SIZE],
        }
    }

    pub fn reset(&mut self) {
        self.reset_range(PROGRAM_START_ADDRESS, RAM_SIZE);
    }

    pub fn reset_range(&mut self, start: usize, end: usize) {
        for i in start..end {
            self.memory[i] = 0;
        }
    }

    pub fn read8(&self, addr: u16) -> Result<u8, Chip8ErrorKind> {
        if addr >= RAM_SIZE as u16{
            return Err(Chip8ErrorKind::SegmentationFault);
        }
        Ok(self.memory[addr as usize])
    }

    pub fn write8(&mut self, addr: u16, val: u8) -> Result<(), Chip8ErrorKind>{
        if addr >= RAM_SIZE as u16{
            return Err(Chip8ErrorKind::SegmentationFault);
        }
        self.memory[addr as usize] = val;
        Ok(())
    }

    pub fn read16(&self, addr: u16) -> Result<u16, Chip8ErrorKind> {

        let hi = self.read8(addr)? as u16;
        let lo = self.read8(addr + 1)? as u16;

        Ok((hi << 8) | lo)
    }

    pub fn write_bytes(&mut self, start_addr: u16, bytes: &[u8]) -> Result<(), Chip8ErrorKind>{
        for (i, byte) in bytes.iter().enumerate() {
            self.write8(start_addr + i as u16, *byte)?;
        }

        Ok(())
    }

    pub fn read_bytes(&self, start_addr: u16, end_addr: u16) -> Result<&[u8], Chip8ErrorKind> {
        let ram_size = RAM_SIZE as u16;
        if start_addr >= ram_size || end_addr >= ram_size || start_addr >= end_addr {
            return Err(Chip8ErrorKind::SegmentationFault)
        }
        Ok(&self.memory[start_addr as usize..end_addr as usize])
    }
}