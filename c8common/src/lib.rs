#![deny(missing_debug_implementations)]
#![warn(missing_copy_implementations)]

pub type Address = u16;
pub const NUMBER_OF_ADDRESSES: usize = ((Address::MAX >> 4) + 1) as usize;

mod instruction;
pub use instruction::{Instruction, InstructionDecodeError, ScreenInstruction};

mod data;
pub use data::Datum;

pub mod asm;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum GeneralRegister {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    VA,
    VB,
    VC,
    VD,
    VE,
    VF,
}
