#![deny(missing_debug_implementations)]
#![warn(missing_copy_implementations)]

mod address;
pub use address::Address;

pub const NUMBER_OF_ADDRESSES: usize = Address::NUMBER_OF_ADDRESSES;

mod instruction;
pub use instruction::{Instruction, InstructionDecodeError, RawInstruction};

mod data;
use crate::data::Nibble;
pub use data::Datum;

pub mod asm;

pub mod memory;

pub mod display;
pub use display::Display;

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

impl GeneralRegister {
    pub fn index(self) -> usize {
        match self {
            GeneralRegister::V0 => 0,
            GeneralRegister::V1 => 1,
            GeneralRegister::V2 => 2,
            GeneralRegister::V3 => 3,
            GeneralRegister::V4 => 4,
            GeneralRegister::V5 => 5,
            GeneralRegister::V6 => 6,
            GeneralRegister::V7 => 7,
            GeneralRegister::V8 => 8,
            GeneralRegister::V9 => 9,
            GeneralRegister::VA => 10,
            GeneralRegister::VB => 11,
            GeneralRegister::VC => 12,
            GeneralRegister::VD => 13,
            GeneralRegister::VE => 14,
            GeneralRegister::VF => 15,
        }
    }

    pub fn from_nibble(nibble: Nibble) -> Self {
        match nibble.as_half_byte() {
            0 => GeneralRegister::V0,
            1 => GeneralRegister::V1,
            2 => GeneralRegister::V2,
            3 => GeneralRegister::V3,
            4 => GeneralRegister::V4,
            5 => GeneralRegister::V5,
            6 => GeneralRegister::V6,
            7 => GeneralRegister::V7,
            8 => GeneralRegister::V8,
            9 => GeneralRegister::V9,
            10 => GeneralRegister::VA,
            11 => GeneralRegister::VB,
            12 => GeneralRegister::VC,
            13 => GeneralRegister::VD,
            14 => GeneralRegister::VE,
            15 => GeneralRegister::VF,
            _ => unreachable!(),
        }
    }
}
