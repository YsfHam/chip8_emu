use super::instruction::Instruction;


#[derive(Debug)]
pub enum Chip8ErrorKind {
    SegmentationFault,
    UnknownInstruction(Instruction),
    EmptyStack,
    StackOverflow,
}

impl std::fmt::Display for Chip8ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Chip8ErrorKind::SegmentationFault => {
                write!(f, "Segmentation fault")
            },
            Chip8ErrorKind::UnknownInstruction(instr) => {
                write!(f, "Unknown instruction, opcode {:#X}", instr.opcode())
            },
            Chip8ErrorKind::EmptyStack => {
                write!(f, "The stack is empty!")

            },
            Chip8ErrorKind::StackOverflow => {
                write!(f, "Stack overflow")

            },
        }
    }
}