//! Represents the ModR/M and SIB bytes parsing

/// Made up of three parts:
/// - R/M, bits[0:3]
/// - Reg/Opcode, bits[3:6]
/// - Mod, bits[6:8]
///
/// The `mod` field combines with the r/m field to form 32 possible values: eight registers and 24
/// addressing modes.
/// The `reg/opcode` field specifies either a register number or three more bits of opcode
// TODO: We need to parse addresses based on 16-bit and 32-bit addressing forms.
/// information. The purpose of the `reg/opcode` field is specified in the primary opcode.
/// The `r/m` field can specify a register as an operand or it can be combined with the mod field
/// to encode an addressing mode
pub struct ModRM(u8);


/// Made up of also 3 parts:
/// - Base, bits[0:3], specifies the register number of the base register.
/// - Index, bits[3:6], specifies the register number of the index register.
/// - Scale, bits[6:8], specifies the scale factor.
///
/// Certain encodings of the ModR/M byte require a second addressing byte (the SIB byte). The
/// base-plus-index and scale-plus-index forms of 32-bit addressing require the SIB byte.
pub struct SIB(u8);
