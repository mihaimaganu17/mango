//! Module that takes care of parsing the Opcode field in an instruction.
use crate::{
    prefix::{Prefix, Group1},
    reader::{Reader, ReaderError},
    rex::Rex,
    reg::Reg,
};
use core::fmt::Debug;

/// Represents a primary opcode in an x86_64 Architecture. The primary opcode can be 1, 2 or even
/// 3 bytes in length. An additional 3-bit opcode field is sometimes encoded in the ModR/M byte.
/// Smaller fields can be defines within the primary opcode. Such fields can define:
/// - direction of operation
/// - size of displacements
/// - register encoding
/// - condition codes
/// - sign extension
///
/// Two-byte opcode formats for general-purpose and SIMD instructions consists of either:
/// - An escape code like `TWO_BYTE_ESCAPE_CODE`
/// - A prefix from `prefix.rs:Prefix` and the escape code mentioned above.
///
/// Three-bytes opcode formats are just like above, but instead of 1 bytes following the escape
/// code, there are 2 bytes
#[derive(Debug)]
pub enum OpcodeType {
    // A prefix byte for special operations or extending the instruction encoding
    Prefix(Prefix),
    // A REX prefix used to configure 64-bit mode operations
    Rex(Rex),
    // A bitwise XOR between 2 operands
    Xor,
    // The opcode alone is not enough and it needs an Extension from a ModRM field
    NeedsModRMExtension,
    // Terminate an indirect branch in 32 bit and compatibility mode.
    EndBr32,
    // Terminate an indirect branch in 64 bit mode.
    EndBr64,
    // Specifies and unknown opcode
    Unknown,
}

#[derive(Debug)]
pub struct Opcode {
    pub ident: OpcodeType,
    pub operands: [Option<Operand>; 4],
    pub op_size: Option<OperandSize>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum OperandSize {
    Byte,
    Word,
    DoubleWord,
    QuadWord,
    None,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Operand {
    // Represents a register or a memory operand found in the R/M field of ModR/M
    ModRM,
    // Represents a register from the `reg` part of the ModRM field
    ModReg,
    // The operand is embedded in the opcode
    Opcode,
    // There is an Immediate integer following the opcode that represents the operand
    Immediate,
    // The operand is a specific register
    Reg(Reg),
}

#[derive(Debug)]
pub enum OpcodeError {
    ReaderError(ReaderError),
    InvalidPrefix(Prefix),
    Invalid3ByteOpcode(u8, u8, u8),
}

impl From<ReaderError> for OpcodeError {
    fn from(err: ReaderError) -> Self {
        Self::ReaderError(err)
    }
}

impl Opcode {
    /// Reads one byte from the passed reader and parses it
    pub fn from_reader(reader: &mut Reader) -> Result<Self, OpcodeError> {
        // Read the first byte from the `reader`
        let byte = reader.read::<u8>()?;

        Self::from_byte(byte)
    }

    /// Parse the next `Opcode` from the `reader`, given the prefix. We need to pass the `reader`
    /// to this function, since we do not know if the opcode is 1, 2 or 3 bytes
    pub fn from_byte(byte: u8) -> Result<Self, OpcodeError> {
        // We first try and parse the byte for a prefix
        let maybe_prefix = Prefix::from_byte(byte);

        // If we do get a prefix, we return and it is the caller job, to do something with it
        if let Some(prefix) = maybe_prefix {
            return Ok(Opcode {
                ident: OpcodeType::Prefix(prefix),
                operands: [None, None, None, None],
                op_size: None,
            });
        }

        // If it is not a prefix, we still need to check for a REX prefix
        let maybe_rex = Rex::from_byte(byte);

        // If we do get a REX prefix, we return and it is the caller's job to call opcode parsing
        // again for the next byte
        if let Some(rex) = maybe_rex {
            return Ok(Opcode {
                ident: OpcodeType::Rex(rex),
                operands: [None, None, None, None],
                op_size: None,
            });
        }

        // This(soon to be gigantic match) will check the byte for the appropriate instruction.
        // It is the job of this match to make sure we propagate the information upwards, that the
        // calling function needs, in order to parse the rest of the bytes
        match byte {
            // XOR opcodes
            0x30 => Ok(Opcode {
                ident: OpcodeType::Xor,
                operands: [Some(Operand::ModRM), Some(Operand::ModReg), None, None],
                op_size: Some(OperandSize::Byte),
            }),
            0x31 => Ok(Opcode {
                ident: OpcodeType::Xor,
                operands: [Some(Operand::ModRM), Some(Operand::ModReg), None, None],
                op_size: Some(OperandSize::DoubleWord),
            }),
            0x32 => Ok(Opcode {
                ident: OpcodeType::Xor,
                operands: [Some(Operand::ModReg), Some(Operand::ModRM), None, None],
                op_size: Some(OperandSize::Byte),
            }),
            0x33 => Ok(Opcode {
                ident: OpcodeType::Xor,
                operands: [Some(Operand::ModReg), Some(Operand::ModRM), None, None],
                op_size: Some(OperandSize::DoubleWord),
            }),
            0x34 => Ok(Opcode {
                ident: OpcodeType::Xor,
                operands: [Some(Operand::Reg(Reg::AL)), Some(Operand::Immediate), None, None],
                op_size: Some(OperandSize::Byte),
            }),
            0x35 => Ok(Opcode {
                ident: OpcodeType::Xor,
                operands: [Some(Operand::Reg(Reg::EAX)), Some(Operand::Immediate), None, None],
                op_size: Some(OperandSize::DoubleWord),
            }),
            _ => Ok(Opcode {
                ident: OpcodeType::Unknown,
                operands: [None, None, None, None],
                op_size: None,
            }),
        }
    }

    /// Special function that returns results based on the read prefix. This typically, and
    /// practically implies that the Opcode will be 2 or 3-bytes long.
    /// This function does not handle REX prefixes. It is the job of the caller to do that.
    pub fn with_prefix(reader: &mut Reader, prefix: Prefix) -> Result<Self, OpcodeError> {
        // Read the first byte from the `reader`
        let first_byte = reader.read::<u8>()?;

        // Check where the first byte we read is an escaped code or not.
        match first_byte {
            // If we found an escape code, than we know that the Opcode is 2 or 3 bytes long
            opcode_prefix::ESCAPE_CODE => {
                match prefix {
                    Prefix::Group1(gr1) => {
                        match gr1 {
                            Group1::RepNE => Ok(Opcode {
                                ident: OpcodeType::Unknown,
                                operands: [None, None, None, None],
                                op_size: None,
                            }),
                            Group1::Rep => {
                                let second_byte = reader.read::<u8>()?;
                                match second_byte {
                                    // This is the byte that indicates an ENDBR
                                    0x1E => {
                                        // We have to read a 3rd byte
                                        let third_byte = reader.read::<u8>()?;
                                        match third_byte {
                                            0xFB => Ok(Opcode {
                                                ident: OpcodeType::EndBr32,
                                                operands: [None, None, None, None],
                                                op_size: None,
                                            }),
                                            0xFA => Ok(Opcode {
                                                ident: OpcodeType::EndBr64,
                                                operands: [None, None, None, None],
                                                op_size: None,
                                            }),
                                            _ => Err(OpcodeError::Invalid3ByteOpcode(
                                                    first_byte,
                                                    second_byte,
                                                    third_byte,
                                                )),
                                        }
                                    }
                                    _ => Ok(Opcode {
                                        ident: OpcodeType::Unknown,
                                        operands: [None, None, None, None],
                                        op_size: None,
                                    }),
                                }
                            }
                            _ => Err(OpcodeError::InvalidPrefix(prefix)),
                        }
                    }
                    Prefix::OpSize => Ok(Opcode {
                        ident: OpcodeType::Unknown,
                        operands: [None, None, None, None],
                        op_size: None,
                    }),
                    // If we have an escape code, any other prefix is invalid for a 2-byte, 3-byte
                    // opcode
                    _ => Err(OpcodeError::InvalidPrefix(prefix)),
                }
            }
            // If the byte is not an escape code, that means it is just a 1-byte
            // opcode, that we have to parse.
            _ => Self::from_byte(first_byte),
        }
    }
}

mod opcode_prefix {
    pub const ESCAPE_CODE: u8 = 0x0F;

}
