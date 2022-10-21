use crate::{Address, Datum, GeneralRegister as VX, NUMBER_OF_ADDRESSES};
use log::error;
use std::collections::HashMap;
use std::ops::Index;
use tap::prelude::*;

pub mod parsing;

#[derive(Debug, Clone)]
#[allow(missing_copy_implementations)]
pub struct ROM([Datum; NUMBER_OF_ADDRESSES]);

impl ROM {
    pub fn new() -> Self {
        Self([Datum(0); NUMBER_OF_ADDRESSES])
    }

    pub fn save(&self, path: impl AsRef<std::path::Path>) {
        let buf = self.0.map(|datum| datum.0);
        std::fs::write(path, &buf).expect("failed to write to file!")
    }
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
            counter: 0,
        }
    }

    pub fn assemble(&self) -> ROM {
        let Self {
            labels,
            instructions,
            ..
        } = self;
        let mut out_rom = [Datum(0); NUMBER_OF_ADDRESSES];
        for i in 0..(NUMBER_OF_ADDRESSES >> 1) {
            let data = instructions[i].to_data(labels);
            out_rom[i * 2] = data.0;
            out_rom[i * 2 + 1] = data.1;
        }
        ROM(out_rom)
    }

    pub fn instruction(&mut self, instruction: AsmInstruction) -> &mut Self {
        self.instructions[self.counter as usize] = instruction;
        self.counter += 1;
        self
    }

    pub fn label(&mut self, name: String) -> &mut Self {
        let name_2 = name.clone();
        if let Some(old) = self.labels.insert(name, self.counter) {
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
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Source {
    Byte(u8),
    Register(VX),
}

#[derive(Debug, Clone)]
pub enum JumpAddress {
    Address(Address),
    Label(String),
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

#[derive(Debug)]
pub enum AsmInstruction {
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
}

impl AsmInstruction {
    fn to_data(&self, labels: &HashMap<String, Address>) -> (Datum, Datum) {
        let tuple = match self {
            AsmInstruction::NOP => (0x00, 0x00),
            #[allow(deprecated, clippy::identity_op)]
            AsmInstruction::SYS(addr) => (0x00 | (addr >> 8) as u8, (addr & 0xFF) as u8),
            AsmInstruction::CLS => (0x00, 0xE0),
            AsmInstruction::RET => (0x00, 0xEE),
            AsmInstruction::JP(to) => {
                let addr = match to {
                    JumpAddress::Label(s) => labels
                        .get(s)
                        .tap_none(|| error!("Label {} does not exist!", s))
                        .unwrap(),
                    JumpAddress::Address(addr) => addr,
                };
                (0x10 | (addr >> 8) as u8, (addr & 0xFF) as u8)
            }
            AsmInstruction::LD(_, _) => {
                todo!()
            }
        };

        (Datum(tuple.0), Datum(tuple.1))
    }
}
