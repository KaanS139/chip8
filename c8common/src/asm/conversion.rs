use super::tokenizing::{Item, Location, SpannedItem};
use crate::asm::{AsmInstruction, JumpAddress};
use std::cmp::Ordering;
use std::ops::RangeInclusive;

pub fn convert(mut tokens: Vec<SpannedItem>) -> Result<Vec<ExecutionItem>, ConversionError> {
    // `tokens` are front-back ordered
    tokens.reverse();
    Converter::new(tokens).convert()
}

struct Converter {
    /// `tokens` are back-front ordered
    tokens: Vec<SpannedItem>,
    output: Vec<ExecutionItem>,
}

impl Converter {
    /// `tokens` are back-front ordered
    fn new(tokens: Vec<SpannedItem>) -> Self {
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
                Item::Ident(ref ident) => {
                    if self.peek().map(|it| matches!(it, Item::Colon)) == Some(true) {
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

    fn get_line(&mut self, first_token: SpannedItem) -> Vec<SpannedItem> {
        let mut line = vec![first_token];
        'line: loop {
            let next = self.pop();
            if next.is_none() {
                break 'line;
            }
            let next = next.unwrap();
            if next.item == Item::Semicolon {
                break 'line;
            }
            line.push(next);
        }
        line.reverse();
        line
    }

    fn parse_line(&self, mut line: Vec<SpannedItem>) -> Result<AsmInstruction, ConversionError> {
        let total_line_span = Self::get_total_span(&line[..]).unwrap();

        let first: SpannedIdent = line.pop().unwrap().try_into().unwrap();
        match &first.ident.to_ascii_lowercase()[..] {
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
                        Item::Ident(s) => Ok(AsmInstruction::JP(JumpAddress::Label(s))),
                        Item::Numeric(_) => Err(ConversionError::invalid_argument_type(arg.at)),
                        Item::Hash => Err(ConversionError::invalid_argument_type(arg.at)),
                        Item::Comma
                        | Item::Colon
                        | Item::Semicolon
                        | Item::OpenBrace
                        | Item::CloseBrace => Err(ConversionError::invalid_argument_type(arg.at)),
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

    fn get_total_span(over: &[SpannedItem]) -> Option<RangeInclusive<Location>> {
        let (mut a, mut b) = (&over.last()?.at, &over.first()?.at);
        let cmp = Location::compare_ranges(a, b);
        if cmp.is_none() {
            return Some(*a.start().min(b.start())..=*a.end().max(b.end()));
        }
        if cmp.unwrap() == Ordering::Greater {
            std::mem::swap(&mut a, &mut b);
        }
        Some(*a.start()..=*b.end())
    }

    fn peek(&self) -> Option<&Item> {
        self.tokens.last().map(|si| &si.item)
    }

    fn pop(&mut self) -> Option<SpannedItem> {
        self.tokens.pop()
    }
}

#[derive(Debug, Clone)]
pub struct SpannedIdent {
    ident: String,
    at: RangeInclusive<Location>,
}

impl TryFrom<SpannedItem> for SpannedIdent {
    type Error = SpannedItem;

    fn try_from(value: SpannedItem) -> Result<Self, Self::Error> {
        if let Item::Ident(s) = value.item {
            Ok(Self {
                ident: s,
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
    ExpectedIdent { at: RangeInclusive<Location> },
    UnknownInstruction { opcode: SpannedIdent },
    TooManyArguments { arguments: RangeInclusive<Location> },
    InvalidArgumentType { argument: RangeInclusive<Location> },
}

impl ConversionError {
    fn expecting_ident(at: RangeInclusive<Location>) -> Self {
        Self::ExpectedIdent { at }
    }

    fn unknown_instruction(opcode: SpannedIdent) -> Self {
        Self::UnknownInstruction { opcode }
    }

    fn too_many_arguments(range: RangeInclusive<Location>) -> Self {
        Self::TooManyArguments { arguments: range }
    }

    fn invalid_argument_type(argument: RangeInclusive<Location>) -> Self {
        Self::InvalidArgumentType { argument }
    }
}
