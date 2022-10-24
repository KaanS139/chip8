use crate::control::{ControlledInterpreter, ControlledToInterpreter, FrameInfo, InterpreterState};
use crate::key::Keys;
use chip8_base::{Display, Keys as RawKeys};
use log::{debug, info, trace, warn};
use std::time::Duration;

#[derive(Debug)]
pub struct Interpreter<I: ControlledInterpreter> {
    inner: I,
    buzzer_active: bool,
    step_frequency: u32,
    internal_frequency_scale: Option<f32>,
    sixty_hertz_progress: Duration,
    state: InterpreterState,
}

impl<T: ControlledInterpreter> chip8_base::Interpreter for Interpreter<T> {
    fn step(&mut self, keys: &RawKeys) -> Option<Display> {
        let keys = Keys::from_raw(keys);
        match self.state {
            InterpreterState::Normal => {}
            InterpreterState::Held => {
                todo!("Check for resume")
            }
            InterpreterState::WaitForKey(reg) => {
                if keys.pressed() {
                    let parsed_keys = keys.one_key();
                    if let Some(key) = parsed_keys {
                        info!("Key pressed, continuing!");
                        self.state = InterpreterState::Normal;
                        self.inner.set_register(reg, key);
                    } else {
                        warn!("Multiple keys pressed at once, not continuing!");
                        return None;
                    }
                } else {
                    return None;
                }
            }
            InterpreterState::BusyWaiting => return None,
        }
        trace!("Beginning step.");
        let mut frame_info = FrameInfo::empty();

        let internal_frequency = self.internal_frequency_scale.unwrap_or(1.);
        let more_progress =
            Duration::from_secs_f32(self.speed().as_secs_f32() * internal_frequency);
        self.sixty_hertz_progress += more_progress;
        if self.sixty_hertz_progress.as_secs_f32() >= 1. / 60. {
            self.sixty_hertz_progress = Duration::ZERO;
            if self.inner.timer_tick_60hz().buzzer_active() {
                frame_info.set_buzzer(true);
            } else {
                frame_info.set_buzzer(false);
            }
        }
        self.inner.step(keys, &mut frame_info);
        trace!("Step complete!");

        let FrameInfo {
            screen_modified,
            buzzer_change_state,
            entered_busywait,
            wait_for_key,
        } = frame_info;

        if let Some(reg) = wait_for_key {
            self.state = InterpreterState::WaitForKey(reg);
            info!("Waiting to store next keypress in {:?}", reg);
        }

        if entered_busywait {
            self.state = InterpreterState::BusyWaiting;
        }

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
            internal_frequency_scale: None,
            sixty_hertz_progress: Duration::ZERO,
            state: InterpreterState::Normal,
        }
    }

    pub fn with_frequency(mut self, frequency: u32) -> Self {
        self.step_frequency = frequency;
        self
    }

    pub fn with_simulated_frequency(mut self, frequency_scale: Option<f32>) -> Self {
        self.internal_frequency_scale = frequency_scale;
        self
    }
}
