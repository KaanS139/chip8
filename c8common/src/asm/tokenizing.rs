use std::cmp::Ordering;
use std::ops::RangeInclusive;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Item {
    Lexical(Lexical),
    Punct(Punct),
    Linebreak,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Lexical {
    Ident(String),
    Numeric(u16),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Punct {
    Comma,
    Period,
    Colon,
    Dollar,
    Equals,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Spanned<T> {
    pub(crate) item: T,
    pub(crate) at: SourceSpan,
}

impl<T> Spanned<T> {
    fn new(item: T, at: SourceSpan) -> Self {
        Self { item, at }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Location {
    byte: usize,
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Location {
    fn cmp(&self, other: &Self) -> Ordering {
        self.byte.cmp(&other.byte)
    }
}

impl Location {
    pub fn new(at: usize) -> Self {
        Self {
            byte: at
        }
    }

    pub fn compare_ranges(a: &RangeInclusive<Self>, b: &RangeInclusive<Self>) -> Option<Ordering> {
        if a.end() >= b.start() || b.end() >= a.start() {
            None
        } else {
            a.start().partial_cmp(b.start())
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum MultiCharItem {
    Ident(String),
    Numeric(String),
}

impl MultiCharItem {
    fn string(s: impl Into<String>) -> Self {
        Self::Ident(s.into())
    }

    fn number(s: impl Into<String>) -> Self {
        Self::Numeric(s.into())
    }

    fn into_item(self) -> Result<Item, String> {
        Ok(match self {
            Self::Ident(s) => Item::Lexical(Lexical::Ident(s)),
            Self::Numeric(n) => Item::Lexical(Lexical::Numeric({
                if let Some(hex) = n.strip_prefix("0x") {
                    u16::from_str_radix(hex, 16).map_err(|_| n)?
                } else {
                    n.parse().map_err(|_| n)?
                }
            })),
        })
    }

    fn push(&mut self, ch: char) {
        match self {
            Self::Ident(s) => s.push(ch),
            Self::Numeric(s) => s.push(ch),
        }
    }
}

pub fn tokenize(original: &str) -> Result<Vec<Spanned<Item>>, TokenizingError> {
    use TokenizingError::*;
    let mut output = Vec::new();

    let mut current_ident: Option<(usize, MultiCharItem)> = None;
    let mut go_to_next_linebreak = false;
    for (index, character) in original.chars().enumerate() {
        if go_to_next_linebreak {
            if character == '\n' {
                go_to_next_linebreak = false;
            }
            continue;
        }

        let single_character_range = (index, 1).into();
        if !character.is_ascii() {
            return Err(Unicode {
                offending_character: character,
                at: single_character_range,
            });
        }

        {
            let punctuation = match character {
                ':' => Some(Item::Punct(Punct::Colon)),
                ';' => Some(Item::Linebreak),
                ',' => Some(Item::Punct(Punct::Comma)),
                '$' => Some(Item::Punct(Punct::Dollar)),
                '=' => Some(Item::Punct(Punct::Equals)),
                '.' => Some(Item::Punct(Punct::Period)),
                _ => None,
            };

            if let Some(punctuation) = punctuation {
                if let Some((start, current)) = current_ident {
                    let range = (start, index - start).into();
                    output.push(Spanned::new(
                        current.into_item().map_err(|e| InvalidNumber {
                            offending_string: e,
                            at: range,
                        })?,
                        range,
                    ));
                    current_ident = None;
                }

                if punctuation == Item::Linebreak {
                    go_to_next_linebreak = true;
                }

                output.push(Spanned {
                    item: punctuation.clone(),
                    at: single_character_range,
                });

                if punctuation == Item::Punct(Punct::Colon) {
                    output.push(Spanned {
                        item: Item::Linebreak,
                        at: single_character_range,
                    });
                }
                continue;
            }
        }

        if character.is_whitespace() {
            if let Some((start, current)) = current_ident {
                let range = (start, index - start).into();
                output.push(Spanned::new(
                    current.into_item().map_err(|e| InvalidNumber {
                        offending_string: e,
                        at: range,
                    })?,
                    range,
                ));
                current_ident = None;
            }
            if character == '\n' {
                output.push(Spanned::new(Item::Linebreak, (index, 1).into()))
            }
            continue;
        }

        if character.is_ascii_alphabetic() || character == '_' {
            if let Some((_, ref mut current)) = current_ident {
                current.push(character);
            } else {
                current_ident = Some((
                    index,
                    MultiCharItem::string(character),
                ));
            }
            continue;
        }

        if character.is_ascii_digit() {
            if let Some((_, ref mut current)) = current_ident {
                current.push(character);
            } else {
                current_ident = Some((
                    index,
                    MultiCharItem::number(character),
                ));
            }
            continue;
        }

        return Err(UnrecognisedItem {
            offending_character: character,
            at: single_character_range,
        });
    }

    Ok(output)
}

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum TokenizingError {
    #[error("Unrecognised item '{}'", .offending_character)]
    #[diagnostic(code(c8common::asm::unrecognised_item))]
    UnrecognisedItem {
        offending_character: char,
        #[label("here")]
        at: SourceSpan,
    },
    #[error("Invalid item '{}'", .offending_string)]
    #[diagnostic(code(c8common::asm::invalid_item))]
    InvalidNumber {
        offending_string: String,
        #[label("here")]
        at: SourceSpan,
    },
    #[error("Non-ASCII Unicode is not supported")]
    #[diagnostic(code(c8common::asm::unicode))]
    Unicode {
        offending_character: char,
        #[label("here")]
        at: SourceSpan,
    },
}
