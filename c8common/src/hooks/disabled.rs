use crate::control::{ControlledInterpreter, FrameInfo, InterpreterState};
use crate::hooks::{HookedItem, InterpreterHook};
use crate::key::Keys;

#[derive(Debug)]
pub struct EnabledHook<I> {
    inner: I,
    enabled: bool,
}

impl<I> EnabledHook<I> {
    fn inner(&mut self) -> Option<&mut I> {
        if self.enabled {
            Some(&mut self.inner)
        } else {
            None
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

impl<T: ControlledInterpreter, I: InterpreterHook<T>> InterpreterHook<T> for EnabledHook<I> {
    fn get_keys(&mut self, state: InterpreterState, int: &T, keys: Keys) -> HookedItem<Keys> {
        self.inner()
            .map(|i| i.get_keys(state, int, keys))
            .unwrap_or_else(HookedItem::ignore)
    }

    fn pre_cycle(&mut self, state: &mut InterpreterState) {
        if let Some(i) = self.inner() {
            i.pre_cycle(state)
        }
    }

    fn before_step(&mut self, int: &mut T, frame: &mut FrameInfo) {
        if let Some(i) = self.inner() {
            i.before_step(int, frame)
        }
    }

    fn after_step(&mut self, int: &mut T, frame: &mut FrameInfo) {
        if let Some(i) = self.inner() {
            i.after_step(int, frame)
        }
    }

    fn post_cycle(&mut self, state: &mut InterpreterState) {
        if let Some(i) = self.inner() {
            i.pre_cycle(state)
        }
    }
}
