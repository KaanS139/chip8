use crate::control::{ControlledInterpreter, FrameInfo, InterpreterState};
use crate::key::Keys;
use crate::memory::Memory;
use crate::{Datum, GeneralRegister, NUMBER_OF_ADDRESSES};
use std::fmt::Debug;

pub mod disabled;

#[allow(unused_variables)]
pub trait InterpreterHook<T: ControlledInterpreter>: Debug + Send {
    /// Called at the very start of each step
    /// Use this to setup/reset any frame-tracking logic
    fn pre_cycle(&mut self, state: &mut InterpreterState) {}
    /// Called when reading the keys from the windowing system
    fn get_keys(&mut self, state: InterpreterState, int: &T, keys: Keys) -> HookedItem<Keys> {
        HookedItem::ignore()
    }
    /// Called immediately before a step
    /// Has access to the `FrameInfo`
    fn before_step(&mut self, int: &mut T, frame: &mut FrameInfo) {}
    /// Called immediately after a step
    /// Has access to the `FrameInfo`
    fn after_step(&mut self, int: &mut T, frame: &mut FrameInfo) {}
    /// Called at the very end of each step
    /// Use this to analyse something over the whole frame or flush datastreams
    fn post_cycle(&mut self, state: &mut InterpreterState) {}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FurtherHooks {
    Continue,
    Stop,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct HookedItem<T> {
    pub(crate) item: Option<T>,
    pub(crate) behaviour: FurtherHooks,
}

impl<T> HookedItem<T> {
    pub fn passthrough(item: T) -> Self {
        Self {
            item: Some(item),
            behaviour: FurtherHooks::Continue,
        }
    }

    pub fn finish(item: T) -> Self {
        Self {
            item: Some(item),
            behaviour: FurtherHooks::Stop,
        }
    }

    pub fn ignore() -> Self {
        Self {
            item: None,
            behaviour: FurtherHooks::Continue,
        }
    }
}

pub trait HookInternalAccess<I> {
    fn is_entered_busywait(&self, frame: &FrameInfo) -> bool {
        frame.entered_busywait
    }

    fn is_modify_screen(&self, frame: &FrameInfo) -> bool {
        frame.screen_modified
    }

    fn is_buzzer_change_state(&self, frame: &FrameInfo) -> Option<bool> {
        frame.buzzer_change_state
    }

    fn is_wait_for_key(&self, frame: &FrameInfo) -> Option<GeneralRegister> {
        frame.wait_for_key
    }

    fn extract_memory(&self, memory: Memory) -> [Datum; NUMBER_OF_ADDRESSES] {
        memory.extract()
    }

    #[deprecated = "do not use"]
    fn dummy(&self, _: Option<&I>) {
        unimplemented!("This method should not be called!")
    }
}

impl<I: ControlledInterpreter, T: InterpreterHook<I>> HookInternalAccess<I> for T {}
