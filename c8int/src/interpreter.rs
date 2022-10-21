use crate::prelude::*;
use asm::ROM;
use log::{debug, error, info, trace};
use std::time::Duration;
use tap::prelude::*;

pub struct Chip8Interpreter {
    program_counter: Address,
    memory: ROM,
    display: Display,
    general_registers: [Datum; 16],
    register_i: Datum,
    register_vf: Datum,
    stack_pointer: u8,
    stack: [Datum; 16],
    clock_frequency: f32,
}

impl Interpreter for Chip8Interpreter {
    fn step(&mut self, _keys: &Keys) -> Option<Display> {
        trace!("Beginning step.");
        let d1 = self.fetch();
        let d2 = self.fetch();
        let instruction = self
            .decode((d1, d2))
            .expect("Instructions should be valid!");

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
        let datum = self.memory[self.program_counter];
        self.program_counter += 1;
        if self.program_counter >= 4096 {
            self.program_counter = 0;
        }
        debug!("Fetched {:X} from program memory.", datum);
        datum
    }

    fn decode(&mut self, data: (Datum, Datum)) -> Result<Instruction, (Datum, Datum)> {
        debug!("Decoding 0x{:0X}{:0X}", data.0, data.1);
        let processing = Instruction::try_from_data(data);

        processing
            .tap_ok(|inst| debug!("Instruction is {:?}", inst))
            .map_err(|e| e.invalid_data().unwrap())
    }

    fn execute(&mut self, instruction: Instruction) -> Vec<Command> {
        match instruction {
            Instruction::Nop => {
                info!("Nop")
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
                    error!("Invalid jump address! 0x{:X} is out of bounds!", addr);
                    panic!()
                }
                self.program_counter = addr;
            }
        }

        vec![]
    }

    pub fn new() -> Self {
        let program = asm::Assembler::new()
            .label_str("start")
            .nop()
            .nop()
            .label_str("begin_loop")
            .cls()
            .nop()
            .jump("begin_loop")
            .assemble();

        program.save("roms/test.ch8");

        // let program = c8asm_proc::c8asm!(
        //     start: nop
        //         nop,
        //     begin_loop:
        //         cls,
        //         nop,
        //         jp @begin_loop
        // );

        Self {
            program_counter: 0,
            memory: program,
            display: [[Pixel::Black; 64]; 32],
            general_registers: [Datum(0); 16],
            register_i: Datum(0),
            register_vf: Datum(0),
            stack_pointer: 0,
            stack: [Datum(0); 16],
            clock_frequency: 10.,
        }
    }
}

impl Default for Chip8Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

enum Command {
    ClearScreen,
}
