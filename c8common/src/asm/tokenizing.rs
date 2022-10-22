use std::cmp::Ordering;
use std::ops::RangeInclusive;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Item {
    Ident(String),
    Numeric(i64),
    Comma,
    Colon,
    Semicolon,
    Hash,
    OpenBrace,
    CloseBrace,
}

impl Item {
    pub(crate) fn ident_name(&self) -> Option<&String> {
        match self {
            Self::Ident(s) => Some(s),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SpannedItem {
    pub(crate) item: Item,
    pub(crate) at: RangeInclusive<Location>,
}

impl SpannedItem {
    fn new(item: Item, at: RangeInclusive<Location>) -> Self {
        Self { item, at }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Location {
    line: usize,
    column: usize,
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Location {
    fn cmp(&self, other: &Self) -> Ordering {
        let s = (self.line, self.column);
        let o = (other.line, other.column);
        s.cmp(&o)
    }
}

impl Location {
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
            Self::Ident(s) => Item::Ident(s),
            Self::Numeric(n) => Item::Numeric({
                if let Some(hex) = n.strip_prefix("0x") {
                    i64::from_str_radix(hex, 16).map_err(|_| n)?
                } else {
                    n.parse().map_err(|_| n)?
                }
            }),
        })
    }

    fn push(&mut self, ch: char) {
        match self {
            Self::Ident(s) => s.push(ch),
            Self::Numeric(s) => s.push(ch),
        }
    }
}

pub fn tokenize(original: &str) -> Result<Vec<SpannedItem>, TokenizingError> {
    use TokenizingError::*;
    let mut output = Vec::new();

    let mut current_line: usize = 0;
    let mut old_line_end: usize = 0;
    let mut current_ident: Option<(Location, MultiCharItem)> = None;
    for (index, character) in original.chars().enumerate() {
        let current_start_loc = Location {
            line: current_line,
            column: index.checked_sub(old_line_end).unwrap(),
        };
        let current_end_loc = Location {
            line: current_line,
            column: (index + 1).checked_sub(old_line_end).unwrap(),
        };
        let single_character_range = current_start_loc..=current_end_loc;
        if !character.is_ascii() {
            return Err(Unicode {
                offending_character: character,
                at: single_character_range,
            });
        }

        {
            let punctuation = match character {
                ':' => Some(Item::Colon),
                ';' => Some(Item::Semicolon),
                '#' => Some(Item::Hash),
                '[' => Some(Item::OpenBrace),
                ']' => Some(Item::CloseBrace),
                _ => None,
            };

            if let Some(punctuation) = punctuation {
                if let Some((start, current)) = current_ident {
                    let range = start..=current_end_loc;
                    output.push(SpannedItem::new(
                        current.into_item().map_err(|e| InvalidItem {
                            offending_string: e,
                            at: range,
                        })?,
                        start..=current_end_loc,
                    ));
                    current_ident = None;
                }

                output.push(SpannedItem {
                    item: punctuation,
                    at: single_character_range,
                });
                continue;
            }
        }

        if character.is_whitespace() {
            if let Some((start, current)) = current_ident {
                let range = start..=current_end_loc;
                output.push(SpannedItem::new(
                    current.into_item().map_err(|e| InvalidItem {
                        offending_string: e,
                        at: range,
                    })?,
                    start..=current_end_loc,
                ));
                current_ident = None;
            }
            if character == '\n' {
                current_line += 1;
                old_line_end = index;
            }
            continue;
        }

        if character.is_ascii_alphabetic() || character == '_' {
            if let Some((_, ref mut current)) = current_ident {
                current.push(character);
            } else {
                current_ident = Some((
                    Location {
                        line: current_line,
                        column: index.checked_sub(old_line_end).unwrap(),
                    },
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
                    Location {
                        line: current_line,
                        column: index.checked_sub(old_line_end).unwrap(),
                    },
                    MultiCharItem::number(character),
                ));
            }
            continue;
        }

        if character == '-' && current_ident.is_none() {
            current_ident = Some((
                Location {
                    line: current_line,
                    column: index.checked_sub(old_line_end).unwrap(),
                },
                MultiCharItem::number(character),
            ));
        }

        return Err(UnrecognisedItem {
            offending_character: character,
            at: single_character_range,
        });
    }

    Ok(output)
}

#[derive(Debug)]
pub enum TokenizingError {
    UnrecognisedItem {
        offending_character: char,
        at: RangeInclusive<Location>,
    },
    InvalidItem {
        offending_string: String,
        at: RangeInclusive<Location>,
    },
    Unicode {
        offending_character: char,
        at: RangeInclusive<Location>,
    },
}