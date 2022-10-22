#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Address(u16);

impl Address {
    pub const MAX: Self = Self(u16::MAX >> 4);
    pub const NUMBER_OF_ADDRESSES: usize = (Self::MAX.0 + 1) as usize;
    pub const ZERO: Self = Self(0);

    pub fn new(at: u16) -> Self {
        assert!(at <= Self::MAX.0);
        Self(at)
    }

    pub fn from_triplet(high: u8, mid: u8, low: u8) -> Self {
        let inner = ((high as u16) << 8) | ((mid as u16) << 4) | low as u16;
        Self(inner)
    }

    pub fn increment(&mut self) {
        if *self == Self::MAX {
            panic!("Address overflow!");
        }
        self.0 += 1;
    }

    pub fn to_bytes(self) -> [u8; 2] {
        [(self.0 >> 8) as u8, (self.0 & 0xFF) as u8]
    }

    pub fn as_u16(self) -> u16 {
        self.0
    }
}

impl PartialEq<u16> for Address {
    fn eq(&self, other: &u16) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<u16> for Address {
    fn partial_cmp(&self, other: &u16) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl From<Address> for usize {
    fn from(value: Address) -> Self {
        value.0.into()
    }
}

impl Shr<usize> for Address {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        Self(self.0 >> rhs)
    }
}

impl Shr<usize> for &Address {
    type Output = Address;

    fn shr(self, rhs: usize) -> Self::Output {
        Address(self.0 >> rhs)
    }
}

impl BitAnd<u16> for Address {
    type Output = Self;

    fn bitand(self, rhs: u16) -> Self::Output {
        Self(self.0 & rhs)
    }
}

impl BitAnd<u16> for &Address {
    type Output = Address;

    fn bitand(self, rhs: u16) -> Self::Output {
        Address(self.0 & rhs)
    }
}

use crate::data::impl_fmt;
use std::cmp::Ordering;
use std::ops::{BitAnd, Shr};
impl_fmt! {
    (Address, u16),
    std::fmt::LowerHex,
    std::fmt::UpperHex,
    std::fmt::Octal,
    std::fmt::Binary
}
