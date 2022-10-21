#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Datum(pub u8);

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Nibble(u8);

impl Datum {
    pub fn as_nibbles(&self) -> (Nibble, Nibble) {
        (Nibble(self.0 >> 4), Nibble(self.0 & 0b1111))
    }
}

impl Nibble {
    pub fn nibble(&self) -> u8 {
        self.0
    }
}

macro_rules! impl_fmt {
    ($ty:ty, $tr:path) => {
        impl $tr for $ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                <u8 as $tr>::fmt(&self.0, f)
            }
        }
    };
    ($ty:ty, $($tr:path),+) => {
        $(
            impl_fmt!($ty, $tr);
        )+
    }
}

impl_fmt!(
    Datum,
    std::fmt::LowerHex,
    std::fmt::UpperHex,
    std::fmt::Octal,
    std::fmt::Binary
);
