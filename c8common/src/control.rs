use crate::control::execute::Interpreter;
use crate::key::Keys;
use crate::memory::Memory;
use crate::{Address, Datum, Display, GeneralRegister};

pub mod execute;

pub trait ControlledInterpreter {
    fn step(&mut self, keys: Keys, frame: &mut FrameInfo);

    fn display(&self) -> &Display;

    fn register(&self, register: GeneralRegister) -> &Datum;
    fn register_mut(&mut self, register: GeneralRegister) -> &mut Datum;

    fn set_register(&mut self, register: GeneralRegister, datum: Datum) {
        *self.register_mut(register) = datum;
    }

    fn get_register(&self, register: GeneralRegister) -> Datum {
        *self.register(register)
    }

    fn stack(&self) -> &Vec<Address>;
    fn stack_mut(&mut self) -> &mut Vec<Address>;

    fn stack_push(&mut self, addr: Address) {
        if self.stack().len() >= 16 {
            panic!("Stack overflow!")
        }
        self.stack_mut().push(addr);
    }

    fn stack_pop(&mut self) -> Address {
        self.stack_mut().pop().expect("Stack underflow!")
    }

    fn memory(&self) -> &Memory;
    fn memory_mut(&mut self) -> &mut Memory;

    fn program_counter(&self) -> Address;
    fn program_counter_mut(&mut self) -> &mut Address;
    fn increment_program_counter(&mut self) {
        self.program_counter_mut().increment();
    }
    fn set_program_counter(&mut self, to: Address) {
        *self.program_counter_mut() = to;
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(missing_copy_implementations)]
pub struct FrameInfo {
    screen_modified: bool,
    buzzer_change_state: Option<bool>,
}

impl FrameInfo {
    fn empty() -> Self {
        Self {
            screen_modified: false,
            buzzer_change_state: None,
        }
    }

    pub fn modify_screen(&mut self) {
        self.screen_modified = true;
    }

    fn set_buzzer(&mut self, to: bool) {
        self.buzzer_change_state = Some(to);
    }
}

pub trait ControlledToInterpreter: ControlledInterpreter {
    fn to_interpreter(self) -> Interpreter<Self>
    where
        Self: Sized;
}

impl<T: ControlledInterpreter> ControlledToInterpreter for T {
    fn to_interpreter(self) -> Interpreter<Self>
    where
        Self: Sized,
    {
        Interpreter::new(self)
    }
}
