use super::tokenizing::{Item, Spanned};
use crate::asm::{AsmInstruction, JumpAddress};
use miette::SourceSpan;
use crate::Address;
use crate::asm::tokenizing::{Lexical, Punct};

pub fn convert(mut tokens: Vec<Spanned<Item>>) -> Result<Vec<ExecutionItem>, ConversionError> {
    // `tokens` are front-back ordered
    tokens.reverse();
    Converter::new(tokens).convert()
}

struct Converter {
    /// `tokens` are back-front ordered
    tokens: Vec<Spanned<Item>>,
    output: Vec<ExecutionItem>,
}

impl Converter {
    /// `tokens` are back-front ordered
    fn new(tokens: Vec<Spanned<Item>>) -> Self {
        Self {
            tokens,
            output: vec![],
        }
    }

    fn convert(mut self) -> Result<Vec<ExecutionItem>, ConversionError> {
        loop {
            let first_token = self.pop();
            if first_token.is_none() {
                return Ok(self.output);
            }
            let first_token = first_token.unwrap();

            match first_token.item {
                Item::Lexical(Lexical::Ident(ref ident)) => {
                    if self.peek().map(|it| matches!(it, Item::Punct(Punct::Colon))) == Some(true) {
                        self.output.push(ExecutionItem::Label(ident.to_owned()));
                        self.pop();
                        continue;
                    }

                    let line = self.get_line(first_token);
                    let inst = self.parse_line(line)?;
                    self.output.push(inst.into())
                }
                _ => Err(ConversionError::expecting_ident(first_token.at))?,
            }
        }
    }

    fn get_line(&mut self, first_token: Spanned<Item>) -> Vec<Spanned<Item>> {
        let mut line = vec![first_token];
        'line: loop {
            let next = self.pop();
            if next.is_none() {
                break 'line;
            }
            let next = next.unwrap();
            if next.item == Item::Linebreak {
                break 'line;
            }
            line.push(next);
        }
        line.reverse();
        line
    }

    fn parse_line(&self, mut line: Vec<Spanned<Item>>) -> Result<AsmInstruction, ConversionError> {
        let total_line_span = Self::get_total_span(&line[..]).unwrap();

        let first: Spanned<Ident> = line.pop().unwrap().try_into().unwrap();
        match &first.item.0.to_ascii_lowercase()[..] {
            "nop" => {
                if line.len() == 1 {
                    Ok(AsmInstruction::NOP)
                } else {
                    Err(ConversionError::too_many_arguments(
                        Self::get_total_span(&line[..]).unwrap_or(total_line_span),
                    ))
                }
            }
            "cls" => {
                if line.len() == 1 {
                    Ok(AsmInstruction::CLS)
                } else {
                    Err(ConversionError::too_many_arguments(
                        Self::get_total_span(&line[..]).unwrap_or(total_line_span),
                    ))
                }
            }
            "jp" => {
                if line.len() == 2 {
                    let arg = line.pop().unwrap();
                    match arg.item {
                        Item::Lexical(lex) => {
                            match lex {
                                Lexical::Ident(s) => Ok(AsmInstruction::JP(JumpAddress::Label(s))),
                                Lexical::Numeric(a) => Ok(AsmInstruction::JP(JumpAddress::Address(Address::new(a)))),
                            }

                        }
                        _ => Err(ConversionError::invalid_argument_type(arg.at)),
                    }
                } else {
                    Err(ConversionError::too_many_arguments(
                        Self::get_total_span(&line[1..]).unwrap(),
                    ))
                }
            }
            _ => Err(ConversionError::unknown_instruction(first)),
        }
    }

    fn get_total_span(over: &[Spanned<Item>]) -> Option<SourceSpan> {
        let (a, b) = (&over.last()?.at, &over.first()?.at);
        let start = a.offset().min(b.offset());
        let end = (a.offset() + a.len()).max(b.offset() + b.len());
        Some((start, end).into())
    }

    fn peek(&self) -> Option<&Item> {
        self.tokens.last().map(|si| &si.item)
    }

    fn pop(&mut self) -> Option<Spanned<Item>> {
        self.tokens.pop()
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

#[derive(Debug, Clone)]
pub enum ExecutionItem {
    Instruction(AsmInstruction),
    Label(String),
}

impl From<AsmInstruction> for ExecutionItem {
    fn from(inst: AsmInstruction) -> Self {
        Self::Instruction(inst)
    }
}

#[derive(Debug, Clone)]
pub enum ConversionError {
    ExpectedIdent { at: SourceSpan },
    UnknownInstruction { opcode: Spanned<Ident> },
    TooManyArguments { arguments: SourceSpan },
    InvalidArgumentType { argument: SourceSpan },
}

impl ConversionError {
    fn expecting_ident(at: SourceSpan) -> Self {
        Self::ExpectedIdent { at }
    }

    fn unknown_instruction(opcode: Spanned<Ident>) -> Self {
        Self::UnknownInstruction { opcode }
    }

    fn too_many_arguments(range: SourceSpan) -> Self {
        Self::TooManyArguments { arguments: range }
    }

    fn invalid_argument_type(argument: SourceSpan) -> Self {
        Self::InvalidArgumentType { argument }
    }
}
