use c8common::asm::ROM;
use c8common::{
    instruction::RawInstruction, Address, Datum, GeneralRegister as VX, NUMBER_OF_ADDRESSES,
};
use log::error;
use std::{collections::HashMap, ops::BitOrAssign};
use tap::prelude::*;

pub mod parsing;
pub mod tokenizing;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Source {
    Byte(u8),
    Register(VX),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum JumpAddress {
    Address(Address),
    Label(String),
    Relative(Address),
}

impl From<String> for JumpAddress {
    fn from(string: String) -> Self {
        Self::Label(string)
    }
}

impl From<&str> for JumpAddress {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

impl From<Address> for JumpAddress {
    fn from(addr: Address) -> Self {
        Self::Address(addr)
    }
}

impl From<u16> for JumpAddress {
    fn from(addr: u16) -> Self {
        Address::new(addr).into()
    }
}

#[derive(Debug)]
pub struct Assembler {
    instructions: [AsmInstruction; NUMBER_OF_ADDRESSES >> 1],
    labels: HashMap<String, Address>,
    counter: Address,
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            instructions: [(); NUMBER_OF_ADDRESSES >> 1].map(|_| AsmInstruction::NOP),
            labels: Default::default(),
            counter: Address::ZERO,
        }
    }

    pub fn assemble(&self) -> ROM {
        let Self {
            labels,
            instructions,
            ..
        } = self;
        let mut out_rom = [Datum(0); NUMBER_OF_ADDRESSES - 0x200];
        for i in 0..((NUMBER_OF_ADDRESSES - 0x200) >> 1) {
            let data = instructions[i].to_data(labels);
            out_rom[i * 2] = data.first();
            out_rom[i * 2 + 1] = data.second();
        }
        ROM::containing(out_rom)
    }

    pub fn instruction(&mut self, instruction: AsmInstruction) -> &mut Self {
        self.instructions[self.counter.conv::<usize>()] = instruction;
        self.counter.increment();
        self
    }

    pub fn raw_instruction(&mut self, raw: u16) -> &mut Self {
        #[allow(deprecated)]
        self.instruction(AsmInstruction::RAW(raw))
    }

    pub fn label(&mut self, name: String) -> &mut Self {
        let name_2 = name.clone();
        if let Some(old) = self.labels.insert(
            name,
            Address::new(self.counter.as_u16() + Address::PROGRAM_START.as_u16() + 1),
        ) {
            error!(
                "Label {} has been overwritten! (from 0x{:X} to 0x{:X})",
                name_2, old, self.counter
            )
        }
        self
    }

    pub fn label_str(&mut self, name: &str) -> &mut Self {
        self.label(name.to_string())
    }

    pub fn nop(&mut self) -> &mut Self {
        self.instruction(AsmInstruction::NOP)
    }

    pub fn cls(&mut self) -> &mut Self {
        self.instruction(AsmInstruction::CLS)
    }

    pub fn jump(&mut self, to: impl Into<JumpAddress>) -> &mut Self {
        self.instruction(AsmInstruction::JP(to.into()))
    }

    pub fn rng(&mut self, reg: VX, byte: u8) -> &mut Self {
        self.instruction(AsmInstruction::RNG(reg, byte))
    }
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AsmInstruction {
    #[deprecated]
    /// Used for adding a raw instruction to the assembler
    /// If an alternative exists, use that instead.
    RAW(u16),

    NOP,
    #[deprecated]
    /// Jump to a machine code routine at nnn.
    /// This instruction is only used on the old computers on which Chip-8 was originally implemented. It is ignored by modern interpreters.
    SYS(Address),
    /// Clear the display.
    CLS,
    /// Return from a subroutine.
    /// The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    RET,
    /// Jump to location nnn.
    /// The interpreter sets the program counter to nnn.
    JP(JumpAddress),

    /// For Source = Byte : The interpreter puts the value kk into register Vx.
    /// For Source = Register : Stores the value of register Vy in register Vx.
    LD(VX, Source),

    /// Generate a random byte using u8 as a mask, and store it in _Vx_
    RNG(VX, u8),
}

impl AsmInstruction {
    pub fn to_data(&self, labels: &HashMap<String, Address>) -> RawInstruction {
        match self {
            #[allow(deprecated)]
            Self::RAW(raw) => raw.into(),
            Self::NOP => 0x0000.into(),
            #[allow(deprecated, clippy::identity_op)]
            Self::SYS(addr) => {
                let mut raw = RawInstruction::from_raw_bytes(addr.to_bytes());
                raw.highest().bitor_assign(0x00);
                raw
            }
            Self::CLS => 0x00E0.into(),
            Self::RET => 0x00EE.into(),
            Self::JP(to) => {
                if let JumpAddress::Relative(rel_addr) = to {
                    let mut raw = RawInstruction::from_raw_bytes(rel_addr.to_bytes());
                    raw.highest().bitor_assign(0xB0);
                    raw
                } else {
                    let addr = match to {
                        JumpAddress::Label(s) => labels
                            .get(s)
                            .tap_none(|| error!("Label {} does not exist!", s))
                            .unwrap(),
                        JumpAddress::Address(addr) => addr,
                        JumpAddress::Relative(_) => unreachable!(),
                    };
                    let mut raw = RawInstruction::from_raw_bytes(addr.to_bytes());
                    raw.highest().bitor_assign(0x10);
                    raw
                }
            }
            Self::LD(_, _) => {
                todo!()
            }
            Self::RNG(reg, byte) => {
                let mut raw = RawInstruction::from_raw_bytes([*reg as usize as u8, *byte]);
                raw.highest().bitor_assign(0xC0);
                raw
            }
        }
    }
}
