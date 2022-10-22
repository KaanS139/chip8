#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Datum(pub u8);

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Nibble(u8);

impl Datum {
    pub fn as_nibbles(&self) -> [Nibble; 2] {
        [Nibble(self.0 >> 4), Nibble(self.0 & 0b1111)]
    }
}

impl Nibble {
    pub fn as_half_byte(&self) -> u8 {
        self.0
    }

    pub fn byte_with(self, other: Self) -> u8 {
        self.0 << 4 | other.0
    }
}

impl BitOr<u8> for Datum {
    type Output = Self;

    fn bitor(self, rhs: u8) -> Self::Output {
        Self(self.0 | rhs)
    }
}

impl BitOrAssign<u8> for Datum {
    fn bitor_assign(&mut self, rhs: u8) {
        self.0 |= rhs;
    }
}

macro_rules! impl_fmt {
    (($ty:ty, $inner:ty), $tr:path) => {
        impl $tr for $ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                <$inner as $tr>::fmt(&self.0, f)
            }
        }
    };
    (($ty:ty, $inner:ty), $($tr:path),+) => {
        $(
            impl_fmt!(($ty, $inner), $tr);
        )+
    }
}

pub(crate) use impl_fmt;
use std::ops::{BitOr, BitOrAssign};

impl_fmt!(
    (Datum, u8),
    std::fmt::LowerHex,
    std::fmt::UpperHex,
    std::fmt::Octal,
    std::fmt::Binary
);
