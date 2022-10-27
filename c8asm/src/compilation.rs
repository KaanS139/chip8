use crate::parsing::{ExecutionItem, Label, LocalBinding, Value};
use crate::tokenizing::Spanned;
use c8common::asm::ROM;
use c8common::{Address, Datum, GeneralRegister as VX, NUMBER_OF_ADDRESSES};
use error::*;
use log::info;
use miette::SourceSpan;
use std::collections::HashMap;

pub fn compile(items: Vec<Spanned<ExecutionItem>>) -> Result<ROM, CompilationError> {
    Assembler::with(items).assemble()
}

#[derive(Debug)]
pub struct Assembler {
    items: Vec<Spanned<ExecutionItem>>,
}

impl Assembler {
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    #[allow(clippy::needless_update)]
    pub fn with(items: Vec<Spanned<ExecutionItem>>) -> Self {
        Self {
            items,
            ..Self::new()
        }
    }

    pub fn assemble(self) -> Result<ROM, CompilationError> {
        let Self { items } = self;
        let mut mapped_items: Vec<MappedItem> = vec![];
        let mut constants: HashMap<String, ConcreteValue> = HashMap::new();
        let mut locals: HashMap<String, ConcreteValue> = HashMap::new();

        for Spanned { item, at } in items {
            match item {
                ExecutionItem::Nothing => {}
                ExecutionItem::DefineConstant { name, value } => {
                    if constants.insert(
                        name.clone(),
                        ConcreteValue::create(value.spanned(at), &constants, &locals)?,
                    ).is_some() {
                        Err(ValueError::rebound_constant(name, at))?
                    }
                }
                ExecutionItem::BindLocal(bindings) => {
                    for LocalBinding { name, value } in bindings {
                        if locals.insert(
                            name.clone(),
                            ConcreteValue::create(value.spanned(at), &constants, &locals)?,
                        ).is_some() {
                            info!("Local name '{}' rebound", name);
                        }
                    }
                }
                ExecutionItem::Instruction { opcode, arguments } => {
                    mapped_items.push(Self::instruction(
                        opcode, at, arguments, &constants, &locals,
                    )?);
                }
                ExecutionItem::Label(label) => match label {
                    Label::Direct(name) => {
                        mapped_items.push(MappedItem::Label(name));
                    }
                    Label::AssertAddress(addr) => match addr {
                        Value::Numeric(i) => mapped_items.push(MappedItem::AssertAddress(i)),
                        Value::Constant(c) => {
                            let number = constants
                                .get(&c)
                                .ok_or_else(|| ValueError::no_constant(c, at))?
                                .numeric()
                                .ok_or_else(|| ValueError::assert_non_numeric(at))?;
                            mapped_items.push(MappedItem::AssertAddress(number));
                        }
                        _ => Err(ValueError::assert_non_numeric(at))?,
                    },
                },
                ExecutionItem::RawData(raw) => {
                    mapped_items.extend(raw.into_iter().map(MappedItem::RawDatum))
                }
            }
        }

        let mut out = [Datum(0); NUMBER_OF_ADDRESSES - Address::PROGRAM_START_INDEX];
        let mut counter = Address::PROGRAM_START_INDEX;
        for mapped in mapped_items {
            todo!()
        }

        Ok(ROM::containing(out))
    }

    fn instruction(
        opcode: String,
        at: SourceSpan,
        arguments: Vec<Value>,
        constants: &HashMap<String, ConcreteValue>,
        locals: &HashMap<String, ConcreteValue>,
    ) -> Result<MappedItem, ValueError> {
        Ok(MappedItem::Instruction {
            opcode,
            arguments: arguments
                .into_iter()
                .map(|value| match value {
                    Value::Numeric(i) => Ok(ConcreteValue::Numeric(i)),
                    Value::Constant(c) => constants
                        .get(&c)
                        .ok_or_else(|| ValueError::no_constant(c, at))
                        .map(|i| i.clone()),
                    Value::Local(local) => locals
                        .get(&local)
                        .ok_or_else(|| ValueError::no_local(local, at))
                        .map(|i| i.clone()),
                    Value::Name(name) => Ok(ConcreteValue::name(name, at)),
                })
                .collect::<Result<_, _>>()?,
        })
    }

    // pub fn instruction(&mut self, instruction: AsmInstruction) -> &mut Self {
    //     todo!()
    //     // self.instructions[self.counter.conv::<usize>()] = instruction;
    //     // self.counter.increment();
    //     // self
    // }

    pub fn raw_instruction(&mut self, _raw: u16) -> &mut Self {
        todo!()
        // #[allow(deprecated)]
        // self.instruction(AsmInstruction::RAW(raw))
    }

    pub fn label(&mut self, _name: String) -> &mut Self {
        todo!()
        // let name_2 = name.clone();
        // if let Some(old) = self.labels.insert(
        //     name,
        //     Address::new(self.counter.as_u16() + Address::PROGRAM_START.as_u16() + 1),
        // ) {
        //     error!(
        //         "Label {} has been overwritten! (from 0x{:X} to 0x{:X})",
        //         name_2, old, self.counter
        //     )
        // }
        // self
    }

    pub fn label_str(&mut self, name: &str) -> &mut Self {
        self.label(name.to_string())
    }

    // pub fn nop(&mut self) -> &mut Self {
    //     self.instruction(AsmInstruction::NOP)
    // }
    //
    // pub fn cls(&mut self) -> &mut Self {
    //     self.instruction(AsmInstruction::CLS)
    // }
    //
    // pub fn jump(&mut self, to: impl Into<JumpAddress>) -> &mut Self {
    //     self.instruction(AsmInstruction::JP(to.into()))
    // }
    //
    // pub fn rng(&mut self, reg: VX, byte: u8) -> &mut Self {
    //     self.instruction(AsmInstruction::RNG(reg, byte))
    // }
}
#[derive(Debug, Clone)]
pub enum MappedItem {
    Label(String),
    AssertAddress(u16),
    RawDatum(u16),
    Instruction {
        opcode: String,
        arguments: Vec<ConcreteValue>,
    },
}

#[derive(Debug, Clone)]
pub enum ConcreteValue {
    Numeric(u16),
    Register(VX),
    Label(String),
}

impl ConcreteValue {
    fn create(
        from: Spanned<Value>,
        constants: &HashMap<String, ConcreteValue>,
        locals: &HashMap<String, ConcreteValue>,
    ) -> Result<Self, ValueError> {
        match from.item {
            Value::Numeric(num) => Ok(Self::Numeric(num)),
            Value::Constant(name) => Ok(constants
                .get(&name)
                .ok_or_else(|| ValueError::no_constant(name, from.at))?
                .clone()),
            Value::Local(name) => Ok(locals
                .get(&name)
                .ok_or_else(|| ValueError::no_local(name, from.at))?
                .clone()),
            Value::Name(name) => Ok(Self::name(name, from.at)),
        }
    }

    fn name(name: String, at: SourceSpan) -> Self {
        let reg = VX::from_name(&name);

        match reg {
            Some(vx) => Self::Register(vx),
            None => Self::Label(name),
        }
    }

    fn numeric(&self) -> Option<u16> {
        match self {
            Self::Numeric(i) => Some(*i),
            _ => None,
        }
    }
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

mod error {
    use miette::{Diagnostic, SourceSpan};
    use thiserror::Error;

    #[derive(Debug, Error, Diagnostic)]
    pub enum CompilationError {
        #[error(transparent)]
        #[diagnostic(transparent)]
        ValueError(#[from] ValueError),
    }

    #[derive(Debug, Error, Diagnostic)]
    pub enum ValueError {
        #[error("Constant '{}' cannot be rebound", .name)]
        ReboundConstant {
            name: String,
            #[label("here")]
            at: SourceSpan,
        },
        #[error("No constant by the name '{}'", .name)]
        NoConstant {
            name: String,
            #[label("here")]
            at: SourceSpan,
        },
        #[error("No local by the name '{}'", .name)]
        NoLocal {
            name: String,
            #[label("here")]
            at: SourceSpan,
        },
        #[error("Asserts must use a numeric address")]
        AssertNonNumeric {
            #[label("here")]
            at: SourceSpan,
        },
    }

    impl ValueError {
        pub(super) fn no_constant(name: String, at: SourceSpan) -> Self {
            Self::NoConstant { name, at }
        }

        pub(super) fn no_local(name: String, at: SourceSpan) -> Self {
            Self::NoLocal { name, at }
        }

        pub(super) fn rebound_constant(name: String, at: SourceSpan) -> Self {
            Self::ReboundConstant { name, at }
        }

        pub(super) fn assert_non_numeric(at: SourceSpan) -> Self {
            Self::AssertNonNumeric { at }
        }
    }
}
