use crate::memory::Memory;
use crate::{Address, Datum, NUMBER_OF_ADDRESSES};
use log::info;
use std::cmp::Ordering;
use std::ops::Index;
use std::path::Path;
use tap::prelude::*;

#[derive(Debug, Clone)]
#[allow(missing_copy_implementations)]
pub struct ROM([Datum; NUMBER_OF_ADDRESSES - Address::PROGRAM_START_INDEX]);

impl ROM {
    pub fn new() -> Self {
        Self([Datum(0); NUMBER_OF_ADDRESSES - Address::PROGRAM_START_INDEX])
    }

    pub fn containing(
        containing: [Datum; NUMBER_OF_ADDRESSES - Address::PROGRAM_START_INDEX],
    ) -> Self {
        Self(containing)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let buf = self.0.map(|datum| datum.0);
        std::fs::write(path, buf)
    }

    pub fn from_bytes(mut bytes: Vec<u8>) -> Result<Self, LoadError> {
        if bytes.len() < NUMBER_OF_ADDRESSES - Address::PROGRAM_START_INDEX {
            info!(
                "Padding bytes from {} to {}",
                bytes.len(),
                NUMBER_OF_ADDRESSES - Address::PROGRAM_START_INDEX
            );
            bytes.extend(
                std::iter::repeat(0)
                    .take(NUMBER_OF_ADDRESSES - Address::PROGRAM_START_INDEX - bytes.len()),
            )
        }
        match bytes
            .len()
            .cmp(&(NUMBER_OF_ADDRESSES - Address::PROGRAM_START_INDEX))
        {
            Ordering::Less => panic!("Should not be possible!"),
            Ordering::Equal => {
                let bytes = bytes
                    .try_conv::<[u8; NUMBER_OF_ADDRESSES - Address::PROGRAM_START_INDEX]>()
                    .unwrap();
                let data = bytes.map(Datum);
                Ok(Self(data))
            }
            Ordering::Greater => Err(LoadError::WrongSize {
                size: bytes.len(),
                expected: NUMBER_OF_ADDRESSES - Address::PROGRAM_START_INDEX,
            }),
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, FileLoadError> {
        let file_contents = std::fs::read(path).map_err(FileLoadError::IO)?;
        Self::from_bytes(file_contents).map_err(FileLoadError::LoadError)
    }

    pub(crate) fn into_data(self) -> [Datum; NUMBER_OF_ADDRESSES - 0x200] {
        self.0
    }

    pub fn to_memory(self) -> Memory {
        Memory::from_rom(self)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum LoadError {
    WrongSize { size: usize, expected: usize },
}

#[derive(Debug)]
pub enum FileLoadError {
    IO(std::io::Error),
    LoadError(LoadError),
}

impl Default for ROM {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<Address> for ROM {
    type Output = Datum;

    fn index(&self, index: Address) -> &Self::Output {
        &self.0[index.try_conv::<usize>().unwrap()]
    }
}

// impl IndexMut<Address> for ROM {
//     fn index_mut(&mut self, index: Address) -> &mut Self::Output {
//         &mut self.0[index.try_conv::<usize>().unwrap()]
//     }
// }
