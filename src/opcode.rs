//! Module that takes care of parsing the Opcode field in an instruction.

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
/// Three-bytes opcdoe formats are just like above, but instead of 1 bytes following the escape
/// code, there are 2 bytes
pub struct Opcode;

mod opcode_prefix {
    pub const TWO_BYTE_ESCAPE_CODE: u8 = 0x0F;
}
