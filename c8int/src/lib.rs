mod interpreter;

pub use interpreter::Chip8Interpreter;

pub(crate) mod prelude {
    pub(crate) use chip8_base::{Display, Interpreter, Keys, Pixel};

    pub(crate) use c8common::{
        Address, Datum, Instruction, InstructionDecodeError, ScreenInstruction,
    };
}
