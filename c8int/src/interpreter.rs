use crate::prelude::*;
use asm::ROM;
use log::{debug, error, info, trace, warn};
use std::time::Duration;
use tap::prelude::*;

pub struct Chip8Interpreter {
    program_counter: Address,
    memory: Memory,
    display: Display,
    general_registers: [Datum; 16],
    register_i: u16,
    register_vf: Datum,
    stack_pointer: u8,
    stack: Vec<Address>,
    clock_frequency: f32,
}

impl Interpreter for Chip8Interpreter {
    fn step(&mut self, _keys: &Keys) -> Option<chip8_base::Display> {
        trace!("Beginning step.");
        let d1 = self.fetch();
        let d2 = self.fetch();
        let instruction = self
            .decode((d1, d2))
            .expect("Instructions should be valid!");

        let screen_state = self.execute(instruction);

        trace!("Step complete!");

        if screen_state == ScreenState::Modified {
            debug!("Screen has been updated.");
            return Some(*self.display.raw());
        }
        None
    }

    fn speed(&self) -> Duration {
        Duration::from_secs_f32(1. / self.clock_frequency)
    }

    fn buzzer_active(&self) -> bool {
        false
    }
}

impl Chip8Interpreter {
    fn fetch(&mut self) -> Datum {
        let datum = self.memory[self.program_counter];
        debug!(
            "Fetched {:X} from program memory address {:X}.",
            datum, self.program_counter
        );
        self.program_counter.increment();
        if self.program_counter >= 4096 {
            warn!("Program counter overflow!");
            self.program_counter = Address::ZERO;
        }
        datum
    }

    fn decode(&mut self, data: (Datum, Datum)) -> Result<Instruction, RawInstruction> {
        debug!("Decoding 0x{:0X}{:0X}", data.0, data.1);
        let processing = Instruction::try_from_data(data.into());

        processing
            .tap_ok(|inst| debug!("Instruction is {:?}", inst))
            .map_err(|e| e.invalid_data().unwrap())
    }

    fn execute(&mut self, instruction: Instruction) -> ScreenState {
        match instruction {
            Instruction::Nop => {
                info!("Nop")
            }
            Instruction::ClearScreen => {
                info!("Clear screen");
                self.display.clear();
                return ScreenState::Modified;
            }
            Instruction::Jump(addr) => {
                info!("Jump {:X}", addr);
                if addr & 0xF000 != 0 {
                    error!("Invalid jump address! 0x{:X} is out of bounds!", addr);
                    panic!()
                }
                self.program_counter = addr;
            }
            Instruction::Call(subroutine) => {
                info!("Call {:X}", subroutine);
                self.stack_push(self.program_counter);
                self.program_counter = subroutine;
            }

            Instruction::LoadRegByte(reg, byte) => {
                info!("Load immediate {:X} into {:?}", byte, reg);
                self.set_register(reg, Datum(byte));
            }

            Instruction::LoadImmediate(value) => {
                info!("Load immediate {:X} into I", value);
                self.register_i = value.as_u16();
            }

            Instruction::DisplaySprite {
                x: vx,
                y: vy,
                number_of_bytes,
            } => {
                info!(
                    "Display sprite; RX={:?} RY={:?} bytes={}",
                    vx,
                    vy,
                    number_of_bytes.as_half_byte()
                );
                let addr = Address::new(self.register_i);
                let x_coord = self.get_register(vx);
                let y_coord = self.get_register(vy);
                debug!("sprite={:X} x={} y={}", addr, x_coord.0, y_coord.0);
                self.display.sprite(
                    x_coord,
                    y_coord,
                    self.memory.substring(addr, number_of_bytes.as_half_byte()),
                );
                return ScreenState::Modified;
            }
        }

        ScreenState::Unchanged
    }

    fn set_register(&mut self, register: GeneralRegister, datum: Datum) {
        self.general_registers[register.index()] = datum;
    }

    fn get_register(&self, register: GeneralRegister) -> Datum {
        self.general_registers[register.index()]
    }

    fn stack_push(&mut self, addr: Address) {
        if self.stack.len() >= 16 {
            panic!("Stack overflow!")
        }
        self.stack.push(addr);
    }

    fn stack_pop(&mut self) -> Address {
        self.stack.pop().expect("Stack underflow!")
    }

    pub fn new() -> Self {
        // let program = asm::Assembler::new()
        //     .label_str("start")
        //     .nop()
        //     .nop()
        //     .label_str("begin_loop")
        //     .cls()
        //     .nop()
        //     .jump("begin_loop")
        //     .assemble();
        //
        // program.save("roms/test.ch8");

        let program = ROM::from_file("roms/UWCS.ch8").unwrap();

        // let program = c8asm_proc::c8asm!(
        //     start: nop
        //         nop,
        //     begin_loop:
        //         cls,
        //         nop,
        //         jp @begin_loop
        // );

        Self {
            program_counter: Address::new(0x200),
            memory: program.to_memory(),
            display: Display::blank(),
            general_registers: [Datum(0); 16],
            register_i: 0,
            register_vf: Datum(0),
            stack_pointer: 0,
            stack: Vec::with_capacity(16),
            clock_frequency: 5.,
        }
    }
}

impl Default for Chip8Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ScreenState {
    Unchanged,
    Modified,
}
