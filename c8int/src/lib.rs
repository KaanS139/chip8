#![deny(unused_must_use, missing_debug_implementations)]
#![warn(missing_copy_implementations)]

mod interpreter;
pub use interpreter::Chip8Interpreter;

pub(crate) mod prelude {
    pub(crate) use c8common::{
        asm, memory::Memory, Address, Datum, Display, GeneralRegister, Instruction, RawInstruction,
    };
}
