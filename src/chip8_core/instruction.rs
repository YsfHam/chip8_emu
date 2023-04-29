#[derive(Debug)]
pub struct Instruction {
    opcode: u16,
}

impl Instruction {
    pub fn new(opcode: u16) -> Instruction {
        Self { opcode }
    }

    pub fn x(&self) -> u8 {
        self.extract(0x0F00, 8) as u8
    }

    pub fn y(&self) -> u8 {
        self.extract(0x00F0, 4) as u8
    }

    pub fn nnn(&self) -> u16 {
        self.extract(0x0FFF, 0)
    }

    pub fn nn(&self) -> u8 {
        self.extract(0x00FF, 0) as u8
    }

    pub fn n(&self) -> u8 {
        self.extract(0x000F, 0) as u8
    }

    pub fn extract(&self, mask: u16, shift: u16) -> u16 {
        (self.opcode & mask) >> shift
    }

    pub fn opcode(&self) -> u16 {
        self.opcode
    }

    pub fn to_string(&self) -> String {
        let nnn = self.nnn();
        let nn = self.nn();
        let n = self.n();
        let x = self.x();
        let y = self.y();

        let hi_n = self.extract(0xF000, 12);

        match (hi_n, x, y, n) {
            (0x0, _, _, 0x0) => format!("CLS"),
            (0x0, _, _, 0xE) => format!("RET"),
            (0x0, _, _, _) => format!("SYS {:#04X}", nnn),

            (0x1, _, _, _) => format!("JP {:#04X}", nnn),
            (0x2, _, _, _) => format!("CALL {:#04X}", nnn),
            (0x3, _, _, _) => format!("SE V{:X}, {:#X}", x, nn),
            (0x4, _, _, _) => format!("SNE V{:X}, {:#X}", x, nn),
            (0x5, _, _, _) => format!("SE V{:X}, V{:X}", x, y),
            (0x6, _, _, _) => format!("LD V{:X}, {:#X}", x, nn),
            (0x7, _, _, _) => format!("ADD V{:X}, {:#X}", x, nn),

            (0x8, _, _, 0x0) => format!("LD V{:X}, V{:X}", x, y),
            (0x8, _, _, 0x1) => format!("OR V{:X}, V{:X}", x, y),
            (0x8, _, _, 0x2) => format!("AND V{:X}, V{:X}", x, y),
            (0x8, _, _, 0x3) => format!("XOR V{:X}, {:#X}", x, y),
            (0x8, _, _, 0x4) => format!("ADD V{:X}, {:#X}", x, y),
            (0x8, _, _, 0x5) => format!("SUB V{:X}, {:#X}", x, y),
            (0x8, _, _, 0x6) => format!("SHR V{:X}", x),
            (0x8, _, _, 0x7) => format!("SUBN V{:X}, {:#X}", x, y),
            (0x8, _, _, 0xE) => format!("SHL V{:X}", x),

            (0x9, _, _, _) => format!("SNE V{:X}, V{:X}", x, y),
            (0xA, _, _, _) => format!("LD I {:#X}", nnn),
            (0xB, _, _, _) => format!("JP V0 {:#X}", nnn),
            (0xC, _, _, _) => format!("RND V{:X}, {:#X}", x, nn),
            (0xD, _, _, _) => format!("DRW V{:X}, V{:X}, {:#X}", x, y, n),

            (0xE, _, _, 0xE) => format!("SKP V{:X}", x),
            (0xE, _, _, 0x1) => format!("SKNP V{:X}", x),

            (0xF, _, _, 0x7) => format!("LD V{:X}, DT", x),
            (0xF, _, _, 0xA) => format!("LD V{:X}, K", x),
            (0xF, _, 0x1, 0x5) => format!("LD DT, V{:X}", x),
            (0xF, _, _, 0x8) => format!("LD ST, V{:X}", x),
            (0xF, _, _, 0xE) => format!("ADD I, V{:X}", x),
            (0xF, _, _, 0x9) => format!("LD F, V{:X}", x),
            (0xF, _, _, 0x3) => format!("LD B, V{:X}", x),
            (0xF, _, 0x5, 0x5) => format!("LD [I], V{:X}", x),
            (0xF, _, 0x6, 0x5) => format!("LD V{:X}, [I]", x),
            (_, _, _, _) => {
                let res = format!("UNK {:#X} => ({:#X}, {:#X}, {:#X}, {:#X})", self.opcode, hi_n, x, y, n);
                res
            }
        }
    }
}