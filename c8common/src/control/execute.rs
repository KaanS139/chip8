use crate::control::{ControlledInterpreter, ControlledToInterpreter, FrameInfo};
use crate::key::Keys;
use chip8_base::{Display, Keys as RawKeys};
use log::{debug, trace};
use std::time::Duration;

#[derive(Debug)]
pub struct Interpreter<I: ControlledInterpreter> {
    inner: I,
    buzzer_active: bool,
    step_frequency: u32,
    internal_speed: Option<u32>,
}

impl<T: ControlledInterpreter> chip8_base::Interpreter for Interpreter<T> {
    fn step(&mut self, keys: &RawKeys) -> Option<Display> {
        trace!("Beginning step.");
        // TODO: Some registers are auto-decremented based on real time. Use the internal_speed to be able to vary the emulated speed;
        let mut frame_info = FrameInfo::empty();
        self.inner.step(Keys::from_raw(keys), &mut frame_info);
        trace!("Step complete!");

        let FrameInfo {
            screen_modified,
            buzzer_change_state,
        } = frame_info;
        if let Some(buzzer) = buzzer_change_state {
            self.buzzer_active = buzzer;
        }
        if screen_modified {
            debug!("Screen has been updated.");
            return Some(*self.inner.display().raw());
        }
        None
    }

    fn speed(&self) -> Duration {
        Duration::from_secs_f32(1. / (self.step_frequency as f32))
    }

    fn buzzer_active(&self) -> bool {
        self.buzzer_active
    }
}

impl<T: ControlledToInterpreter> Interpreter<T> {
    pub fn new(from: T) -> Self {
        Self {
            inner: from,
            buzzer_active: false,
            step_frequency: 8,
            internal_speed: None,
        }
    }
}
