use crate::{Address, Datum};
use crate::InstructionDecodeError::InvalidInstruction;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Instruction {
    Nop,
    Screen(ScreenInstruction),
    Jump(Address),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ScreenInstruction {
    Clear,
}

impl Instruction {
    pub fn try_from_data(data: (Datum, Datum)) -> Result<Self, InstructionDecodeError> {
        let ((n1, n2), (n3, n4)) = (data.0.as_nibbles(), data.1.as_nibbles());
        match (n1.nibble(), n2.nibble(), n3.nibble(), n4.nibble()) {
            // https://rs118.uwcs.co.uk/chip8.html
            (0x0, 0x0, 0x0, 0x0) => Ok(Self::Nop),
            (0x0, 0x0, 0xE, 0x0) => Ok(Self::Screen(ScreenInstruction::Clear)),
            (0x1, a1, a2, a3) => Ok(Self::Jump(
                ((a1 as u16) << 8) | ((a2 as u16) << 4) | a3 as u16,
            )),
            _ => Err(InvalidInstruction(data)),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(missing_copy_implementations)]
pub enum InstructionDecodeError {
    InvalidInstruction((Datum, Datum)),
}

impl InstructionDecodeError {
    pub fn invalid_data(self) -> Option<(Datum, Datum)> {
        match self {
            Self::InvalidInstruction(inner) => Some(inner),
        }
    }
}
