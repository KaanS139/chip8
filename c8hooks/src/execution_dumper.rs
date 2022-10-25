use c8common::control::ControlledInterpreter;
use c8common::hooks::{HookedItem, InterpreterHook};
use c8common::key::Keys;
use log::info;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub struct ExecutionDumper {
    to: File,
}

impl<T: ControlledInterpreter> InterpreterHook<T> for ExecutionDumper {
    fn get_keys(&mut self, int: &T, keys: Keys) -> HookedItem<Keys> {
        info!("Keys were {:?}", keys);
        HookedItem::finish(Keys::from_number(0x00))
    }
}

impl ExecutionDumper {
    pub fn dump_to(to: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        Ok(Self {
            to: File::create(to)?,
        })
    }
}
