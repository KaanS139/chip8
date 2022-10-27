use crate::tokenizing::{Item, Lexical, Punct, Spanned};
use crate::AsmInstruction;
use error::*;
use miette::SourceSpan;
use std::iter::Peekable;

pub fn parse(tokens: Vec<Spanned<Item>>) -> Result<Vec<ExecutionItem>, ConversionError> {
    Parser::new(tokens.into_iter()).convert()
}

struct Parser<T: Iterator<Item = Spanned<Item>>> {
    tokens: Peekable<T>,
    output: Vec<ExecutionItem>,
}

impl<T: Iterator<Item = Spanned<Item>>> Parser<T> {
    fn new(tokens: T) -> Self {
        Self {
            tokens: tokens.peekable(),
            output: vec![],
        }
    }

    fn convert(mut self) -> Result<Vec<ExecutionItem>, ConversionError> {
        loop {
            let line = match self.get_line() {
                Some(line) => line,
                None => {
                    return Ok(self.output);
                }
            };
            let action = dbg!(self.parse_line(line)?);
            self.output.push(action);
        }
    }

    fn get_line(&mut self) -> Option<Peekable<std::vec::IntoIter<Spanned<Item>>>> {
        let mut line = vec![];
        for next in self.tokens.by_ref() {
            if next.item == Item::Linebreak {
                break;
            }

            line.push(next);
        }
        Some(line.into_iter().peekable())
    }

    fn parse_line<S: Iterator<Item = Spanned<Item>>>(
        &self,
        mut line: Peekable<S>,
    ) -> Result<ExecutionItem, ConversionError> {
        match line.peek() {
            Some(first) => match first.item {
                Item::Lexical(Lexical::PrefixedIdent(_, _)) => Self::parse_line_internal(line),
                Item::Lexical(Lexical::Ident(_)) => {
                    let first = line.next().expect("known to exist by peeking");
                    if matches!(
                        line.peek().map(|s| &s.item),
                        Some(Item::Punct(Punct::Colon))
                    ) {
                        // This is a label, so we know nothing else is on this "line"
                        let label = first
                            .item
                            .to_lexical()
                            .expect("known correct by match")
                            .to_ident()
                            .expect("known correct by match");
                        return Ok(ExecutionItem::Label(Label::Direct(label)));
                    }
                    // This is an instruction
                    todo!()
                }
                Item::Lexical(Lexical::Numeric(_)) => {
                    // Raw data must use the `.data` internal attribute
                    Err(DataDefinitionError::exposed_data(
                        Self::get_total_span(&line.collect::<Vec<_>>()[..])
                            .expect("the span exists"),
                    ))?
                }
                Item::Punct(_) => Err(ConversionError::no_rules(first.at)),
                Item::Linebreak => Ok(ExecutionItem::Nothing),
            },
            None => Ok(ExecutionItem::Nothing),
        }
    }

    fn add_raw_data<S: Iterator<Item = Spanned<Item>>>(
        line: Peekable<S>,
    ) -> Result<ExecutionItem, ConversionError> {
        let mut data = vec![];
        let mut expecting_number = true;
        for Spanned { item, at } in line {
            if expecting_number {
                let number = item
                    .to_lexical()
                    .map(|i| i.as_numeric())
                    .flatten()
                    .ok_or_else(|| DataDefinitionError::data_entry(at, true))?;
                data.push(number);
                expecting_number = false;
            } else {
                if item.as_punct().map(|p| p == Punct::Comma) != Some(true) {
                    Err(DataDefinitionError::data_entry(at, false))?
                } else {
                    expecting_number = true;
                }
            }
        }
        assert!(
            !data.is_empty(),
            "there must be at least one number in here, guaranteed by peeking"
        );
        Ok(ExecutionItem::RawData(data))
    }

    fn parse_line_internal<S: Iterator<Item = Spanned<Item>>>(
        mut line: Peekable<S>,
    ) -> Result<ExecutionItem, ConversionError> {
        let token = line.next().expect("this is known to exist by peeking");
        let at = token.at;
        let (punct, ident) = token
            .item
            .to_lexical()
            .expect("known to be lexical")
            .to_prefixed()
            .expect("known to be a prefixed ident");
        match punct {
            Punct::Period => Self::parse_internal_item(ident, at, line),
            Punct::Dollar => {
                // Dollar item, first token of line => constant assignment
                let next_token = line
                    .next()
                    .ok_or_else(|| ConstantDefinitionError::constant_needs_value(at))?;
                let (at, value) = (next_token.at, next_token.item);
                let value: Value = {
                    match value {
                        Item::Lexical(Lexical::Numeric(number)) => Ok(Value::Numeric(number)),
                        Item::Lexical(Lexical::PrefixedIdent(prefix, ident))
                            if prefix == Punct::Dollar =>
                        {
                            Ok(Value::Constant(ident))
                        }
                        _ => Err(ConstantDefinitionError::constant_value_type(at)),
                    }?
                };
                Ok(ExecutionItem::DefineConstant { name: ident, value })
            }
            _ => panic!("Only `Period` and `Dollar` can be used as prefixes!"),
        }
    }

    fn parse_internal_item<S: Iterator<Item = Spanned<Item>>>(
        mut invocation: String,
        invocation_at: SourceSpan,
        mut line: Peekable<S>,
    ) -> Result<ExecutionItem, ConversionError> {
        invocation.make_ascii_lowercase();
        match &invocation[..] {
            "data" => todo!(),
            "name" => {
                let mut bindings: Vec<LocalBinding> = vec![];
                // let mut values = vec![];

                // loop {
                //     let name = line.next();
                //     if name.is_none() {
                //         break;
                //     }
                //     let Spanned { item, at } = name.unwrap();
                //     let name = item
                //         .to_lexical()
                //         .map(|i| i.to_ident())
                //         .flatten()
                //         .ok_or_else(|| NameDefinitionError::name_invalid_type(at))?;
                //     let Spanned { item, at } = line
                //         .next()
                //         .ok_or_else(|| NameDefinitionError::no_equals(at))?;
                //     if item != Item::Punct(Punct::Equals) {
                //         Err(NameDefinitionError::not_an_equals(at))?
                //     }
                //     let Spanned { item, at } = line
                //         .next()
                //         .ok_or_else(|| NameDefinitionError::missing_value(at))?;
                //     let item = item
                //         .to_lexical()
                //         .ok_or_else(|| NameDefinitionError::invalid_value_type(at))?;
                //     let value = match item {
                //         Lexical::PrefixedIdent(prefix, ident) => Some(match prefix {
                //             Punct::Period => Value::Local(ident),
                //             Punct::Dollar => Value::Constant(ident),
                //             _ => panic!("Only `Period` and `Dollar` can be used as prefixes!"),
                //         }),
                //         Lexical::Numeric(num) => Value::Numeric(num),
                //         _ => None,
                //     }
                //     .ok_or_else(|| NameDefinitionError::invalid_value_type(at))?;
                //     bindings.push(LocalBinding { name, value });
                // }

                dbg!(bindings);
                todo!()
            }
            "assert_addr" => {
                let addr = line
                    .next()
                    .ok_or_else(|| AssertDefinitionError::assert_missing_addr(invocation_at))?;
                let at = addr.at;
                let item = addr
                    .item
                    .to_lexical()
                    .ok_or_else(|| AssertDefinitionError::assert_addr_type(at))?;

                let target = if let Some(addr) = item.as_numeric() {
                    Some(Value::Numeric(addr))
                } else if let Some((prefix, name)) = item.to_prefixed() {
                    if prefix == Punct::Dollar {
                        Some(Value::Constant(name))
                    } else {
                        None
                    }
                } else {
                    None
                };

                let target = target.ok_or_else(|| AssertDefinitionError::assert_addr_type(at))?;

                let rest = line.collect::<Vec<_>>();
                if rest.is_empty() {
                    Ok(ExecutionItem::Label(Label::AssertAddress(target)))
                } else {
                    Err(AssertDefinitionError::assert_too_many(
                        Self::get_total_span(&rest).expect("line exists"),
                    ))?
                }
            }
            _ => Err(InvocationError::unknown_invocation(invocation_at))?,
        }
    }

    fn get_total_span(over: &[Spanned<Item>]) -> Option<SourceSpan> {
        let (a, b) = (&over.last()?.at, &over.first()?.at);
        let start = a.offset().min(b.offset());
        let end = (a.offset() + a.len()).max(b.offset() + b.len());
        Some((start, end - start).into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ident(String);

impl TryFrom<Spanned<Item>> for Spanned<Ident> {
    type Error = Spanned<Item>;

    fn try_from(value: Spanned<Item>) -> Result<Self, Self::Error> {
        if let Item::Lexical(Lexical::Ident(s)) = value.item {
            Ok(Self {
                item: Ident(s),
                at: value.at,
            })
        } else {
            Err(value)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value {
    Numeric(u16),
    Constant(String),
    Local(String),
}

#[derive(Debug, Clone)]
pub enum ExecutionItem {
    Nothing,
    DefineConstant { name: String, value: Value },
    BindLocal(Vec<LocalBinding>),
    Instruction(AsmInstruction),
    Label(Label),
    RawData(Vec<u16>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Label {
    Direct(String),
    /// The Value can only be numeric or a constant
    AssertAddress(Value),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LocalBinding {
    name: String,
    value: Value,
}

impl From<AsmInstruction> for ExecutionItem {
    fn from(inst: AsmInstruction) -> Self {
        Self::Instruction(inst)
    }
}

mod error {
    use crate::parsing::Ident;
    use crate::tokenizing::Spanned;
    use miette::{Diagnostic, SourceSpan};
    use tap::Conv;
    use thiserror::Error;

    #[derive(Debug, Error, Diagnostic)]
    pub enum ConversionError {
        #[error("No rules expected this token")]
        NoRules {
            #[label("here")]
            at: SourceSpan,
        },
        #[error("Expected an identifier")]
        ExpectedIdent {
            #[label("here")]
            at: SourceSpan,
        },

        #[error(transparent)]
        #[diagnostic(transparent)]
        Instruction(#[from] InstructionError),
        #[error(transparent)]
        #[diagnostic(transparent)]
        Invocation(#[from] InvocationError),
    }

    impl ConversionError {
        pub(super) fn no_rules(at: SourceSpan) -> Self {
            Self::NoRules { at }
        }

        pub(super) fn expecting_ident(at: SourceSpan) -> Self {
            Self::ExpectedIdent { at }
        }
    }

    impl InstructionError {
        pub(super) fn unknown_instruction(opcode: Spanned<Ident>) -> Self {
            Self::UnknownInstruction { opcode }
        }

        pub(super) fn too_many_arguments(range: SourceSpan) -> Self {
            Self::TooManyArguments { arguments: range }
        }
    }

    impl InvocationError {
        pub(super) fn unknown_invocation(at: SourceSpan) -> Self {
            Self::UnknownInvocation { at }.into()
        }
    }

    impl ConstantDefinitionError {
        pub(super) fn constant_needs_value(constant: SourceSpan) -> Self {
            Self::ConstantNeedsValue { constant }
        }

        pub(super) fn constant_value_type(value: SourceSpan) -> Self {
            Self::ConstantValueType { value }
        }
    }

    impl DataDefinitionError {
        pub(super) fn data_entry(item: SourceSpan, should_be_number: bool) -> Self {
            Self::DataEntryInvalidType {
                item,
                should_be: (if should_be_number {
                    "expected a number"
                } else {
                    "expected a comma, or the end of the list"
                })
                .to_string(),
            }
        }

        pub(super) fn exposed_data(at: SourceSpan) -> Self {
            Self::ExposedData { at }
        }
    }

    impl AssertDefinitionError {
        pub(super) fn assert_missing_addr(at: SourceSpan) -> Self {
            Self::AssertMissingAddr { at }
        }

        pub(super) fn assert_addr_type(at: SourceSpan) -> Self {
            Self::AssertAddrType { at }
        }

        pub(super) fn assert_too_many(at: SourceSpan) -> Self {
            Self::AssertAddrTooMany { at }
        }
    }

    impl NameDefinitionError {
        pub(super) fn name_invalid_type(at: SourceSpan) -> Self {
            Self::NameInvalidType { at }
        }
    }

    #[derive(Debug, Error, Diagnostic)]
    pub enum InstructionError {
        #[error("Unknown instruction '{}'", .opcode.item.0)]
        UnknownInstruction {
            #[label("here")]
            opcode: Spanned<Ident>,
        },
        #[error("Too many arguments")]
        TooManyArguments {
            #[label("here")]
            arguments: SourceSpan,
        },
    }

    #[derive(Debug, Error, Diagnostic)]
    pub enum InvocationError {
        #[error("Unknown invocation")]
        #[diagnostic(help("try one of `name`, `data` or `assert_addr`"))]
        UnknownInvocation {
            #[label("here")]
            at: SourceSpan,
        },

        #[error(transparent)]
        #[diagnostic(transparent)]
        Constant(#[from] ConstantDefinitionError),
        #[error(transparent)]
        #[diagnostic(transparent)]
        Data(#[from] DataDefinitionError),
        #[error(transparent)]
        #[diagnostic(transparent)]
        Assert(#[from] AssertDefinitionError),
        #[error(transparent)]
        #[diagnostic(transparent)]
        Name(#[from] NameDefinitionError),
    }

    #[derive(Debug, Error, Diagnostic)]
    pub enum ConstantDefinitionError {
        #[error("Constants must be defined with a value")]
        ConstantNeedsValue {
            #[label("here")]
            constant: SourceSpan,
        },
        #[error("Constant value must be numeric or constant")]
        ConstantValueType {
            #[label("here")]
            value: SourceSpan,
        },
    }

    #[derive(Debug, Error, Diagnostic)]
    pub enum DataDefinitionError {
        #[error("Invalid item in data list")]
        DataEntryInvalidType {
            #[label("here")]
            item: SourceSpan,
            #[help]
            should_be: String,
        },
        #[error("Raw data cannot be included as-is")]
        #[diagnostic(help("use `.data` to include arbitrary data"))]
        ExposedData {
            #[label("here")]
            at: SourceSpan,
        },
    }

    #[derive(Debug, Error, Diagnostic)]
    pub enum AssertDefinitionError {
        #[error("Asserting an address requires an address to assert")]
        AssertMissingAddr {
            #[label("here")]
            at: SourceSpan,
        },
        #[error("The address must be numeric")]
        AssertAddrType {
            #[label("here")]
            at: SourceSpan,
        },
        #[error("Assert expects a single address")]
        AssertAddrTooMany {
            #[label("here")]
            at: SourceSpan,
        },
    }

    #[derive(Debug, Error, Diagnostic)]
    pub enum NameDefinitionError {
        #[error("Invalid item in name list")]
        NameInvalidType {
            #[label("here")]
            at: SourceSpan,
        },
        // #[error("Raw data cannot be included as-is")]
        // ExposedData {
        //     #[label("here")]
        //     at: SourceSpan,
        // },
    }

    macro_rules! convert {
        ($($t:ty),+) => {
            $(impl From<$t> for ConversionError {
                fn from(e: $t) -> Self {
                    e.conv::<InvocationError>().into()
                }
            })+
        };
    }

    convert!(
        ConstantDefinitionError,
        DataDefinitionError,
        AssertDefinitionError,
        NameDefinitionError
    );
}
