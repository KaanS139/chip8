use crate::data::Nibble;
use crate::{Address, Datum, GeneralRegister as VX};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Instruction {
    /// 0000 (Not a standard instruction)
    /// Does nothing
    Nop,
    /// 00E0
    /// Clears the screen (all pixels to black)
    ClearScreen,
    /// 00EE
    /// The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    Return,
    /// 1nnn
    /// The interpreter sets the program counter to _nnn_.
    Jump(Address),
    /// 2nnn
    /// The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
    Call(Address),
    /// 3xkk
    /// The interpreter compares register _Vx_ to _kk_, and if they are equal, increments the program counter by 2.
    SkipIfEqual(VX, u8),
    /// 4xkk
    /// The interpreter compares register _Vx_ to _kk_, and if they are not equal, increments the program counter by 2.
    SkipNotEqual(VX, u8),
    /// 5xy0
    /// The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    SkipRegistersEqual(VX, VX),
    /// 6xkk
    /// The interpreter puts the value _kk_ into register _Vx_.
    LoadRegByte(VX, u8),
    /// 7xkk
    /// Adds the value _kk_ to the value of register _Vx_, then stores the result in _Vx_.
    Add(VX, u8),
    /// 8xy0
    /// Stores the value of register `Vy` in register `Vx`.
    CopyRegToReg { x: VX, y: VX },
    /// 8xy1
    /// Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx.
    Or { x: VX, y: VX },
    /// 8xy2
    /// Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx.
    And { x: VX, y: VX },
    /// 8xy3
    /// Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx.
    Xor { x: VX, y: VX },
    /// 8xy4
    /// The values of Vx and Vy are added together.
    /// If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0.
    /// Only the lowest 8 bits of the result are kept, and stored in Vx.
    AddReg { x: VX, y: VX },
    /// 8xy5
    /// If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
    Sub { x: VX, y: VX },
    /// 8xy6
    /// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    Shr(VX),
    /// 8xy7
    /// If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
    SubN { x: VX, y: VX },
    /// 8xyE
    /// If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    Shl(VX),
    /// 9xy0
    /// The interpreter compares register Vx to register Vy, and if they are not equal, increments the program counter by 2.
    SkipRegistersNotEqual(VX, VX),
    /// Annn
    /// The value of register I is set to _nnn_.
    LoadImmediate(Address),
    /// Bnnn
    /// The program counter is set to nnn plus the value of V0.
    JumpRelative(Address),
    /// Cxkk
    /// The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk.
    /// The results are stored in Vx.
    /// See instruction 8xy2 for more information on AND.
    Random(VX, u8),
    /// Dxyn
    /// The interpreter reads n bytes from memory, starting at the address stored in I.
    /// These bytes are then displayed as sprites on screen at coordinates (_Vx_, _Vy_).
    /// Sprites are XORed onto the existing screen.
    /// If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.
    /// If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen.
    /// See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
    DisplaySprite { x: VX, y: VX, number_of_bytes: u8 },
    /// Ex9E
    /// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.
    SkipPressed(VX),
    /// ExA1
    /// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.
    SkipNotPressed(VX),
    /// Fx07
    /// The value of DT is placed into Vx.
    GetDelayTimer(VX),
    /// Fx0A
    /// All execution stops until a key is pressed, then the value of that key is stored in Vx.
    WaitForKey(VX),
    /// Fx15
    /// DT is set equal to the value of Vx.
    SetDelayTimer(VX),
    /// Fx18
    /// ST is set equal to the value of Vx.
    SetSoundTimer(VX),
    /// Fx1E
    /// The values of I and Vx are added, and the results are stored in I.
    AddI(VX),
    /// Fx29
    /// The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx.
    /// See section 2.4, Display, for more information on the Chip-8 hexadecimal font.
    GetSprite(VX),
    /// Fx33
    /// The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
    BCD(VX),
    /// Fx55
    /// The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
    WriteMultiple(VX),
    /// Fx65
    /// The interpreter reads values from memory starting at location I into registers V0 through Vx.
    ReadMultiple(VX),
}

impl Instruction {
    pub fn try_from_data(data: RawInstruction) -> Result<Self, InstructionDecodeError> {
        let [n1, n2, n3, n4] = data.as_nibbles();
        match [
            n1.as_half_byte(),
            n2.as_half_byte(),
            n3.as_half_byte(),
            n4.as_half_byte(),
        ] {
            [0x0, 0x0, 0x0, 0x0] => Ok(Self::Nop),
            [0x0, 0x0, 0xE, 0x0] => Ok(Self::ClearScreen),
            [0x0, 0x0, 0xE, 0xE] => Ok(Self::Return),
            [0x1, a1, a2, a3] => Ok(Self::Jump(Address::from_triplet(a1, a2, a3))),
            [0x2, a1, a2, a3] => Ok(Self::Call(Address::from_triplet(a1, a2, a3))),
            [0x3, x, b1, b2] => Ok(Self::SkipIfEqual(VX::from_byte(x), byte_with(b1, b2))),
            [0x4, x, b1, b2] => Ok(Self::SkipNotEqual(VX::from_byte(x), byte_with(b1, b2))),
            [0x5, x, y, 0x0] => Ok(Self::SkipRegistersEqual(VX::from_byte(x), VX::from_byte(y))),
            [0x6, x, b1, b2] => Ok(Self::LoadRegByte(VX::from_byte(x), byte_with(b1, b2))),
            [0x7, x, b1, b2] => Ok(Self::Add(VX::from_byte(x), byte_with(b1, b2))),
            [0x8, x, y, 0x0] => Ok(Self::CopyRegToReg {
                x: VX::from_byte(x),
                y: VX::from_byte(y),
            }),
            [0x8, x, y, 0x1] => Ok(Self::Or {
                x: VX::from_byte(x),
                y: VX::from_byte(y),
            }),
            [0x8, x, y, 0x2] => Ok(Self::And {
                x: VX::from_byte(x),
                y: VX::from_byte(y),
            }),
            [0x8, x, y, 0x3] => Ok(Self::Xor {
                x: VX::from_byte(x),
                y: VX::from_byte(y),
            }),
            [0x8, x, y, 0x4] => Ok(Self::AddReg {
                x: VX::from_byte(x),
                y: VX::from_byte(y),
            }),
            [0x8, x, y, 0x5] => Ok(Self::Sub {
                x: VX::from_byte(x),
                y: VX::from_byte(y),
            }),
            [0x8, x, _, 0x6] => Ok(Self::Shr(VX::from_byte(x))),
            [0x8, x, y, 0x7] => Ok(Self::SubN {
                x: VX::from_byte(x),
                y: VX::from_byte(y),
            }),
            [0x8, x, _, 0xE] => Ok(Self::Shl(VX::from_byte(x))),
            [0x9, x, y, 0x0] => Ok(Self::SkipRegistersNotEqual(
                VX::from_byte(x),
                VX::from_byte(y),
            )),
            [0xA, a1, a2, a3] => Ok(Self::LoadImmediate(Address::from_triplet(a1, a2, a3))),
            [0xB, a1, a2, a3] => Ok(Self::JumpRelative(Address::from_triplet(a1, a2, a3))),
            [0xC, x, b1, b2] => Ok(Self::Random(VX::from_byte(x), byte_with(b1, b2))),
            [0xD, x, y, n] => Ok(Self::DisplaySprite {
                x: VX::from_byte(x),
                y: VX::from_byte(y),
                number_of_bytes: n,
            }),
            [0xE, x, 0x9, 0xE] => Ok(Self::SkipPressed(VX::from_byte(x))),
            [0xE, x, 0xA, 0x1] => Ok(Self::SkipNotPressed(VX::from_byte(x))),
            [0xF, x, 0x0, 0x7] => Ok(Self::GetDelayTimer(VX::from_byte(x))),
            [0xF, x, 0x0, 0xA] => Ok(Self::WaitForKey(VX::from_byte(x))),
            [0xF, x, 0x1, 0x5] => Ok(Self::SetDelayTimer(VX::from_byte(x))),
            [0xF, x, 0x1, 0x8] => Ok(Self::SetSoundTimer(VX::from_byte(x))),
            [0xF, x, 0x1, 0xE] => Ok(Self::AddI(VX::from_byte(x))),
            [0xF, x, 0x2, 0x9] => Ok(Self::GetSprite(VX::from_byte(x))),
            [0xF, x, 0x3, 0x3] => Ok(Self::BCD(VX::from_byte(x))),
            [0xF, x, 0x5, 0x5] => Ok(Self::WriteMultiple(VX::from_byte(x))),
            [0xF, x, 0x6, 0x5] => Ok(Self::ReadMultiple(VX::from_byte(x))),
            _ => Err(InstructionDecodeError::InvalidInstruction(data)),
        }
    }

    pub fn to_data(self) -> RawInstruction {
        use conversion::ConvertToRaw;
        match self {
            Self::Nop => 0x0000_u16.to_raw(),
            Self::ClearScreen => 0x00E0_u16.to_raw(),
            Self::Return => 0x00EE_u16.to_raw(),
            Self::Jump(addr) => (0x1, addr).to_raw(),
            Self::Call(addr) => (0x2, addr).to_raw(),
            Self::SkipIfEqual(reg, byte) => (0x3, reg, byte).to_raw(),
            Self::SkipNotEqual(reg, byte) => (0x4, reg, byte).to_raw(),
            Self::SkipRegistersEqual(r1, r2) => (0x5, r1, r2, 0).to_raw(),
            Self::LoadRegByte(reg, byte) => (0x6, reg, byte).to_raw(),
            Self::Add(reg, byte) => (0x7, reg, byte).to_raw(),
            Self::CopyRegToReg { x, y } => (0x8, x, y, 0).to_raw(),
            Self::Or { x, y } => (0x8, x, y, 1).to_raw(),
            Self::And { x, y } => (0x8, x, y, 2).to_raw(),
            Self::Xor { x, y } => (0x8, x, y, 3).to_raw(),
            Self::AddReg { x, y } => (0x8, x, y, 4).to_raw(),
            Self::Sub { x, y } => (0x8, x, y, 5).to_raw(),
            Self::Shr(reg) => (0x8, reg, 0x0, 0x6).to_raw(),
            Self::SubN { x, y } => (0x8, x, y, 7).to_raw(),
            Self::Shl(reg) => (0x8, reg, 0x0, 0xE).to_raw(),
            Self::SkipRegistersNotEqual(r1, r2) => (0x9, r1, r2, 0).to_raw(),
            Self::LoadImmediate(value) => (0xA, value).to_raw(),
            Self::JumpRelative(addr) => (0xB, addr).to_raw(),
            Self::Random(reg, byte) => (0xC, reg, byte).to_raw(),
            Self::DisplaySprite {
                x,
                y,
                number_of_bytes,
            } => (0xD, x, y, number_of_bytes).to_raw(),
            Self::SkipPressed(reg) => (0xE, reg, 0x9E).to_raw(),
            Self::SkipNotPressed(reg) => (0xE, reg, 0xA1).to_raw(),
            Self::GetDelayTimer(reg) => (0xF, reg, 0x07).to_raw(),
            Self::WaitForKey(reg) => (0xF, reg, 0x0A).to_raw(),
            Self::SetDelayTimer(reg) => (0xF, reg, 0x15).to_raw(),
            Self::SetSoundTimer(reg) => (0xF, reg, 0x18).to_raw(),
            Self::AddI(reg) => (0xF, reg, 0x1E).to_raw(),
            Self::GetSprite(reg) => (0xF, reg, 0x29).to_raw(),
            Self::BCD(reg) => (0xF, reg, 0x33).to_raw(),
            Self::WriteMultiple(reg) => (0xF, reg, 0x55).to_raw(),
            Self::ReadMultiple(reg) => (0xF, reg, 0x65).to_raw(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RawInstruction([Datum; 2]);

impl RawInstruction {
    pub fn from_raw_bytes(value: [u8; 2]) -> Self {
        Self(value.map(Datum))
    }

    pub fn from_raw_nibbles(nibbles: [Nibble; 4]) -> Self {
        let bytes = [
            nibbles[0].byte_with(nibbles[1]),
            nibbles[2].byte_with(nibbles[3]),
        ];
        Self::from_raw_bytes(bytes)
    }

    pub fn as_nibbles(&self) -> [Nibble; 4] {
        let ([n1, n2], [n3, n4]) = (self.0[0].as_nibbles(), self.0[1].as_nibbles());
        [n1, n2, n3, n4]
    }

    pub fn first(&self) -> Datum {
        self.0[0]
    }

    pub fn highest(&mut self) -> &mut Datum {
        &mut self.0[0]
    }

    pub fn second(&self) -> Datum {
        self.0[1]
    }
}

impl From<u16> for RawInstruction {
    fn from(value: u16) -> Self {
        Self(value.to_be_bytes().map(Datum))
    }
}

impl From<&u16> for RawInstruction {
    fn from(value: &u16) -> Self {
        Self(value.to_be_bytes().map(Datum))
    }
}

impl From<(u8, u8)> for RawInstruction {
    fn from(value: (u8, u8)) -> Self {
        Self([value.0, value.1].map(Datum))
    }
}

impl From<(Datum, Datum)> for RawInstruction {
    fn from(value: (Datum, Datum)) -> Self {
        Self([value.0, value.1])
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(missing_copy_implementations)]
pub enum InstructionDecodeError {
    InvalidInstruction(RawInstruction),
}

impl InstructionDecodeError {
    pub fn invalid_data(self) -> Option<RawInstruction> {
        match self {
            Self::InvalidInstruction(inner) => Some(inner),
        }
    }
}

fn byte_with(a: u8, b: u8) -> u8 {
    assert_eq!(a & 0xF0, 0x00);
    assert_eq!(b & 0xF0, 0x00);
    a << 4 | b
}

mod conversion {
    use crate::{Address, Datum, GeneralRegister, Nibble, RawInstruction};

    pub(super) trait ConvertToRaw {
        fn to_raw(self) -> RawInstruction;
    }

    impl ConvertToRaw for (u8, u8, u8, u8) {
        fn to_raw(self) -> RawInstruction {
            RawInstruction::from_raw_nibbles(
                [self.0, self.1, self.2, self.3].map(Nibble::new_from_half_byte),
            )
        }
    }

    impl ConvertToRaw for (u8, Address) {
        fn to_raw(self) -> RawInstruction {
            let [a1, a2, a3] = self.1.to_nibbles();
            let n1 = Nibble::new_from_half_byte(self.0);
            RawInstruction::from_raw_nibbles([n1, a1, a2, a3])
        }
    }

    impl ConvertToRaw for (u8, GeneralRegister, u8) {
        fn to_raw(self) -> RawInstruction {
            let [high, low] = Datum(self.2).as_nibbles();
            RawInstruction::from_raw_nibbles([
                Nibble::new_from_half_byte(self.0),
                self.1.to_nibble(),
                high,
                low,
            ])
        }
    }

    impl ConvertToRaw for (u8, GeneralRegister, GeneralRegister, u8) {
        fn to_raw(self) -> RawInstruction {
            RawInstruction::from_raw_nibbles([
                Nibble::new_from_half_byte(self.0),
                self.1.to_nibble(),
                self.2.to_nibble(),
                Nibble::new_from_half_byte(self.3),
            ])
        }
    }

    impl ConvertToRaw for (u8, GeneralRegister, u8, u8) {
        fn to_raw(self) -> RawInstruction {
            RawInstruction::from_raw_nibbles([
                Nibble::new_from_half_byte(self.0),
                self.1.to_nibble(),
                Nibble::new_from_half_byte(self.2),
                Nibble::new_from_half_byte(self.3),
            ])
        }
    }

    impl ConvertToRaw for u16 {
        fn to_raw(self) -> RawInstruction {
            let [high, low] = self.to_be_bytes();
            RawInstruction::from_raw_bytes([high, low])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_conversion() {
        for i in 0..=u16::MAX {
            let bytes = i.to_be_bytes();
            let inst = Instruction::try_from_data(RawInstruction::from_raw_bytes(bytes));
            if let Ok(inst) = inst {
                let reconstituted = {
                    let returned = inst.to_data();
                    let bytes = returned.0.map(Datum::inner);
                    u16::from_be_bytes(bytes)
                };
                let inst_2 = Instruction::try_from_data(RawInstruction::from_raw_bytes(
                    reconstituted.to_be_bytes(),
                ));
                assert_eq!(
                    Ok(inst),
                    inst_2,
                    "test failed on instruction {:04X} (expected {:?}, got {:?})",
                    i,
                    inst,
                    inst_2
                );
            }
        }
    }
}
