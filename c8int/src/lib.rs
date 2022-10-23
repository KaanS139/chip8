mod interpreter;
pub use interpreter::Chip8Interpreter;

pub(crate) mod prelude {
    pub(crate) use c8common::{
        asm, memory::Memory, Address, Datum, Display, GeneralRegister, Instruction, RawInstruction,
    };
}
