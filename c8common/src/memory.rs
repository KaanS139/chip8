use crate::asm::ROM;
use crate::{Address, Datum, NUMBER_OF_ADDRESSES};
use std::ops::Index;
use tap::TryConv;

#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct Memory([Datum; NUMBER_OF_ADDRESSES]);

impl Memory {
    pub fn from_rom(rom: ROM) -> Self {
        #[allow(clippy::identity_op)] // For clarity
        let mut internal_data = [Datum(0); 0x200 - 0x000];

        for (i, byte) in FONT_DATA.iter().enumerate() {
            internal_data[i + FONT_START_ADDR] = Datum(*byte);
        }

        let working_data = rom.into_data();
        let out_vec = internal_data
            .into_iter()
            .chain(working_data.into_iter())
            .collect::<Vec<_>>();
        let out_data = out_vec
            .try_conv::<[Datum; NUMBER_OF_ADDRESSES]>()
            .expect("ROM is constant size, extending with constant size!");

        Self(out_data)
    }

    pub fn substring(&self, start: Address, number: u8) -> &[Datum] {
        let start = start.as_u16() as usize;
        let end = start + number as usize;
        &self.0[start..end]
    }
}

impl From<ROM> for Memory {
    fn from(rom: ROM) -> Self {
        Self::from_rom(rom)
    }
}

impl Index<Address> for Memory {
    type Output = Datum;

    fn index(&self, index: Address) -> &Self::Output {
        &self.0[index.try_conv::<usize>().unwrap()]
    }
}

const FONT_START_ADDR: usize = 0x50;
const FONT_DATA: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
