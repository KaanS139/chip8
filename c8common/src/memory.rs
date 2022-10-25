use crate::asm::{FileLoadError, LoadError, ROM};
use crate::{Address, Datum, NUMBER_OF_ADDRESSES};
use std::io::Write;
use std::ops::{Index, IndexMut};
use std::path::Path;
use tap::TryConv;

#[derive(Debug, Clone)]
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

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, FileLoadError> {
        let file_contents = std::fs::read(path).map_err(FileLoadError::IO)?;
        Self::from_bytes(file_contents).map_err(FileLoadError::LoadError)
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, LoadError> {
        if bytes.len() == NUMBER_OF_ADDRESSES {
            let bytes = bytes.try_conv::<[u8; NUMBER_OF_ADDRESSES]>().unwrap();
            let data = bytes.map(Datum);
            Ok(Self(data))
        } else {
            Err(LoadError::WrongSize {
                size: bytes.len(),
                expected: NUMBER_OF_ADDRESSES,
            })
        }
    }

    pub fn empty() -> Self {
        let mut inner = [Datum(0); NUMBER_OF_ADDRESSES];
        // Add an illegal instruction at the entrypoint
        inner[Address::PROGRAM_START.as_u16() as usize] = Datum(0x00);
        inner[Address::PROGRAM_START.as_u16() as usize + 1] = Datum(0xF0);
        Self(inner)
    }

    pub fn substring(&self, start: Address, number: u8) -> &[Datum] {
        let start = start.as_u16() as usize;
        let end = start + number as usize;
        &self.0[start..end]
    }

    pub fn all(&self) -> &[Datum] {
        &self.0[..]
    }

    pub(crate) fn extract(self) -> [Datum; NUMBER_OF_ADDRESSES] {
        self.0
    }

    pub fn save(&self, mut writer: impl Write) {
        writer.write_all(&self.0.map(Datum::inner)[..]).unwrap();
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

impl IndexMut<Address> for Memory {
    fn index_mut(&mut self, index: Address) -> &mut Self::Output {
        &mut self.0[index.try_conv::<usize>().unwrap()]
    }
}

pub const FONT_START_ADDR: usize = 0x50;
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
