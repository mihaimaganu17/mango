//! Module that takes care of parsing the Opcode field in an instruction.
use crate::{
    prefix::{Prefix, Group1},
    reader::{Reader, ReaderError},
    reg::Reg,
    imm::Immediate,
    modrm::ModRM,
    rex::Rex,
};

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
pub enum OpcodeIdent {
    // Terminate an indirect branch in 32 bit and compatibility mode.
    EndBr32,
    // Terminate an indirect branch in 64 bit mode.
    EndBr64,
    Xor(Operand, Operand),
    Unknown,
}

#[derive(Debug)]
pub enum Operand {
    Reg,
    Immediate,
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

impl OpcodeIdent {
    /// Parse the next `Opcode` from the `reader`, given the prefix. We need to pass the `reader`
    /// to this function, since we do not know if the opcode is 1, 2 or 3 bytes
    pub fn from_reader(reader: &mut Reader) -> Result<Self, OpcodeError> {
        // Read one byte
        let byte = reader.read::<u8>()?;

        // Try to parse a prefix from it
        let prefix = Prefix::from_byte(byte);

        // Next, we check if we actually read a prefix, or not and we update the next byte we 
        // have to parse, accordingly
        let maybe_rex_byte = match prefix {
            // If there is no prefix, the first byte is actually the one we just read
            Prefix::None => byte,
            // If there is a prefix, we read another byte
            _ => reader.read::<u8>()?,
        };

        // Check if our the byte we read above is actually a `Rex` prefix
        let maybe_rex = Rex::from_byte(maybe_rex_byte);

        // Now based, on the fact that we got a REX prefix, we either
        // a. Read another byte
        // b. Use the last byte we read as the current byte
        let first_byte = match maybe_rex {
            // If we actually got a REX prefix, we just fetch the next byte
            Some(rex) => reader.read::<u8>()?,
            // If not, the current byte is actually the next opcode
            None => maybe_rex_byte,
        };

        // We keep a check that we can easily reference forwards. This match may be merged into the
        // match below
        let is_escape_code = match first_byte {
            opcode_prefix::ESCAPE_CODE => true,
            _ => false,
        };

        match is_escape_code {
            true => {
                match prefix {
                    Prefix::Group1(gr1) => {
                        match gr1 {
                            Group1::RepNE => Ok(OpcodeIdent::Unknown),
                            Group1::Rep => {
                                let second_byte = reader.read::<u8>()?;
                                match second_byte {
                                    // This is the byte that indicates an ENDBR
                                    0x1E => {
                                        // We have to read a 3rd byte
                                        let third_byte = reader.read::<u8>()?;
                                        match third_byte {
                                            0xFB => Ok(OpcodeIdent::EndBr32),
                                            0xFA => Ok(OpcodeIdent::EndBr64),
                                            _ => Err(OpcodeError::Invalid3ByteOpcode(
                                                    first_byte,
                                                    second_byte,
                                                    third_byte,
                                                )),
                                        }
                                    }
                                    _ => Ok(OpcodeIdent::Unknown),
                                }
                            }
                            _ => Err(OpcodeError::InvalidPrefix(prefix)),
                        }
                    }
                    Prefix::OpSize => Ok(OpcodeIdent::Unknown),
                    // If we have an escape code, any other prefix is invalid for a 2-byte, 3-byte
                    // opcode
                    _ => Err(OpcodeError::InvalidPrefix(prefix)),
                }
            }
            false => match first_byte {
                // XOR opcodes
                0x31 => {
                    let mod_rm_byte = reader.read::<u8>()?;
                    let mod_rm = ModRM::from_opcode_reg(mod_rm_byte, None);
                    println!("{:x?}", mod_rm);
                    Ok(OpcodeIdent::Xor(Operand::Immediate, Operand::Immediate))
                }
                _ => Ok(OpcodeIdent::Unknown),
            }
        }
    }
}

mod opcode_prefix {
    pub const ESCAPE_CODE: u8 = 0x0F;

}
