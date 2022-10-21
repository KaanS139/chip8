#![deny(missing_debug_implementations)]
#![warn(missing_copy_implementations)]

pub type Address = u16;

mod instruction;
pub use instruction::{Instruction, InstructionDecodeError, ScreenInstruction};

mod data;
pub use data::Datum;
