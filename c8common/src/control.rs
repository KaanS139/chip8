use crate::control::execute::Interpreter;
use crate::key::Keys;
use crate::memory::Memory;
use crate::{Address, Datum, Display, GeneralRegister};

pub mod execute;

pub trait ControlledInterpreter {
    fn step(&mut self, keys: Keys, frame: &mut FrameInfo);

    fn display(&self) -> &Display;

    fn delay_timer_register(&self) -> &Datum;
    fn delay_timer_register_mut(&mut self) -> &mut Datum;

    fn sound_timer_register(&self) -> &Datum;
    fn sound_timer_register_mut(&mut self) -> &mut Datum;

    fn timer_tick_60hz(&mut self) -> TimerTick {
        let mut tick = TimerTick::new();
        tick.delay(self.delay_timer_register_mut().towards_zero());
        tick.sound(self.sound_timer_register_mut().towards_zero());
        tick
    }

    fn register(&self, register: GeneralRegister) -> &Datum;
    fn register_mut(&mut self, register: GeneralRegister) -> &mut Datum;

    fn set_register(&mut self, register: GeneralRegister, datum: Datum) {
        *self.register_mut(register) = datum;
    }

    fn get_register(&self, register: GeneralRegister) -> Datum {
        *self.register(register)
    }

    fn get_i(&self) -> u16;
    fn get_i_mut(&mut self) -> &mut u16;

    fn set_i(&mut self, to: u16) {
        *self.get_i_mut() = to;
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
    entered_busywait: bool,
    screen_modified: bool,
    buzzer_change_state: Option<bool>,
    wait_for_key: Option<GeneralRegister>,
}

impl FrameInfo {
    fn empty() -> Self {
        Self {
            entered_busywait: false,
            screen_modified: false,
            buzzer_change_state: None,
            wait_for_key: None,
        }
    }

    pub fn modify_screen(&mut self) {
        self.screen_modified = true;
    }

    fn set_buzzer(&mut self, to: bool) {
        self.buzzer_change_state = Some(to);
    }

    pub fn busywait(&mut self) {
        self.entered_busywait = true;
    }

    pub fn wait_for_key_on(&mut self, register: GeneralRegister) {
        self.wait_for_key = Some(register);
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

#[derive(Debug, Copy, Clone)]
pub struct TimerTick {
    delay: bool,
    sound: bool,
}

impl TimerTick {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            delay: false,
            sound: false,
        }
    }

    pub fn delay(&mut self, decremented: bool) {
        if decremented {
            self.delay = true;
        }
    }

    pub fn sound(&mut self, decremented: bool) {
        if decremented {
            self.sound = true;
        }
    }

    pub fn buzzer_active(&self) -> bool {
        self.sound
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(missing_copy_implementations)]
pub enum InterpreterState {
    Normal,
    Held,
    WaitForKey(GeneralRegister),
    BusyWaiting,
}
