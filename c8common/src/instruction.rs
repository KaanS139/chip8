use crate::{Address, Datum};
use log::error;

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
    pub fn try_from_datum(datum: Datum) -> Result<Self, InstructionDecodeError> {
        Err(InstructionDecodeError::IncompleteInstruction(vec![datum]))
    }

    pub fn try_from_data(
        mut previous: Vec<Datum>,
        datum: Datum,
    ) -> Result<Self, InstructionDecodeError> {
        previous.push(datum);
        let data = previous;
        match &data[..] {
            [_] => Err(InstructionDecodeError::IncompleteInstruction(data)),
            [d1, d2] => {
                if let Some(inst) = Self::try_second_order(*d1, *d2) {
                    return Ok(inst);
                }
                Err(InstructionDecodeError::IncompleteInstruction(data))
            }
            more => {
                error!("Too many bytes, invalid instruction! {:?}", more);
                Err(InstructionDecodeError::InvalidInstruction(data))
            }
        }
    }

    fn try_second_order(d1: Datum, d2: Datum) -> Option<Self> {
        let ((n1, n2), (n3, n4)) = (d1.as_nibbles(), d2.as_nibbles());
        match (n1.nibble(), n2.nibble(), n3.nibble(), n4.nibble()) {
            // https://rs118.uwcs.co.uk/chip8.html
            (0x0, 0x0, 0x0, 0x0) => Some(Self::Nop),
            (0x0, 0x0, 0xE, 0x0) => Some(Self::Screen(ScreenInstruction::Clear)),
            (0x1, a1, a2, a3) => Some(Self::Jump(
                ((a1 as u16) << 8) | ((a2 as u16) << 4) | a3 as u16,
            )),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum InstructionDecodeError {
    IncompleteInstruction(Vec<Datum>),
    InvalidInstruction(Vec<Datum>),
}

impl InstructionDecodeError {
    pub fn invalid_data(self) -> Option<Vec<Datum>> {
        match self {
            Self::InvalidInstruction(inner) => Some(inner),
            _ => None,
        }
    }
}
