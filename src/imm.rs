//! Specifies the Displacement and Immediate rules and parsing mechanism

/// The "displacement" is just a constant that gets added to the rest of the address. Examples
/// include:
/// - [reg + displacement]
/// - [displacmeent]
/// - [reg * constant + displacement]
/// Some addressing forms include a displacement immediately following the ModR/M byte (or the SIB
/// byte if one is present). If a displacement is required, it can be 1, 2, or 4 bytes.
pub struct Displacement;

/// If an instruction specifies an immediate operand, the operand always follows any displacement
/// bytes. An immediate operand can be 1, 2 or 4 bytes
pub struct Immediate;
