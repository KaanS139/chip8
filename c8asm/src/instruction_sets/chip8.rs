use crate::compilation::{ConcreteValue, InstructionBuilder, InstructionError};
use crate::parsing::ReservedName;
use c8common::GeneralRegister as VX;
use miette::SourceSpan;
use std::collections::HashMap;

pub struct Chip8InstructionSet;

impl InstructionBuilder for Chip8InstructionSet {
    fn instruction(
        opcode: String,
        arguments: Vec<ConcreteValue>,
        at: SourceSpan,
        labels: &HashMap<String, u16>,
    ) -> Result<(u8, u8), InstructionError> {
        match &opcode[..] {
            "cls" => Self::no_args(at, arguments, (0x00, 0xE0)),
            "ret" => Self::no_args(at, arguments, (0x00, 0xEE)),
            "jp" => Self::jump(at, arguments, labels),
            "call" => Self::call(at, arguments, labels),
            "se" => Self::skip(false, at, arguments),
            "sne" => Self::skip(true, at, arguments),
            "ld" => Self::load(at, arguments, labels),
            "add" => Self::add(at, arguments),

            op @ ("or" | "and" | "xor" | "sub" | "subn") => {
                Self::operation(op, at, Self::two_args(at, arguments)?)
            }
            // ALU
            // Rand
            "drw" => Self::draw_sprite(at, arguments),
            "skp" => Self::skip_key(false, at, arguments),
            "sknp" => Self::skip_key(true, at, arguments),
            _ => Err(InstructionError::UnknownInstruction { opcode, at }),
        }
    }
}

fn split_raw(raw: u16) -> (u8, u8) {
    let [high, low] = raw.to_be_bytes();
    (high, low)
}

fn byte(at: SourceSpan, raw: u16) -> Result<u8, InstructionError> {
    InstructionError::expects_byte(at, raw)
}

fn register_to_byte(reg: VX) -> u8 {
    reg.index() as u8
}

impl Chip8InstructionSet {
    fn no_args(
        at: SourceSpan,
        arguments: Vec<ConcreteValue>,
        correct: (u8, u8),
    ) -> Result<(u8, u8), InstructionError> {
        if arguments.is_empty() {
            Ok(correct)
        } else {
            Err(InstructionError::too_many_arguments(at, 1, arguments.len()))
        }
    }

    fn two_args(
        at: SourceSpan,
        mut arguments: Vec<ConcreteValue>,
    ) -> Result<(ConcreteValue, ConcreteValue), InstructionError> {
        if let [_, _] = arguments[..] {
            let last = arguments.pop().expect("known by match");
            let first = arguments.pop().expect("known by match");
            Ok((first, last))
        } else if arguments.len() > 2 {
            Err(InstructionError::too_many_arguments(at, 2, arguments.len()))
        } else {
            Err(InstructionError::not_enough_arguments(
                at,
                2,
                arguments.len(),
            ))
        }
    }

    fn one_arg(
        at: SourceSpan,
        mut arguments: Vec<ConcreteValue>,
    ) -> Result<ConcreteValue, InstructionError> {
        if let [_] = arguments[..] {
            let first = arguments.pop().expect("known by match");
            Ok(first)
        } else if arguments.len() > 2 {
            Err(InstructionError::too_many_arguments(at, 1, arguments.len()))
        } else {
            Err(InstructionError::not_enough_arguments(
                at,
                1,
                arguments.len(),
            ))
        }
    }

    fn skip(
        invert: bool,
        at: SourceSpan,
        arguments: Vec<ConcreteValue>,
    ) -> Result<(u8, u8), InstructionError> {
        use ConcreteValue::*;
        let (first, last) = Self::two_args(at, arguments)?;
        match (first, last) {
            (Register(reg), Numeric(num)) => {
                let byte = byte(at, num)?;
                Ok((
                    if invert { 0x40 } else { 0x30 } | register_to_byte(reg),
                    byte,
                ))
            }
            (Register(rx), Register(ry)) => Ok((
                if invert { 0x90 } else { 0x50 } | register_to_byte(rx),
                register_to_byte(ry) << 4,
            )),
            _ => Err(InstructionError::invalid_type(
                at,
                "a register+byte or register+register pair",
                "something else",
            )),
        }
    }

    fn skip_key(
        invert: bool,
        at: SourceSpan,
        arguments: Vec<ConcreteValue>,
    ) -> Result<(u8, u8), InstructionError> {
        use ConcreteValue::*;
        match Self::one_arg(at, arguments)? {
            Register(reg) => Ok((
                0xE0 | register_to_byte(reg),
                if invert { 0xA1 } else { 0x9E },
            )),
            _ => Err(InstructionError::invalid_type(
                at,
                "a register",
                "something else",
            )),
        }
    }

    fn operation(
        op: &str,
        at: SourceSpan,
        args: (ConcreteValue, ConcreteValue),
    ) -> Result<(u8, u8), InstructionError> {
        let vx = InstructionError::expects_register(at, args.0)?;
        let vy = InstructionError::expects_register(at, args.1)?;
        let (high, mut low) = (0x80 | register_to_byte(vx), register_to_byte(vy) << 4);

        low |= match op {
            "or" => 0x01,
            "and" => 0x02,
            "xor" => 0x03,
            "add" => 0x04,
            "sub" => 0x05,
            "subn" => 0x07,
            _ => panic!("Only operations should be passed to this function! {}", op),
        };

        Ok((high, low))
    }

    fn add(at: SourceSpan, arguments: Vec<ConcreteValue>) -> Result<(u8, u8), InstructionError> {
        use ConcreteValue::*;
        let (first, last) = Self::two_args(at, arguments)?;
        if let (Register(reg), Numeric(num)) = (&first, &last) {
            let byte = byte(at, *num)?;
            Ok((0x70 | register_to_byte(*reg), byte))
        } else {
            Self::operation("add", at, (first, last))
        }
    }

    fn draw_sprite(
        at: SourceSpan,
        mut arguments: Vec<ConcreteValue>,
    ) -> Result<(u8, u8), InstructionError> {
        if let [_, _, _] = arguments[..] {
            let last = arguments.pop().expect("known by match");
            let mid = arguments.pop().expect("known by match");
            let first = arguments.pop().expect("known by match");

            let num = match last {
                ConcreteValue::Numeric(num) => InstructionError::expects_nibble(at, num),
                _ => Err(InstructionError::invalid_type(
                    at,
                    "a number",
                    "something else",
                )),
            }?;
            let rx = InstructionError::expects_register(at, first)?;
            let ry = InstructionError::expects_register(at, mid)?;

            Ok((
                0xD0 | register_to_byte(rx),
                (register_to_byte(ry) << 4) | num,
            ))
        } else if arguments.len() > 3 {
            Err(InstructionError::too_many_arguments(at, 3, arguments.len()))
        } else {
            Err(InstructionError::not_enough_arguments(
                at,
                3,
                arguments.len(),
            ))
        }
    }

    fn load(
        at: SourceSpan,
        mut arguments: Vec<ConcreteValue>,
        labels: &HashMap<String, u16>,
    ) -> Result<(u8, u8), InstructionError> {
        use ConcreteValue::*;
        use ReservedName::*;
        match arguments[..] {
            [_] => Err(InstructionError::not_enough_arguments(at, 2, 1)),
            [_, _] => {
                let second = arguments
                    .pop()
                    .expect("known by match")
                    .bake_label(at, labels)?;
                let first = arguments
                    .pop()
                    .expect("known by match")
                    .bake_label(at, labels)?;

                match (first, second) {
                    (Register(reg), Numeric(num)) => {
                        let byte = byte(at, num)?;
                        Ok((0x60 | register_to_byte(reg), byte))
                    }
                    (Register(rx), Register(ry)) => {
                        Ok((0x80 | register_to_byte(rx), register_to_byte(ry) << 4))
                    }
                    (Reserved(I), Numeric(addr)) => {
                        let addr = InstructionError::address(at, addr)?;
                        let [high, low] = addr.to_be_bytes();
                        Ok((0xA0 | high, low))
                    }
                    (Register(reg), Reserved(DT)) => Ok((0xF0 | register_to_byte(reg), 0x07)),
                    (Register(reg), Reserved(K)) => Ok((0xF0 | register_to_byte(reg), 0x0A)),
                    (Reserved(DT), Register(reg)) => Ok((0xF0 | register_to_byte(reg), 0x15)),
                    (Reserved(ST), Register(reg)) => Ok((0xF0 | register_to_byte(reg), 0x18)),
                    (Reserved(F), Register(reg)) => Ok((0xF0 | register_to_byte(reg), 0x29)),
                    (Reserved(B), Register(reg)) => Ok((0xF0 | register_to_byte(reg), 0x33)),
                    (Reserved(I), Register(reg)) => Ok((0xF0 | register_to_byte(reg), 0x55)),
                    (Register(reg), Reserved(I)) => Ok((0xF0 | register_to_byte(reg), 0x65)),
                    _ => Err(InstructionError::invalid_load(at)),
                }
            }
            _ => Err(InstructionError::too_many_arguments(at, 2, arguments.len())),
        }
    }

    fn call(
        at: SourceSpan,
        mut arguments: Vec<ConcreteValue>,
        labels: &HashMap<String, u16>,
    ) -> Result<(u8, u8), InstructionError> {
        let target = match arguments[..] {
            [] => Err(InstructionError::not_enough_arguments(
                at,
                1,
                arguments.len(),
            )),
            [_] => Self::get_jump_target(at, arguments.pop().expect("known by match"), labels),
            _ => Err(InstructionError::too_many_arguments(at, 1, arguments.len())),
        }?;
        let (high, low) = split_raw(target);
        assert!(high <= 0xF);
        Ok((0x20 | high, low))
    }

    fn jump(
        at: SourceSpan,
        mut arguments: Vec<ConcreteValue>,
        labels: &HashMap<String, u16>,
    ) -> Result<(u8, u8), InstructionError> {
        match arguments[..] {
            [] => Err(InstructionError::not_enough_arguments(
                at,
                1,
                arguments.len(),
            )),
            [_] => {
                let target =
                    Self::get_jump_target(at, arguments.pop().expect("known by match"), labels)?;
                let (high, low) = split_raw(target);
                assert!(high <= 0xF);
                Ok((0x10 | high, low))
            }
            [_, _] => {
                let addr = arguments.pop().expect("known by match");
                let reg = arguments.pop().expect("known by match");
                if reg != ConcreteValue::Register(VX::V0) {
                    Err(InstructionError::invalid_type(
                        at,
                        "the register V0",
                        "something else",
                    ))
                } else {
                    let relative = Self::get_jump_target(at, addr, labels)?;
                    let (high, low) = split_raw(relative);
                    assert!(high <= 0xF);
                    Ok((0xB0 | high, low))
                }
            }
            _ => Err(InstructionError::too_many_arguments(at, 1, arguments.len())),
        }
    }

    fn get_jump_target(
        at: SourceSpan,
        only: ConcreteValue,
        labels: &HashMap<String, u16>,
    ) -> Result<u16, InstructionError> {
        let target = match only {
            ConcreteValue::Numeric(addr) => addr,
            ConcreteValue::Register(_) => Err(InstructionError::invalid_type(
                at,
                "a valid jump address",
                "a register",
            ))?,
            ConcreteValue::Label(_) => only
                .bake_label(at, labels)
                .map(|inner| inner.numeric().expect("known to be an address"))?,
            ConcreteValue::Reserved(_) => Err(InstructionError::invalid_type(
                at,
                "a valid jump address",
                "a reserved name",
            ))?,
        };
        InstructionError::address(at, target)
    }
}
