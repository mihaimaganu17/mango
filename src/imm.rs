//! Specifies the Displacement and Immediate rules and parsing mechanism
use crate::{
    inst::SizedOperand,
    opcode::OpSize,
    reader::{Reader, ReaderError},
};
use core::fmt;

/// The "displacement" is just a constant that gets added to the rest of the address. Examples
/// include:
/// - [reg + displacement]
/// - [displacmeent]
/// - [reg * constant + displacement]
/// Some addressing forms include a displacement immediately following the ModR/M byte (or the SIB
/// byte if one is present). If a displacement is required, it can be 1, 2, or 4 bytes.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Displacement {
    Disp8(u8),
    Disp16(u16),
    Disp32(u32),
    Disp64(u64),
}

impl fmt::Display for Displacement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Displacement::Disp8(value) => write!(f, "0x{:x}", value),
            Displacement::Disp16(value) => write!(f, "0x{:x}", value),
            Displacement::Disp32(value) => write!(f, "0x{:x}", value),
            Displacement::Disp64(value) => write!(f, "0x{:x}", value),
        }
    }
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
/// bytes. An immediate operand can be 1, 2 or 4 bytes.
/// Intel 0x86 Immediates are always sign-extended, so they are always signed.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Immediate {
    ImmU8(u8),
    ImmU16(u16),
    ImmU32(u32),
    ImmU64(u64),
    ImmI8(i8),
    ImmI16(i16),
    ImmI32(i32),
    ImmI64(i64),
}

impl fmt::Display for Immediate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Immediate::ImmU8(value) => write!(f, "0x{:x}", value),
            Immediate::ImmU16(value) => write!(f, "0x{:x}", value),
            Immediate::ImmU32(value) => write!(f, "0x{:x}", value),
            Immediate::ImmU64(value) => write!(f, "0x{:x}", value),
            Immediate::ImmI8(value) => write!(f, "0x{:x}", value),
            Immediate::ImmI16(value) => write!(f, "0x{:x}", value),
            Immediate::ImmI32(value) => write!(f, "0x{:x}", value),
            Immediate::ImmI64(value) => write!(f, "0x{:x}", value),
        }
    }
}

impl SizedOperand for Immediate {
    fn size(&self) -> OpSize {
        match self {
            Immediate::ImmU8(value) => OpSize::U8,
            Immediate::ImmU16(value) => OpSize::U16,
            Immediate::ImmU32(value) => OpSize::U32,
            Immediate::ImmU64(value) => OpSize::U64,
            Immediate::ImmI8(value) => OpSize::I8,
            Immediate::ImmI16(value) => OpSize::I16,
            Immediate::ImmI32(value) => OpSize::I32,
            Immediate::ImmI64(value) => OpSize::I64,
        }
    }
}

impl Immediate {
    pub fn parse(op_size: &OpSize, reader: &mut Reader) -> Result<Self, ImmError> {
        match op_size {
            OpSize::U8 => Ok(Immediate::ImmI8(reader.read::<i8>()?)),
            OpSize::U16 => Ok(Immediate::ImmI16(reader.read::<i16>()?)),
            OpSize::U32 => Ok(Immediate::ImmI32(reader.read::<i32>()?)),
            OpSize::U64 => Ok(Immediate::ImmI32(reader.read::<i32>()?)),
            OpSize::I8 => Ok(Immediate::ImmI8(reader.read::<i8>()?)),
            OpSize::I16 => Ok(Immediate::ImmI16(reader.read::<i16>()?)),
            OpSize::I32 => Ok(Immediate::ImmI32(reader.read::<i32>()?)),
            OpSize::I64 => Ok(Immediate::ImmI32(reader.read::<i32>()?)),
            OpSize::CpuMode => Ok(Immediate::ImmI32(reader.read::<i32>()?)),
            _ => {
                println!("OpSize: {op_size:?}");
                todo!();
            }
        }
    }

    pub fn convert_with_opsize(self, op_size: OpSize) -> Self {
        match op_size {
            OpSize::CpuMode | OpSize::U8 | OpSize::I8 => self,
            OpSize::U16 | OpSize::I16 => match self {
                Immediate::ImmU8(value) => Immediate::ImmU16(value as u16),
                Immediate::ImmI8(value) => Immediate::ImmU16(value as u16),
                _ => self,
            },
            OpSize::U32 | OpSize::I32 => match self {
                Immediate::ImmU8(value) => Immediate::ImmU32(value as u32),
                Immediate::ImmI8(value) => Immediate::ImmU32(value as u32),
                Immediate::ImmU16(value) => Immediate::ImmU32(value as u32),
                Immediate::ImmI16(value) => Immediate::ImmU32(value as u32),
                _ => self,
            },
            OpSize::U64 | OpSize::I32 => match self {
                Immediate::ImmU8(value) => Immediate::ImmU64(value as u64),
                Immediate::ImmI8(value) => Immediate::ImmU64(value as u64),
                Immediate::ImmU16(value) => Immediate::ImmU64(value as u64),
                Immediate::ImmI16(value) => Immediate::ImmU64(value as u64),
                Immediate::ImmU32(value) => Immediate::ImmU64(value as u64),
                Immediate::ImmI32(value) => Immediate::ImmU64(value as u64),
                _ => self,
            },
            _ => self,
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
