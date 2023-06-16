//! Specifies the Displacement and Immediate rules and parsing mechanism
use crate::{opcode::OpSize, reader::{Reader, ReaderError}};

/// The "displacement" is just a constant that gets added to the rest of the address. Examples
/// include:
/// - [reg + displacement]
/// - [displacmeent]
/// - [reg * constant + displacement]
/// Some addressing forms include a displacement immediately following the ModR/M byte (or the SIB
/// byte if one is present). If a displacement is required, it can be 1, 2, or 4 bytes.
#[derive(Debug, PartialEq, Eq)]
pub enum Displacement {
    Disp8(u8),
    Disp16(u16),
    Disp32(u32),
    Disp64(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispArch {
    // Denotes an 8-bit displacement that follows the ModR/M byte and that is sign-extended and
    // added to the index.
    Bit8,
    // Denotes a 16-bit displacement that follows the ModR/M byte and that is added to the index.
    Bit16,
    Bit32,
    Bit64,
}

impl DispArch {
    pub fn read(&self, reader: &mut Reader) -> Result<Displacement, DispError> {
        match self {
            Self::Bit8 => Ok(Displacement::Disp8(reader.read::<u8>()?)),
            Self::Bit16 => Ok(Displacement::Disp16(reader.read::<u16>()?)),
            Self::Bit32 => Ok(Displacement::Disp32(reader.read::<u32>()?)),
            Self::Bit64 => Ok(Displacement::Disp64(reader.read::<u64>()?)),
        }
    }
}

#[derive(Debug)]
pub enum DispError {
    ReaderError(ReaderError),
}

impl From<ReaderError> for DispError {
    fn from(err: ReaderError) -> Self {
        Self::ReaderError(err)
    }
}

/// If an instruction specifies an immediate operand, the operand always follows any displacement
/// bytes. An immediate operand can be 1, 2 or 4 bytes
#[derive(Debug, PartialEq, Eq)]
pub enum Immediate {
    Imm8(u8),
    Imm16(u16),
    Imm32(u32),
    ImmI64(i64),
}

impl Immediate {
    pub fn parse(op_size: &OpSize, reader: &mut Reader) -> Result<Self, ImmError> {
        match op_size {
            OpSize::U8 => Ok(Immediate::Imm8(reader.read::<u8>()?)),
            OpSize::U16 => Ok(Immediate::Imm16(reader.read::<u16>()?)),
            OpSize::U32 => Ok(Immediate::Imm32(reader.read::<u32>()?)),
            OpSize::U64 => Ok(Immediate::ImmI32(reader.read::<i32>()?)),
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub enum ImmError {
    ReaderError(ReaderError),
}

impl From<ReaderError> for ImmError {
    fn from(err: ReaderError) -> Self {
        Self::ReaderError(err)
    }
}
