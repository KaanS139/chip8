#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Datum(pub u8);

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Nibble(u8);

impl Datum {
    pub fn as_nibbles(self) -> [Nibble; 2] {
        [
            Nibble::new_from_half_byte(self.0 >> 4),
            Nibble::new_from_half_byte(self.0 & 0b1111),
        ]
    }

    pub fn towards_zero(&mut self) -> bool {
        if self.0 > 0 {
            self.0 -= 1;
            true
        } else {
            false
        }
    }

    pub fn inner(self) -> u8 {
        self.0
    }
}

impl Nibble {
    pub fn new_from_half_byte(byte: u8) -> Self {
        if byte & 0xF0 != 0 {
            panic!("Invalid value for nibble {}", byte);
        }
        Self(byte)
    }

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

impl BitOr<Datum> for Datum {
    type Output = Self;

    fn bitor(self, rhs: Datum) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitAnd<Datum> for Datum {
    type Output = Self;

    fn bitand(self, rhs: Datum) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitXor<Datum> for Datum {
    type Output = Self;

    fn bitxor(self, rhs: Datum) -> Self::Output {
        Self(self.0 ^ rhs.0)
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
use std::ops::{BitAnd, BitOr, BitOrAssign, BitXor};

impl_fmt!(
    (Datum, u8),
    std::fmt::LowerHex,
    std::fmt::UpperHex,
    std::fmt::Octal,
    std::fmt::Binary
);
