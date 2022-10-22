use crate::data::Nibble;
use crate::{Address, Datum, GeneralRegister as VX};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Instruction {
    /// 0000 (Not a standard instruction
    /// Does nothing
    Nop,
    /// 00E0
    /// Clears the screen (all pixels to black)
    ClearScreen,
    // 00EE - Ret
    /// 1nnn
    /// The interpreter sets the program counter to _nnn_.
    Jump(Address),
    /// 2nnn
    /// The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
    Call(Address),

    /// 6xkk
    /// The interpreter puts the value _kk_ into register _Vx_.
    LoadRegByte(VX, u8),

    /// Annn
    /// The value of register I is set to _nnn_.
    LoadImmediate(Address),

    /// Dxyn
    /// The interpreter reads n bytes from memory, starting at the address stored in I.
    /// These bytes are then displayed as sprites on screen at coordinates (_Vx_, _Vy_).
    /// Sprites are XORed onto the existing screen.
    /// If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.
    /// If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen.
    /// See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
    DisplaySprite {
        x: VX,
        y: VX,
        number_of_bytes: Nibble,
    },
}

impl Instruction {
    pub fn try_from_data(data: RawInstruction) -> Result<Self, InstructionDecodeError> {
        let [n1, n2, n3, n4] = data.as_nibbles();
        match (
            n1.as_half_byte(),
            n2.as_half_byte(),
            n3.as_half_byte(),
            n4.as_half_byte(),
        ) {
            // https://rs118.uwcs.co.uk/chip8.html
            (0x0, 0x0, 0x0, 0x0) => Ok(Self::Nop),
            (0x0, 0x0, 0xE, 0x0) => Ok(Self::ClearScreen),
            (0x1, a1, a2, a3) => Ok(Self::Jump(Address::from_triplet(a1, a2, a3))),
            (0x2, a1, a2, a3) => Ok(Self::Call(Address::from_triplet(a1, a2, a3))),

            (0x6, _, _, _) => Ok(Self::LoadRegByte(VX::from_nibble(n2), n3.byte_with(n4))),

            (0xA, a1, a2, a3) => Ok(Self::LoadImmediate(Address::from_triplet(a1, a2, a3))),
            (0xD, _, _, _) => Ok(Self::DisplaySprite {
                x: VX::from_nibble(n2),
                y: VX::from_nibble(n3),
                number_of_bytes: n4,
            }),
            _ => Err(InstructionDecodeError::InvalidInstruction(data)),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RawInstruction([Datum; 2]);

impl RawInstruction {
    pub fn from_raw_bytes(value: [u8; 2]) -> Self {
        Self(value.map(Datum))
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

#[derive(Debug, Clone)]
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
