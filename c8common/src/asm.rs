use crate::instruction::RawInstruction;
use crate::memory::Memory;
use crate::{Address, Datum, GeneralRegister as VX, NUMBER_OF_ADDRESSES};
use log::{error, info};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::{BitOrAssign, Index};
use std::path::Path;
use tap::prelude::*;

pub mod conversion;
pub mod tokenizing;

#[derive(Debug, Clone)]
#[allow(missing_copy_implementations)]
pub struct ROM([Datum; NUMBER_OF_ADDRESSES - 0x200]);

impl ROM {
    pub fn new() -> Self {
        Self([Datum(0); NUMBER_OF_ADDRESSES - 0x200])
    }

    pub fn save(&self, path: impl AsRef<std::path::Path>) {
        let buf = self.0.map(|datum| datum.0);
        std::fs::write(path, &buf).expect("failed to write to file!")
    }

    pub fn from_bytes(mut bytes: Vec<u8>) -> Result<Self, LoadError> {
        if bytes.len() < NUMBER_OF_ADDRESSES - 0x200 {
            info!(
                "Padding bytes from {} to {}",
                bytes.len(),
                NUMBER_OF_ADDRESSES - Address::PROGRAM_START_INDEX
            );
            bytes.extend(
                std::iter::repeat(0)
                    .take(NUMBER_OF_ADDRESSES - Address::PROGRAM_START_INDEX - bytes.len()),
            )
        }
        match bytes.len().cmp(&(NUMBER_OF_ADDRESSES - 0x200)) {
            Ordering::Less => panic!("Should not be possible!"),
            Ordering::Equal => {
                let bytes = bytes
                    .try_conv::<[u8; NUMBER_OF_ADDRESSES - Address::PROGRAM_START_INDEX]>()
                    .unwrap();
                let data = bytes.map(Datum);
                Ok(Self(data))
            }
            Ordering::Greater => Err(LoadError::WrongSize {
                size: bytes.len(),
                expected: NUMBER_OF_ADDRESSES - Address::PROGRAM_START_INDEX,
            }),
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, FileLoadError> {
        let file_contents = std::fs::read(path).map_err(FileLoadError::IO)?;
        Self::from_bytes(file_contents).map_err(FileLoadError::LoadError)
    }

    pub(crate) fn into_data(self) -> [Datum; NUMBER_OF_ADDRESSES - 0x200] {
        self.0
    }

    pub fn to_memory(self) -> Memory {
        Memory::from_rom(self)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum LoadError {
    WrongSize { size: usize, expected: usize },
}

#[derive(Debug)]
pub enum FileLoadError {
    IO(std::io::Error),
    LoadError(LoadError),
}

impl Default for ROM {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<Address> for ROM {
    type Output = Datum;

    fn index(&self, index: Address) -> &Self::Output {
        &self.0[index.try_conv::<usize>().unwrap()]
    }
}

// impl IndexMut<Address> for ROM {
//     fn index_mut(&mut self, index: Address) -> &mut Self::Output {
//         &mut self.0[index.try_conv::<usize>().unwrap()]
//     }
// }

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
        ROM(out_rom)
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
    fn to_data(&self, labels: &HashMap<String, Address>) -> RawInstruction {
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
