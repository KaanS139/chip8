use crate::prelude::*;
use asm::ROM;
use c8common::control::{ControlledInterpreter, FrameInfo};
use c8common::display::ScreenModification;
use c8common::key::Keys;
use c8common::memory::FONT_START_ADDR;
use log::{debug, error, info, warn};
use rand::rngs::OsRng;
use rand::Rng;
use tap::prelude::*;

#[derive(Debug)]
pub struct Chip8Interpreter {
    program_counter: Address,
    memory: Memory,
    display: Display,
    general_registers: [Datum; 16],
    register_i: u16,
    stack: Vec<Address>,

    delay_timer: Datum,
    sound_timer: Datum,

    rng: OsRng,
}

impl ControlledInterpreter for Chip8Interpreter {
    fn step(&mut self, keys: Keys, frame: &mut FrameInfo) {
        // let orig_pc = self.program_counter;
        // assert_eq!(orig_pc.as_u16() % 2, 0);
        let d1 = self.fetch();
        let d2 = self.fetch();
        let instruction = Self::decode((d1, d2)).expect("Instructions should be valid!");

        // println!("[Addr> {:04X}] (Op> {:02X}{:02X}) {:?}", orig_pc, d1, d2, instruction);
        // println!("Executing opcode=0x{:02X}{:02X} (pc=0x{:04X})", d1, d2, orig_pc.as_u16());

        self.execute(instruction, keys, frame);
    }

    fn display(&self) -> &Display {
        &self.display
    }

    fn delay_timer_register(&self) -> &Datum {
        &self.delay_timer
    }

    fn delay_timer_register_mut(&mut self) -> &mut Datum {
        &mut self.delay_timer
    }

    fn sound_timer_register(&self) -> &Datum {
        &self.sound_timer
    }

    fn sound_timer_register_mut(&mut self) -> &mut Datum {
        &mut self.sound_timer
    }

    fn register(&self, register: GeneralRegister) -> &Datum {
        &self.general_registers[register.index()]
    }

    fn register_mut(&mut self, register: GeneralRegister) -> &mut Datum {
        &mut self.general_registers[register.index()]
    }

    fn get_i(&self) -> u16 {
        self.register_i
    }

    fn get_i_mut(&mut self) -> &mut u16 {
        &mut self.register_i
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
        debug!("Decoding 0x{:02X}{:02X}", data.0, data.1);
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
            Instruction::Return => {
                let pc = self.stack_pop();
                info!("Return to {:02X}", pc);
                self.program_counter = pc;
            }
            Instruction::Jump(addr) => {
                info!("Jump {:X}", addr);
                if addr & 0xF000 != 0 {
                    error!("Invalid jump address! 0x{:X} is out of bounds!", addr);
                    panic!()
                }
                if addr.as_u16() + 2 == self.program_counter.as_u16() {
                    warn!("Entering busywait loop, stopping.");
                    info!("Loop at 0x{:02X}", self.program_counter);
                    frame.busywait();
                }
                self.program_counter = addr;
            }
            Instruction::Call(subroutine) => {
                info!("Call {:X}", subroutine);
                self.stack_push(self.program_counter);
                self.program_counter = subroutine;
            }
            Instruction::SkipIfEqual(reg, byte) => {
                let contents = self.get_register(reg);
                if contents.0 == byte {
                    info!("Skipping next instruction! (EQ)");
                    self.increment_program_counter();
                    self.increment_program_counter();
                } else {
                    info!("Not skipping next instruction! (NE)");
                }
            }
            Instruction::SkipNotEqual(reg, byte) => {
                let contents = self.get_register(reg);
                if contents.0 != byte {
                    info!("Skipping next instruction! (NE)");
                    self.increment_program_counter();
                    self.increment_program_counter();
                } else {
                    info!("Not skipping next instruction! (EQ)");
                }
            }
            Instruction::SkipRegistersEqual(r1, r2) => {
                let c1 = self.get_register(r1);
                let c2 = self.get_register(r2);
                if c1 == c2 {
                    info!("Skipping next instruction! (EQ)");
                    self.increment_program_counter();
                    self.increment_program_counter();
                } else {
                    info!("Not skipping next instruction! (NE)");
                }
            }
            Instruction::LoadRegByte(reg, byte) => {
                info!("Load immediate {:02X} into {:?}", byte, reg);
                self.set_register(reg, Datum(byte));
            }
            Instruction::Add(reg, byte) => {
                info!("Add {} to {:?}", byte, reg);
                self.set_register(reg, Datum(self.register(reg).0.overflowing_add(byte).0));
            }
            Instruction::CopyRegToReg { x: rx, y: ry } => {
                info!("Copy from {:?} to {:?}", ry, rx);
                self.set_register(rx, self.get_register(ry));
            }
            Instruction::Or { x: rx, y: ry } => {
                info!("OR {:?}, {:?}", rx, ry);
                self.set_register(rx, self.get_register(rx) | self.get_register(ry));
            }
            Instruction::And { x: rx, y: ry } => {
                info!("AND {:?}, {:?}", rx, ry);
                self.set_register(rx, self.get_register(rx) & self.get_register(ry));
            }
            Instruction::Xor { x: rx, y: ry } => {
                info!("XOR {:?}, {:?}", rx, ry);
                self.set_register(rx, self.get_register(rx) ^ self.get_register(ry));
            }
            Instruction::AddReg { x: rx, y: ry } => {
                info!("ADD {:?}, {:?}", rx, ry);
                let (num, overflow) = self
                    .get_register(rx)
                    .0
                    .overflowing_add(self.get_register(ry).0);
                self.set_vf(if overflow { Datum(1) } else { Datum(0) });
                self.set_register(rx, Datum(num));
            }
            Instruction::Sub { x: rx, y: ry } => {
                info!("SUB {:?}, {:?}", rx, ry);
                // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
                let (x, y) = (self.get_register(rx), self.get_register(ry));
                self.set_vf(Datum(u8::from(x > y)));
                self.set_register(rx, Datum(x.0.overflowing_sub(y.0).0));
            }
            Instruction::Shr(rx) => {
                info!("SHR {:?}", rx);
                let number = self.get_register(rx).0;
                let right = number & 0b1;
                self.set_vf(Datum(u8::from(right != 0)));
                self.set_register(rx, Datum(number >> 1));
            }
            Instruction::SubN { x: rx, y: ry } => {
                info!("SUBN {:?}, {:?}", rx, ry);
                // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
                let (x, y) = (self.get_register(rx), self.get_register(ry));
                self.set_vf(Datum(u8::from(y > x)));
                self.set_register(rx, Datum(y.0.overflowing_sub(x.0).0));
            }
            Instruction::Shl(rx) => {
                info!("SHL {:?}", rx);
                let number = self.get_register(rx).0;
                let right = number & 0b10000000;
                self.set_vf(Datum(u8::from(right != 0)));
                self.set_register(rx, Datum(number << 1));
            }
            Instruction::SkipRegistersNotEqual(r1, r2) => {
                let c1 = self.get_register(r1);
                let c2 = self.get_register(r2);
                if c1 != c2 {
                    info!("Skipping next instruction! (NE)");
                    self.increment_program_counter();
                    self.increment_program_counter();
                } else {
                    info!("Not skipping next instruction! (EQ)");
                }
            }
            Instruction::LoadImmediate(value) => {
                info!("Load immediate {:03X} into I", value);
                self.register_i = value.as_u16();
            }
            Instruction::JumpRelative(rel_addr) => {
                info!("Relative jump to V0 + {:02X}", rel_addr);
                let v0 = self.get_register(GeneralRegister::V0);
                let target = v0.0 as u16 + rel_addr.as_u16();
                if target & 0xF000 != 0 {
                    error!("Invalid jump address! 0x{:X} is out of bounds!", target);
                    panic!()
                }
                if target + 2 == self.program_counter.as_u16() {
                    warn!("Entering busywait loop, stopping.");
                    info!("Loop at 0x{:02X}", self.program_counter);
                    frame.busywait();
                }
                self.program_counter = Address::new(target);
            }
            Instruction::Random(reg, byte) => {
                let random = self.rng.gen::<u8>();
                let data = random & byte;
                info!("Random is {}", data);
                self.set_register(reg, Datum(data));
            }
            Instruction::DisplaySprite {
                x: vx,
                y: vy,
                number_of_bytes,
            } => {
                info!(
                    "Display sprite; RX={:?} RY={:?} bytes={}",
                    vx, vy, number_of_bytes
                );
                let addr = Address::new(self.register_i);
                let x_coord = self.get_register(vx);
                let y_coord = self.get_register(vy);
                debug!("sprite={:03X} x={} y={}", addr, x_coord.0, y_coord.0);
                let m = self.display.sprite(
                    x_coord,
                    y_coord,
                    self.memory.substring(addr, number_of_bytes),
                );
                self.set_vf(Datum(u8::from(m == ScreenModification::Clears)));
                frame.modify_screen()
            }
            Instruction::SkipPressed(reg) => {
                let data = self.get_register(reg);
                let key = Keys::from_datum(data);
                if (keys & key).pressed() {
                    info!("Skipping next instruction! ({:?} pressed)", key);
                    self.increment_program_counter();
                    self.increment_program_counter();
                } else {
                    info!("Not skipping next instruction! ({:?} not pressed)", key);
                }
            }
            Instruction::SkipNotPressed(reg) => {
                let data = self.get_register(reg);
                let key = Keys::from_datum(data);
                if !(keys & key).pressed() {
                    info!("Skipping next instruction! ({:?} not pressed)", key);
                    self.increment_program_counter();
                    self.increment_program_counter();
                } else {
                    info!("Not skipping next instruction! ({:?} pressed)", key);
                }
            }
            Instruction::GetDelayTimer(reg) => {
                info!("Getting delay timer into {:?}", reg);
                let num = self.delay_timer;
                self.set_register(reg, num);
            }
            Instruction::WaitForKey(reg) => {
                info!("Waiting to store next keypress in {:?}", reg);
                frame.wait_for_key_on(reg);
            }
            Instruction::SetDelayTimer(reg) => {
                info!("Setting delay timer to {:?}", reg);
                let num = self.get_register(reg);
                self.delay_timer = num;
            }
            Instruction::SetSoundTimer(reg) => {
                info!("Setting sound timer to {:?}", reg);
                let num = self.get_register(reg);
                self.sound_timer = num;
            }
            Instruction::AddI(reg) => {
                info!("Add {:?} to I", reg);
                self.set_i(
                    self.get_i()
                        .overflowing_add(self.get_register(reg).0 as u16)
                        .0,
                );
            }
            Instruction::GetSprite(reg) => {
                info!("Get sprite location for {:?}", reg);
                let num = self.register(reg).0;
                assert!(num < 16);
                let addr = FONT_START_ADDR as u16 + (num * 5) as u16;
                self.set_i(addr)
            }
            Instruction::BCD(reg) => {
                info!("BCD {:?}", reg);
                let num = self.get_register(reg).0;
                let units = num % 10;
                let tens = (num / 10) % 10;
                let hundreds = (num / 100) % 10;
                let i = self.register_i;
                self.memory[Address::new(i)] = Datum(hundreds);
                self.memory[Address::new(i + 1)] = Datum(tens);
                self.memory[Address::new(i + 2)] = Datum(units);
            }
            Instruction::WriteMultiple(until_reg) => {
                info!("Read to I+ until {:?}", until_reg);
                for (i, reg) in until_reg.until_including().enumerate() {
                    let data = self.get_register(reg);
                    self.memory[Address::new(self.register_i + i as u16)] = data;
                }
            }
            Instruction::ReadMultiple(until_reg) => {
                info!("Read from I+ through {:?}", until_reg);
                for (i, reg) in until_reg.until_including().enumerate() {
                    let data = self.memory[Address::new(self.register_i + i as u16)];
                    self.set_register(reg, data);
                }
            }
        }
    }

    fn vf(&self) -> Datum {
        *self.register(GeneralRegister::VF)
    }

    fn vf_mut(&mut self) -> &mut Datum {
        self.register_mut(GeneralRegister::VF)
    }

    fn set_vf(&mut self, to: Datum) {
        *self.vf_mut() = to;
    }

    fn empty() -> Self {
        Self {
            program_counter: Address::PROGRAM_START,
            memory: Memory::empty(),
            display: Display::blank(),
            general_registers: [Datum(0); 16],
            register_i: 0,
            stack: Vec::with_capacity(16),
            delay_timer: Datum(0),
            sound_timer: Datum(0),
            rng: OsRng,
        }
    }

    pub fn new_assembled<F: FnOnce(&mut asm::Assembler) -> &mut asm::Assembler>(with: F) -> Self {
        Self::new_from_rom(Self::assembled_program(with))
    }

    pub fn new_assembled_save<F: FnOnce(&mut asm::Assembler) -> &mut asm::Assembler>(
        to: impl AsRef<std::path::Path>,
        with: F,
    ) -> Self {
        let program = Self::assembled_program(with);
        program.save(to);
        Self::new_from_rom(program)
    }

    fn assembled_program<F: FnOnce(&mut asm::Assembler) -> &mut asm::Assembler>(with: F) -> ROM {
        let mut assembler = asm::Assembler::new();
        (with)(&mut assembler);
        assembler.assemble()
    }

    pub fn new_from_mem_file(path: impl AsRef<std::path::Path>) -> Self {
        let memory = Memory::from_file(path).unwrap();
        Self::new_from_memory(memory)
    }

    pub fn new_from_file(path: impl AsRef<std::path::Path>) -> Self {
        let program = ROM::from_file(path).unwrap();
        Self::new_from_rom(program)
    }

    pub fn new_from_memory(memory: Memory) -> Self {
        Self {
            memory,
            ..Self::empty()
        }
    }

    pub fn new_from_rom(rom: ROM) -> Self {
        Self {
            memory: rom.to_memory(),
            ..Self::empty()
        }
    }
}
