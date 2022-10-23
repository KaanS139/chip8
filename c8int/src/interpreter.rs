use crate::prelude::*;
use asm::ROM;
use c8common::control::{ControlledInterpreter, FrameInfo};
use c8common::key::Keys;
use log::{debug, error, info, warn};
use tap::prelude::*;

pub struct Chip8Interpreter {
    program_counter: Address,
    memory: Memory,
    display: Display,
    general_registers: [Datum; 16],
    register_i: u16,
    register_vf: Datum,
    stack: Vec<Address>,
}

impl ControlledInterpreter for Chip8Interpreter {
    fn step(&mut self, keys: Keys, frame: &mut FrameInfo) {
        let d1 = self.fetch();
        let d2 = self.fetch();
        let instruction = Self::decode((d1, d2)).expect("Instructions should be valid!");

        self.execute(instruction, keys, frame);
    }

    fn display(&self) -> &Display {
        &self.display
    }

    fn register(&self, register: GeneralRegister) -> &Datum {
        &self.general_registers[register.index()]
    }

    fn register_mut(&mut self, register: GeneralRegister) -> &mut Datum {
        &mut self.general_registers[register.index()]
    }

    fn stack(&self) -> &Vec<Address> {
        &self.stack
    }

    fn stack_mut(&mut self) -> &mut Vec<Address> {
        &mut self.stack
    }

    fn memory(&self) -> &Memory {
        &self.memory
    }

    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.memory
    }

    fn program_counter(&self) -> Address {
        self.program_counter
    }

    fn program_counter_mut(&mut self) -> &mut Address {
        &mut self.program_counter
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

    fn decode(data: (Datum, Datum)) -> Result<Instruction, RawInstruction> {
        debug!("Decoding 0x{:0X}{:0X}", data.0, data.1);
        let processing = Instruction::try_from_data(data.into());

        processing
            .tap_ok(|inst| debug!("Instruction is {:?}", inst))
            .map_err(|e| e.invalid_data().unwrap())
    }

    fn execute(&mut self, instruction: Instruction, keys: Keys, frame: &mut FrameInfo) {
        match instruction {
            Instruction::Nop => {
                info!("Nop")
            }
            Instruction::ClearScreen => {
                info!("Clear screen");
                self.display.clear();
                frame.modify_screen();
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
                frame.modify_screen()
            }
        }
    }

    fn empty() -> Self {
        Self {
            program_counter: Address::PROGRAM_START,
            memory: Memory::empty(),
            display: Display::blank(),
            general_registers: [Datum(0); 16],
            register_i: 0,
            register_vf: Datum(0),
            stack: Vec::with_capacity(16),
        }
    }

    pub fn new_assembled<F: FnOnce(&mut asm::Assembler) -> &mut asm::Assembler>(with: F) -> Self {
        let mut assembler = asm::Assembler::new();
        (with)(&mut assembler);
        let program = assembler.assemble();
        Self::new_from_rom(program)
    }

    pub fn new_from_file(path: impl AsRef<std::path::Path>) -> Self {
        let program = ROM::from_file(path).unwrap();
        Self::new_from_rom(program)
    }

    pub fn new_from_rom(rom: ROM) -> Self {
        Self {
            memory: rom.to_memory(),
            ..Self::empty()
        }
    }
}
