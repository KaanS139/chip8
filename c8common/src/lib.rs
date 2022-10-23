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
        Self::from_byte(nibble.as_half_byte())
    }

    pub fn from_byte(index: u8) -> Self {
        match index {
            0 => Self::V0,
            1 => Self::V1,
            2 => Self::V2,
            3 => Self::V3,
            4 => Self::V4,
            5 => Self::V5,
            6 => Self::V6,
            7 => Self::V7,
            8 => Self::V8,
            9 => Self::V9,
            10 => Self::VA,
            11 => Self::VB,
            12 => Self::VC,
            13 => Self::VD,
            14 => Self::VE,
            15 => Self::VF,
            index => panic!("Invalid index for register! {}", index),
        }
    }

    pub fn until_including(self) -> impl Iterator<Item = Self> {
        (0..=(self as usize as u8)).map(Self::from_byte)
    }
}
