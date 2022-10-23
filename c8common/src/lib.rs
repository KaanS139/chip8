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

pub mod control;
pub mod key;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum GeneralRegister {
    V0 = 0,
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
    V7 = 7,
    V8 = 8,
    V9 = 9,
    VA = 10,
    VB = 11,
    VC = 12,
    VD = 13,
    VE = 14,
    VF = 15,
}

impl GeneralRegister {
    pub fn index(self) -> usize {
        self as usize
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
