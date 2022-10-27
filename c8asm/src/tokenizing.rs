use miette::{Diagnostic, SourceSpan};
use std::cmp::Ordering;
use thiserror::Error;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Item {
    Lexical(Lexical),
    Punct(Punct),
    Linebreak,
}

impl Item {
    pub fn as_lexical(&self) -> Option<&Lexical> {
        match self {
            Self::Lexical(l) => Some(l),
            _ => None,
        }
    }

    pub fn to_lexical(self) -> Option<Lexical> {
        match self {
            Self::Lexical(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_punct(&self) -> Option<Punct> {
        match self {
            Self::Punct(p) => Some(*p),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Lexical {
    PrefixedIdent(Punct, String),
    Ident(String),
    Numeric(u16),
}

impl Lexical {
    pub fn as_ident(&self) -> Option<&String> {
        match self {
            Self::Ident(s) => Some(s),
            _ => None,
        }
    }

    pub fn to_ident(self) -> Option<String> {
        match self {
            Self::Ident(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_prefixed(&self) -> Option<(Punct, &String)> {
        match self {
            Self::PrefixedIdent(p, s) => Some((*p, s)),
            _ => None,
        }
    }

    pub fn to_prefixed(self) -> Option<(Punct, String)> {
        match self {
            Self::PrefixedIdent(p, s) => Some((p, s)),
            _ => None,
        }
    }

    pub fn as_numeric(&self) -> Option<u16> {
        match self {
            Self::Numeric(p) => Some(*p),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Punct {
    Comma,
    Period,
    Colon,
    Dollar,
    Equals,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Spanned<T> {
    pub(crate) item: T,
    pub(crate) at: SourceSpan,
}

impl<T> Spanned<T> {
    fn new(item: T, at: SourceSpan) -> Self {
        Self { item, at }
    }
}

impl<T> From<Spanned<T>> for SourceSpan {
    fn from(spanned: Spanned<T>) -> Self {
        spanned.at
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
        Self { byte: at }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum MultiCharItem {
    PrefixedIdent(Punct, Option<String>),
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
            Self::PrefixedIdent(prefix, s) => match s {
                Some(s) => Item::Lexical(Lexical::PrefixedIdent(prefix, s)),
                None => Item::Punct(prefix),
            },
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
            Self::PrefixedIdent(_, s) => s.get_or_insert_with(String::new).push(ch),
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
                '=' => Some(Item::Punct(Punct::Equals)),
                '$' => Some(Item::Punct(Punct::Dollar)),
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

                if punctuation == Item::Punct(Punct::Dollar)
                    || punctuation == Item::Punct(Punct::Period)
                {
                    current_ident = Some((
                        index,
                        MultiCharItem::PrefixedIdent(punctuation.as_punct().unwrap(), None),
                    ))
                } else {
                    output.push(Spanned {
                        item: punctuation.clone(),
                        at: single_character_range,
                    });
                }

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
                current_ident = Some((index, MultiCharItem::string(character)));
            }
            continue;
        }

        if character.is_ascii_digit() {
            if let Some((_, ref mut current)) = current_ident {
                current.push(character);
            } else {
                current_ident = Some((index, MultiCharItem::number(character)));
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
