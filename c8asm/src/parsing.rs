use crate::tokenizing::{Item, Lexical, Punct, Spanned};
use error::*;
use miette::SourceSpan;
use std::iter::Peekable;

pub fn parse(tokens: Vec<Spanned<Item>>) -> Result<Vec<Spanned<ExecutionItem>>, ConversionError> {
    Parser::new(tokens.into_iter()).convert()
}

struct Parser<T: Iterator<Item = Spanned<Item>>> {
    tokens: Peekable<T>,
}

impl<T: Iterator<Item = Spanned<Item>>> Parser<T> {
    fn new(tokens: T) -> Self {
        Parser {
            tokens: tokens.peekable(),
        }
    }

    fn convert(mut self) -> Result<Vec<Spanned<ExecutionItem>>, ConversionError> {
        let mut output = vec![];
        loop {
            let line = match self.get_line() {
                Some(line) => line,
                None => {
                    return Ok(output);
                }
            };
            let action = self.parse_line(line)?;
            output.push(action);
        }
    }

    fn get_line(&mut self) -> Option<Peekable<std::vec::IntoIter<Spanned<Item>>>> {
        let mut line = vec![];
        let mut linebroken = false;
        for next in self.tokens.by_ref() {
            if next.item == Item::Linebreak {
                linebroken = true;
                break;
            }

            line.push(next);
        }
        if line.is_empty() && !linebroken {
            return None;
        }
        Some(line.into_iter().peekable())
    }

    fn parse_line<S: Iterator<Item = Spanned<Item>>>(
        &self,
        mut line: Peekable<S>,
    ) -> Result<Spanned<ExecutionItem>, ConversionError> {
        match line.peek() {
            Some(first) => match first.item {
                Item::Lexical(Lexical::PrefixedIdent(_, _)) => Ok(Self::parse_line_internal(line)?),
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
                        return Ok(ExecutionItem::Label(Label::Direct(label)).spanned(first.at));
                    }
                    // This is an instruction
                    let opcode = first
                        .item
                        .to_lexical()
                        .expect("known correct by match")
                        .to_ident()
                        .expect("known correct by match")
                        .to_ascii_lowercase();
                    let arguments = Self::get_instruction_arguments(line)?;
                    Ok(ExecutionItem::Instruction {
                        opcode,
                        arguments: arguments.item,
                    }
                    .spanned(long_span(first.at, arguments.at)))
                }
                Item::Lexical(Lexical::Numeric(_)) => Err(DataDefinitionError::exposed_data(
                    Self::get_total_span(&line.collect::<Vec<_>>()[..]).expect("the span exists"),
                ))?,
                Item::Punct(_) => Err(ConversionError::no_rules(first.at)),
                Item::Linebreak => Ok(ExecutionItem::Nothing.spanned(first.at)),
            },
            None => Ok(ExecutionItem::Nothing.spanned((0, 0).into())),
        }
    }

    fn add_raw_data<S: Iterator<Item = Spanned<Item>>>(
        line: Peekable<S>,
    ) -> Result<Spanned<ExecutionItem>, DataDefinitionError> {
        let mut data = vec![];
        let mut expecting_number = true;
        let mut total_span: Option<SourceSpan> = None;
        for Spanned { item, at } in line {
            if let Some(i) = total_span.as_mut() {
                *i = long_span(*i, at)
            }
            if expecting_number {
                let number = item
                    .to_lexical()
                    .and_then(|i| i.as_numeric())
                    .ok_or_else(|| DataDefinitionError::data_entry(at, true))?;
                let number = Self::parse_datum(number, at)?;
                data.push(number);
                expecting_number = false;
            } else if item.as_punct().map(|p| p == Punct::Comma) != Some(true) {
                Err(DataDefinitionError::data_entry(at, false))?
            } else {
                expecting_number = true;
            }
        }
        // assert!(
        //     !data.is_empty(),
        //     "there must be at least one number in here, guaranteed by peeking"
        // );
        Ok(ExecutionItem::RawData(data).spanned(total_span.unwrap_or_else(|| (0, 0).into())))
    }

    fn parse_datum(from: u16, at: SourceSpan) -> Result<u8, DataDefinitionError> {
        from.try_into()
            .map_err(|_| DataDefinitionError::number_too_big(from, at))
    }

    fn get_instruction_arguments<S: Iterator<Item = Spanned<Item>>>(
        mut line: Peekable<S>,
    ) -> Result<Spanned<Vec<Value>>, InstructionError> {
        let mut args = vec![];
        let mut expects_comma = false;

        let mut first_span = None;
        let mut last_span = None;
        loop {
            if expects_comma {
                let comma = line.next();
                if comma.is_none() {
                    break;
                }
                let Spanned { item, at } = comma.unwrap();
                if item.as_punct().map(|p| p == Punct::Comma) != Some(true) {
                    Err(InstructionError::expecting_comma(at))?
                }
            }
            expects_comma = true;
            if line.peek().map(|n| n.item == Item::Linebreak) == Some(true) {
                break;
            }
            let first = line.next();
            if first.is_none() {
                break;
            }
            let Spanned { item, at } = first.unwrap();
            if first_span.is_none() {
                first_span = Some(at);
            }
            last_span = Some(at);
            let item = item
                .to_lexical()
                .ok_or_else(|| InstructionError::invalid_arg_type(at))?;
            let value = match item {
                Lexical::PrefixedIdent(prefix, ident) => Some(match prefix {
                    Punct::Period => Value::Local(ident),
                    Punct::Dollar => Value::Constant(ident),
                    _ => panic!("Only `Period` and `Dollar` can be used as prefixes!"),
                }),
                Lexical::Numeric(num) => Some(Value::Numeric(num)),
                Lexical::Ident(ident) => Some(Value::name_or_label(ident)),
            }
            .ok_or_else(|| InstructionError::invalid_arg_type(at))?;
            args.push(value);
        }

        Ok(Spanned {
            item: args,
            at: long_span(
                first_span.unwrap_or_else(|| (0, 0).into()),
                last_span.unwrap_or_else(|| (0, 0).into()),
            ),
        })
    }

    fn parse_line_internal<S: Iterator<Item = Spanned<Item>>>(
        mut line: Peekable<S>,
    ) -> Result<Spanned<ExecutionItem>, InvocationError> {
        let token = line.next().expect("this is known to exist by peeking");
        let invocation_at = token.at;
        let (punct, ident) = token
            .item
            .to_lexical()
            .expect("known to be lexical")
            .to_prefixed()
            .expect("known to be a prefixed ident");
        match punct {
            Punct::Period => Self::parse_internal_item(ident, invocation_at, line),
            Punct::Dollar => {
                // Dollar item, first token of line => constant assignment
                let Spanned { item: value, at } = line
                    .next()
                    .ok_or_else(|| ConstantDefinitionError::constant_needs_value(invocation_at))?;
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
                Ok(ExecutionItem::DefineConstant { name: ident, value }
                    .spanned(long_span(invocation_at, at)))
            }
            _ => panic!("Only `Period` and `Dollar` can be used as prefixes!"),
        }
    }

    fn parse_internal_item<S: Iterator<Item = Spanned<Item>>>(
        mut invocation: String,
        invocation_at: SourceSpan,
        mut line: Peekable<S>,
    ) -> Result<Spanned<ExecutionItem>, InvocationError> {
        invocation.make_ascii_lowercase();
        match &invocation[..] {
            "data" => Ok(Self::add_raw_data(line)?),
            "name" => {
                let mut bindings: Vec<LocalBinding> = vec![];
                let mut expects_comma = false;
                let mut total_span: Option<SourceSpan> = None;

                loop {
                    if expects_comma {
                        let comma = line.next();
                        if comma.is_none() {
                            break;
                        }
                        let Spanned { item, at } = comma.unwrap();
                        if item.as_punct().map(|p| p == Punct::Comma) != Some(true) {
                            Err(NameDefinitionError::expecting_comma(at))?
                        }
                    }
                    expects_comma = true;
                    // if line.peek().map(|n| n.item == Item::Linebreak) == Some(true) {
                    //     break;
                    // } TODO
                    let name = line.next();
                    if name.is_none() {
                        break;
                    }
                    let Spanned { item, at } = name.unwrap();
                    if let Some(i) = total_span.as_mut() {
                        *i = long_span(*i, at)
                    }
                    let name = item
                        .to_lexical()
                        .and_then(Lexical::to_ident)
                        .ok_or_else(|| NameDefinitionError::name_invalid_type(at))?;
                    let Spanned { item, at } = line
                        .next()
                        .ok_or_else(|| NameDefinitionError::no_equals(at))?;
                    if item != Item::Punct(Punct::Equals) {
                        Err(NameDefinitionError::not_an_equals(at))?
                    }
                    let Spanned { item, at } = line
                        .next()
                        .ok_or_else(|| NameDefinitionError::missing_value(at))?;
                    if let Some(i) = total_span.as_mut() {
                        *i = long_span(*i, at)
                    }
                    let item = item
                        .to_lexical()
                        .ok_or_else(|| NameDefinitionError::invalid_value_type(at))?;
                    let value = match item {
                        Lexical::PrefixedIdent(prefix, ident) => Some(match prefix {
                            Punct::Period => Value::Local(ident),
                            Punct::Dollar => Value::Constant(ident),
                            _ => panic!("Only `Period` and `Dollar` can be used as prefixes!"),
                        }),
                        Lexical::Numeric(num) => Some(Value::Numeric(num)),
                        Lexical::Ident(ident) => Some(Value::name_or_label(ident)),
                    }
                    .ok_or_else(|| NameDefinitionError::invalid_value_type(at))?;
                    bindings.push(LocalBinding { name, value });
                }

                Ok(ExecutionItem::BindLocal(bindings)
                    .spanned(total_span.unwrap_or_else(|| (0, 0).into())))
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
                if rest.is_empty()
                /*|| rest.get(0).unwrap().item == Item::Linebreak TODO */
                {
                    Ok(ExecutionItem::Label(Label::AssertAddress(target)).spanned(at))
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

fn long_span(start: SourceSpan, end: SourceSpan) -> SourceSpan {
    (
        start.offset(),
        end.offset().max(start.offset()) + end.len() - start.offset(),
    )
        .into()
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
    Name(ReservedName),
    Label(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReservedName {
    /// Address register
    I,
    /// Delay timer
    DT,
    /// Sound timer
    ST,
    /// Load from key
    K,
    /// Sprite flag
    F,
    /// Binary coded decimal flag
    B,
}

impl Value {
    pub(crate) fn spanned(self, at: SourceSpan) -> Spanned<Self> {
        Spanned { item: self, at }
    }

    pub(crate) fn name_or_label(name: String) -> Self {
        Self::Name(match &name.to_ascii_uppercase()[..] {
            "I" => ReservedName::I,
            "DT" => ReservedName::DT,
            "ST" => ReservedName::ST,
            "K" => ReservedName::K,
            "F" => ReservedName::F,
            "B" => ReservedName::B,
            _ => return Self::Label(name),
        })
    }
}

#[derive(Debug, Clone)]
pub enum ExecutionItem {
    Nothing,
    DefineConstant {
        name: String,
        value: Value,
    },
    BindLocal(Vec<LocalBinding>),
    Instruction {
        opcode: String,
        arguments: Vec<Value>,
    },
    Label(Label),
    RawData(Vec<u8>),
}

impl ExecutionItem {
    fn spanned(self, at: SourceSpan) -> Spanned<Self> {
        Spanned { item: self, at }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Label {
    Direct(String),
    /// The Value can only be numeric or a constant
    AssertAddress(Value),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LocalBinding {
    pub(crate) name: String,
    pub(crate) value: Value,
}

mod error {
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
    }

    impl InstructionError {
        pub(super) fn expecting_comma(at: SourceSpan) -> Self {
            Self::ExpectedComma { at }
        }

        pub(super) fn invalid_arg_type(at: SourceSpan) -> Self {
            Self::InvalidArgType { at }
        }
    }

    impl InvocationError {
        pub(super) fn unknown_invocation(at: SourceSpan) -> Self {
            Self::UnknownInvocation { at }
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

        pub(super) fn number_too_big(number: u16, at: SourceSpan) -> Self {
            let (high, low) = ((number & 0xFF00) >> 8, number & 0xFF);
            Self::NumberTooBig {
                number,
                at,
                help: format!(
                    "Try splitting into two bytes: 0x{:02X}, 0x{:02X}",
                    high, low
                ),
            }
        }
    }

    impl AssertDefinitionError {
        pub(super) fn assert_missing_addr(at: SourceSpan) -> Self {
            Self::MissingAddr { at }
        }

        pub(super) fn assert_addr_type(at: SourceSpan) -> Self {
            Self::AddrType { at }
        }

        pub(super) fn assert_too_many(at: SourceSpan) -> Self {
            Self::AddrTooMany { at }
        }
    }

    impl NameDefinitionError {
        pub(super) fn name_invalid_type(at: SourceSpan) -> Self {
            Self::NameInvalidType { at }
        }

        pub(super) fn expecting_comma(at: SourceSpan) -> Self {
            Self::ExpectedComma { at }
        }

        pub(super) fn no_equals(after: SourceSpan) -> Self {
            Self::MissingEquals { after }
        }

        pub(super) fn not_an_equals(at: SourceSpan) -> Self {
            Self::NotAnEquals { at }
        }

        pub(super) fn missing_value(after: SourceSpan) -> Self {
            Self::MissingValue { after }
        }

        pub(super) fn invalid_value_type(at: SourceSpan) -> Self {
            Self::InvalidValueType { at }
        }
    }

    #[derive(Debug, Error, Diagnostic)]
    pub enum InstructionError {
        #[error("Expected a comma")]
        ExpectedComma {
            #[label("here")]
            at: SourceSpan,
        },
        #[error("Invalid type for arguments")]
        InvalidArgType {
            #[label("here")]
            at: SourceSpan,
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
        #[error("This number is too big to be used as a piece of raw data")]
        NumberTooBig {
            number: u16,
            #[label("here")]
            at: SourceSpan,
            #[help]
            help: String,
        },
    }

    #[derive(Debug, Error, Diagnostic)]
    pub enum AssertDefinitionError {
        #[error("Asserting an address requires an address to assert")]
        MissingAddr {
            #[label("here")]
            at: SourceSpan,
        },
        #[error("The address must be numeric")]
        AddrType {
            #[label("here")]
            at: SourceSpan,
        },
        #[error("Assert expects a single address")]
        AddrTooMany {
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
        #[error("Expected a comma")]
        ExpectedComma {
            #[label("here")]
            at: SourceSpan,
        },
        #[error("Expected an equals sign")]
        MissingEquals {
            #[label("after this")]
            after: SourceSpan,
        },
        #[error("Expected an equals sign")]
        NotAnEquals {
            #[label("here")]
            at: SourceSpan,
        },
        #[error("Missing a value for assignment")]
        MissingValue {
            #[label("after this")]
            after: SourceSpan,
        },
        #[error("Invalid type for assignment")]
        InvalidValueType {
            #[label("here")]
            at: SourceSpan,
        },
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
