use crate::parsing::{ExecutionItem, Label, LocalBinding, ReservedName, Value};
use crate::tokenizing::Spanned;
use c8common::asm::ROM;
use c8common::{Address, Datum, GeneralRegister as VX, NUMBER_OF_ADDRESSES};
pub use error::*;
use log::info;
use miette::SourceSpan;
use std::collections::HashMap;

pub fn compile<B: InstructionBuilder>(
    items: Vec<Spanned<ExecutionItem>>,
) -> Result<ROM, CompilationError> {
    Assembler::with(items).assemble::<B>()
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

    pub fn assemble<B: InstructionBuilder>(self) -> Result<ROM, CompilationError> {
        let Self { items } = self;
        let mut mapped_items: Vec<MappedItem> = vec![];
        let mut constants: HashMap<String, ConcreteValue> = HashMap::new();
        let mut locals: HashMap<String, ConcreteValue> = HashMap::new();

        for Spanned { item, at } in items {
            match item {
                ExecutionItem::Nothing => {}
                ExecutionItem::DefineConstant { name, value } => {
                    if constants
                        .insert(
                            name.clone(),
                            ConcreteValue::create(value.spanned(at), &constants, &locals)?,
                        )
                        .is_some()
                    {
                        Err(ValueError::rebound_constant(name, at))?
                    }
                }
                ExecutionItem::BindLocal(bindings) => {
                    for LocalBinding { name, value } in bindings {
                        if locals
                            .insert(
                                name.clone(),
                                ConcreteValue::create(value.spanned(at), &constants, &locals)?,
                            )
                            .is_some()
                        {
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
                        Value::Numeric(i) => {
                            mapped_items.push(MappedItem::AssertAddress(Spanned { item: i, at }))
                        }
                        Value::Constant(c) => {
                            let number = constants
                                .get(&c)
                                .ok_or_else(|| ValueError::no_constant(c, at))?
                                .numeric()
                                .ok_or_else(|| ValueError::assert_non_numeric(at))?;
                            mapped_items
                                .push(MappedItem::AssertAddress(Spanned { item: number, at }));
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
        let mut labels = HashMap::new();
        for mapped in mapped_items.iter() {
            match mapped {
                MappedItem::Label(label) => {
                    if labels.insert(label.clone(), counter as u16).is_some() {
                        Err(CompilationError::label_twice(label.clone()))?
                    }
                }
                MappedItem::AssertAddress(Spanned { item, at }) => {
                    if counter != *item as usize {
                        return Err(CompilationError::assert_failed(
                            *item as usize,
                            counter,
                            *at,
                        ));
                    }
                }
                MappedItem::RawDatum(_) => {
                    counter += 1;
                }
                MappedItem::Instruction { .. } => {
                    counter += 2;
                }
            }
        }
        let mut counter = Address::PROGRAM_START_INDEX;
        for mapped in mapped_items.into_iter() {
            match mapped {
                MappedItem::RawDatum(raw) => {
                    out[counter - Address::PROGRAM_START_INDEX] = Datum(raw);
                    counter += 1;
                }
                MappedItem::Instruction {
                    opcode,
                    at,
                    arguments,
                } => {
                    let (high, low) = B::instruction(opcode, arguments, at, &labels)?;
                    // dbg!(format!("0x{:04X}", u16::from_be_bytes([high, low])));
                    out[counter - Address::PROGRAM_START_INDEX] = Datum(high);
                    counter += 1;
                    out[counter - Address::PROGRAM_START_INDEX] = Datum(low);
                    counter += 1;
                }
                MappedItem::Label(_) | MappedItem::AssertAddress(_) => {}
            }
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
            at,
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
                    Value::Name(name) => Ok(ConcreteValue::Reserved(name)),
                    Value::Label(label) => Ok(ConcreteValue::name(label)),
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
    AssertAddress(Spanned<u16>),
    RawDatum(u8),
    Instruction {
        opcode: String,
        at: SourceSpan,
        arguments: Vec<ConcreteValue>,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConcreteValue {
    Numeric(u16),
    Register(VX),
    Label(String),
    Reserved(ReservedName),
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
            Value::Name(name) => Ok(Self::Reserved(name)),
            Value::Label(label) => Ok(Self::name(label)),
        }
    }

    fn name(name: String) -> Self {
        let reg = VX::from_name(&name);

        match reg {
            Some(vx) => Self::Register(vx),
            None => Self::Label(name),
        }
    }

    pub fn numeric(&self) -> Option<u16> {
        match self {
            Self::Numeric(i) => Some(*i),
            _ => None,
        }
    }

    pub fn bake_label(
        self,
        at: SourceSpan,
        labels: &HashMap<String, u16>,
    ) -> Result<Self, InstructionError> {
        match self {
            Self::Label(label) => {
                Ok(Self::Numeric(*labels.get(&label).ok_or_else(|| {
                    InstructionError::missing_label(at, label)
                })?))
            }
            other => Ok(other),
        }
    }

    pub fn register(&self) -> Option<VX> {
        match self {
            ConcreteValue::Register(reg) => Some(*reg),
            _ => None,
        }
    }
}

pub trait InstructionBuilder {
    fn instruction(
        opcode: String,
        arguments: Vec<ConcreteValue>,
        at: SourceSpan,
        labels: &HashMap<String, u16>,
    ) -> Result<(u8, u8), InstructionError>;
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

mod error {
    use crate::compilation::ConcreteValue;
    use c8common::GeneralRegister;
    use miette::{Diagnostic, SourceSpan};
    use std::convert::Infallible;
    use std::fmt::Display;
    use thiserror::Error;

    #[derive(Debug, Error, Diagnostic)]
    pub enum InstructionError {
        #[error("Unknown instruction '{}'", .opcode)]
        UnknownInstruction {
            opcode: String,
            #[label("here")]
            at: SourceSpan,
        },
        #[error("Invalid arguments")]
        InvalidArguments {
            #[label("here")]
            at: SourceSpan,
            #[help]
            reason: Option<String>,
        },
        #[error("Label '{}' is unknown", .label)]
        MissingLabel {
            #[label("here")]
            at: SourceSpan,
            label: String,
        },
    }

    impl InstructionError {
        pub fn not_enough_arguments(at: SourceSpan, expected: usize, got: usize) -> Self {
            assert!(expected > got);
            Self::InvalidArguments {
                at,
                reason: Some(format!(
                    "Not enough arguments: Expected {}, got {}",
                    expected, got
                )),
            }
        }

        pub fn too_many_arguments(at: SourceSpan, expected: usize, got: usize) -> Self {
            assert!(expected < got);
            Self::InvalidArguments {
                at,
                reason: Some(format!(
                    "Too many arguments: Expected {}, got {}",
                    expected, got
                )),
            }
        }

        pub fn invalid_type(at: SourceSpan, expected: impl Display, got: impl Display) -> Self {
            Self::InvalidArguments {
                at,
                reason: Some(format!(
                    "Invalid argument type: expected {} but got {}",
                    expected, got
                )),
            }
        }

        pub fn invalid_load(at: SourceSpan) -> Self {
            Self::InvalidArguments {
                at,
                reason: Some("Not a valid LD combination".to_string()),
            }
        }

        pub fn address_too_large(at: SourceSpan, address: u16) -> Self {
            Self::InvalidArguments {
                at,
                reason: Some(format!("Address 0x{:04X} is too large", address)),
            }
        }

        pub fn address(at: SourceSpan, address: u16) -> Result<u16, Self> {
            if address & 0xF000 != 0 {
                Err(Self::address_too_large(at, address))
            } else {
                Ok(address)
            }
        }

        pub fn expects_byte(at: SourceSpan, data: u16) -> Result<u8, Self> {
            let [high, low] = data.to_be_bytes();
            if high != 0 {
                Err(Self::InvalidArguments {
                    at,
                    reason: Some(format!("Expected a single byte, got 0x{:04X}", data)),
                })
            } else {
                Ok(low)
            }
        }

        pub fn expects_nibble(at: SourceSpan, data: u16) -> Result<u8, Self> {
            if data > 0xF {
                Err(Self::InvalidArguments {
                    at,
                    reason: Some(format!("Expected a half-byte, got 0x{:04X}", data)),
                })
            } else {
                Ok(data as u8)
            }
        }

        pub fn expects_register(
            at: SourceSpan,
            from: ConcreteValue,
        ) -> Result<GeneralRegister, Self> {
            match from {
                ConcreteValue::Register(reg) => Ok(reg),
                _ => Err(Self::InvalidArguments {
                    at,
                    reason: Some("Expected a register".to_string()),
                }),
            }
        }

        pub fn missing_label(at: SourceSpan, label: String) -> Self {
            Self::MissingLabel { at, label }
        }
    }

    #[derive(Debug, Error, Diagnostic)]
    pub enum CompilationError {
        #[error(transparent)]
        #[diagnostic(transparent)]
        ValueError(#[from] ValueError),
        #[error(transparent)]
        #[diagnostic(transparent)]
        InstructionError(#[from] InstructionError),

        #[error("The label '{}' has been defined twice", .name)]
        RedefinedLabel { name: String },

        #[error("Expected an address of 0x{:03X}, found an address of 0x{:03X}", .expected, .got)]
        AssertFailed {
            expected: usize,
            got: usize,
            #[label("here")]
            at: SourceSpan,
        },
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

    impl CompilationError {
        pub(super) fn label_twice(name: String) -> Self {
            Self::RedefinedLabel { name }
        }

        pub(super) fn assert_failed(expected: usize, got: usize, at: SourceSpan) -> Self {
            Self::AssertFailed { expected, got, at }
        }
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
