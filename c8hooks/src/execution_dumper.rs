use c8common::control::{ControlledInterpreter, FrameInfo, InterpreterState};
use c8common::hooks::{HookInternalAccess, HookedItem, InterpreterHook};
use c8common::key::Keys;
use c8common::{Datum, NUMBER_OF_ADDRESSES};
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug)]
pub struct ExecutionDumper {
    to: File,
    step_number: u64,
    memory_copy: Option<[Datum; NUMBER_OF_ADDRESSES]>,
}

impl<T: ControlledInterpreter> InterpreterHook<T> for ExecutionDumper {
    fn pre_cycle(&mut self, state: &mut InterpreterState) {
        self.dump(format!("------ START OF STEP {} ------", self.step_number));
        self.dump(format!("Starting in state {:?}", state));
        self.memory_copy = None;
    }

    fn get_keys(&mut self, _: InterpreterState, _: &T, keys: Keys) -> HookedItem<Keys> {
        self.dump(format!("Keys are {:?}", keys));
        HookedItem::ignore()
    }

    fn before_step(&mut self, int: &mut T, _: &mut FrameInfo) {
        self.memory_copy = Some(<Self as HookInternalAccess<T>>::extract_memory(
            &*self,
            int.memory().clone(),
        ));
        self.dump_state("Before", int);
    }

    fn after_step(&mut self, int: &mut T, frame: &mut FrameInfo) {
        self.dump_state("After", int);
        let mut changed = false;
        for (addr, (after, before)) in
            <Self as HookInternalAccess<T>>::extract_memory(&*self, int.memory().clone())
                .iter()
                .zip(self.memory_copy.unwrap())
                .enumerate()
        {
            if *after != before {
                if !changed {
                    self.dump(";; Memory".to_string());
                }
                self.dump(format!(
                    "> Addr {:03X} used to be {}, now {}",
                    addr, before.0, after.0
                ));
                changed = true;
            }
        }
        self.dump(";; Status".to_string());
        if <Self as HookInternalAccess<T>>::is_modify_screen(&*self, &*frame) {
            self.dump("> Screen has been modified".to_string());
        }
        if let Some(mode) = <Self as HookInternalAccess<T>>::is_buzzer_change_state(&*self, &*frame)
        {
            self.dump(format!(
                "> Buzzer has been set to {}",
                if mode { "on" } else { "off" }
            ));
        }
        if <Self as HookInternalAccess<T>>::is_entered_busywait(&*self, &*frame) {
            self.dump("> Entered busywait".to_string());
        }
        if let Some(reg) = <Self as HookInternalAccess<T>>::is_wait_for_key(&*self, &*frame) {
            self.dump(format!("> Waiting to store next keypress in {:?}", reg));
        }
    }

    fn post_cycle(&mut self, state: &mut InterpreterState) {
        self.dump(format!("Ending in state {:?}", state));
        self.dump(format!("------- END OF STEP {} -------", self.step_number));
        self.step_number += 1;
    }
}

impl ExecutionDumper {
    pub fn dump_to(to: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        Ok(Self {
            to: File::create(to)?,
            step_number: 0,
            memory_copy: None,
        })
    }

    fn dump_state<T: ControlledInterpreter>(&mut self, prefix: &str, state: &mut T) {
        self.dump(format!("--- {}", prefix));
        self.dump(";; State".to_string());
        self.dump(format!(
            "> Program counter = {:03X}",
            state.program_counter().as_u16()
        ));
        self.dump("> Stack:".to_string());
        for (i, addr) in state.stack().iter().enumerate() {
            self.dump(format!(">> {}: {:03X}", i, addr.as_u16()));
        }
        self.dump(";; Registers".to_string());
        self.dump(format!("> {:?}", state.register_bank().map(|x| x.0)));
        self.dump(format!("> I={:04X}", state.get_i()));
        self.dump(format!(
            "> Delay timer = {}",
            state.delay_timer_register().0
        ));
        self.dump(format!(
            "> Sound timer = {}",
            state.sound_timer_register().0
        ));
    }

    fn dump(&mut self, to_dump: String) {
        writeln!(self.to, "{}", to_dump).unwrap();
    }
}
