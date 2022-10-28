pub mod compilation;
pub mod parsing;
pub mod tokenizing;

pub mod instruction_sets {
    mod chip8;
    pub use chip8::Chip8InstructionSet;
}
