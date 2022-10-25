use crate::control::ControlledInterpreter;
use crate::key::Keys;
use std::fmt::Debug;

pub trait InterpreterHook<T: ControlledInterpreter>: Debug + Send {
    fn get_keys(&mut self, _int: &T, _keys: Keys) -> HookedItem<Keys> {
        HookedItem::ignore()
    }

    fn before_step(&mut self, _int: &mut T) {}
}

pub trait InterpreterHookBundle<T: ControlledInterpreter> {
    type Extended<N: InterpreterHook<T>>;

    fn extend_with<N: InterpreterHook<T>>(self, with: N) -> Self::Extended<N>;

    fn to_vec(self) -> Vec<Box<dyn InterpreterHook<T>>>;
}

impl<T: ControlledInterpreter> InterpreterHookBundle<T> for () {
    type Extended<N: InterpreterHook<T>> = (N,);

    fn extend_with<N: InterpreterHook<T>>(self, with: N) -> Self::Extended<N> {
        (with,)
    }

    fn to_vec(self) -> Vec<Box<dyn InterpreterHook<T>>> {
        vec![]
    }
}

impl<T: ControlledInterpreter, A: InterpreterHook<T> + 'static> InterpreterHookBundle<T> for (A,) {
    type Extended<N: InterpreterHook<T>> = (A, N);

    fn extend_with<N: InterpreterHook<T>>(self, with: N) -> Self::Extended<N> {
        (self.0, with)
    }

    fn to_vec(self) -> Vec<Box<dyn InterpreterHook<T>>> {
        vec![Box::new(self.0)]
    }
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
