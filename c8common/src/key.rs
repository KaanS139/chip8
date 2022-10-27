use crate::Datum;
use std::ops::{BitAnd, BitOrAssign};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Keys(u16);

#[allow(dead_code)]
impl Keys {
    const KEY_0: Self = nth_shift(0);
    const KEY_1: Self = nth_shift(1);
    const KEY_2: Self = nth_shift(2);
    const KEY_3: Self = nth_shift(3);
    const KEY_4: Self = nth_shift(4);
    const KEY_5: Self = nth_shift(5);
    const KEY_6: Self = nth_shift(6);
    const KEY_7: Self = nth_shift(7);
    const KEY_8: Self = nth_shift(8);
    const KEY_9: Self = nth_shift(9);
    const KEY_A: Self = nth_shift(10);
    const KEY_B: Self = nth_shift(11);
    const KEY_C: Self = nth_shift(12);
    const KEY_D: Self = nth_shift(13);
    const KEY_E: Self = nth_shift(14);
    const KEY_F: Self = nth_shift(15);
}

impl Keys {
    pub fn from_raw(raw: [bool; 16]) -> Self {
        let mut s = Self(0);
        for (i, &item) in raw.iter().enumerate() {
            if item {
                s |= nth_shift(i);
            }
        }
        s
    }

    pub fn from_number(value: u8) -> Self {
        nth_shift(value as usize)
    }

    pub fn from_datum(datum: Datum) -> Self {
        Self::from_number(datum.0)
    }

    pub fn pressed(&self) -> bool {
        self.0 != 0
    }

    pub fn one_key(&self) -> Option<Datum> {
        for i in 0..16 {
            if *self == nth_shift(i) {
                return Some(Datum(i as u8));
            }
        }
        None
    }
}

impl BitOrAssign for Keys {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for Keys {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

const fn nth_shift(n: usize) -> Keys {
    Keys(0b1 << n)
}
