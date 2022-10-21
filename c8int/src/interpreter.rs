use crate::prelude::*;
use log::{debug, error, info, trace};
use std::time::Duration;

pub struct Chip8Interpreter {
    program_counter: Address,
    memory: [Datum; (Address::MAX >> 4) as usize],
    display: Display,
    general_registers: [Datum; 16],
    register_i: Datum,
    register_vf: Datum,
    stack_pointer: u8,
    stack: [Datum; 16],
    clock_frequency: f32,
}

impl Interpreter for Chip8Interpreter {
    fn step(&mut self, keys: &Keys) -> Option<Display> {
        trace!("Beginning step.");
        let datum = self.fetch();
        let instruction = self.decode(datum).expect("Instructions should be valid!");

        let commands = self.execute(instruction);

        let mut screen_changed = false;
        for command in commands {
            match command {
                Command::ClearScreen => {
                    self.display = [[Pixel::Black; 64]; 32];
                    screen_changed = true;
                }
            }
        }

        trace!("Step complete!");

        if screen_changed {
            debug!("Screen has been updated.");
            return Some(self.display);
        }
        None
    }

    fn speed(&self) -> Duration {
        Duration::from_secs_f32(10. / self.clock_frequency)
    }

    fn buzzer_active(&self) -> bool {
        false
    }
}

impl Chip8Interpreter {
    fn fetch(&mut self) -> Datum {
        let datum = self.memory[self.program_counter as usize];
        self.program_counter += 1;
        if self.program_counter >= 4096 {
            self.program_counter = 0;
        }
        debug!("Fetched {:X} from program memory.", datum);
        datum
    }

    fn decode(&mut self, datum: Datum) -> Result<Instruction, Vec<Datum>> {
        debug!("Decoding {:X}", datum);
        let mut processing = Instruction::try_from_datum(datum);
        while let Err(e) = processing {
            match e {
                InstructionDecodeError::IncompleteInstruction(bytes) => {
                    processing = Instruction::try_from_data(bytes, self.fetch())
                }
                InstructionDecodeError::InvalidInstruction(_) => {
                    return Err(e.invalid_data().unwrap())
                }
            }
        }
        let instruction = processing.unwrap();
        debug!("Instruction is {:?}", instruction);
        Ok(instruction)
    }

    fn execute(&mut self, instruction: Instruction) -> Vec<Command> {
        match instruction {
            Instruction::Nop => {
                info!("NOP")
            }
            Instruction::Screen(screen_instruction) => match screen_instruction {
                ScreenInstruction::Clear => {
                    info!("Clear screen");
                    return vec![Command::ClearScreen];
                }
            },
            Instruction::Jump(addr) => {
                info!("Jump {:X}", addr);
                if addr & 0xF000 != 0 {
                    error!("Invalid jump address!");
                    panic!()
                }
                self.program_counter = addr;
            }
        }

        vec![]
    }

    pub fn new() -> Self {
        let mut s = Self {
            program_counter: 0,
            memory: [Datum(0); (Address::MAX >> 4) as usize],
            display: [[Pixel::Black; 64]; 32],
            general_registers: [Datum(0); 16],
            register_i: Datum(0),
            register_vf: Datum(0),
            stack_pointer: 0,
            stack: [Datum(0); 16],
            clock_frequency: 10.,
        };

        s.memory[0] = Datum(0x00);
        s.memory[1] = Datum(0xE0);

        s.memory[4] = Datum(0x10);
        s.memory[5] = Datum(0x00);

        s
    }
}

enum Command {
    ClearScreen,
}
