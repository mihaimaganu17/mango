/// REX prefixes are instruction-prefix bytes used in 64-bit mode. They do the following:
/// - Specify GPRs and SSE registers.
/// - Specify 64-bit operand size.
/// - Specifiy extended control registers.
/// This type of prefix is necessary only if an instruction references one of the extended
/// registers or uses a 64-bit operand. If a REX prefix is used when it has no meaning, it is
/// ignored.
///
/// In 64-bit mode, two groups of instructions have default operand size of 64bits(Do not need a
/// REX prefix for this operans size). These are:
/// - Near branches
/// - All instructions, except far branches that implicitly, reference the RSP.
///
/// REX prefixes are a set of 16 opcode that span one row of the opcode map and occupy entries
/// 0x40 to 0x4F. These opcodes represent VALID INSTRUCTIONS(INC or DEC) in IA-32 operating modes
/// and in compatibility mode.
/// In 64-bit mode, the same opcodes represent the instruction prefix REX and are not treated as
/// individual instructions.
/// The single-byte-opcode forms of the INC/DEC instructions are not available in 64-bit mode.
/// INC/DEC functionality is still available using ModR/M forms of the same instructions
/// (opcodes FF/0 and FF/1)
/// The bits in position [4:8] are always 0100
pub struct Rex {
    // This value represents a single bit, with the following values:
    // - 0: Operand size determined by CS.d(either 16-bit or 32-bit).
    // - 1: 64-bit Operand size
    // Used to determine the operand size but does not solely dertemine operand width. Specifying
    // 64-bit operand size override has no effect on byte-specific operations.
    //
    // For non-byte operations: if a 0x66 prefix is used with prefix (REX.W = 1), 0x66 is ignored.
    // If a 0x66 ovveride is used with REX and REX.W = 0, the operand size is 16 bits.
    w: u8,
    // This value represents a single bit and is an Extension of the ModR/M reg field.
    // Modifies the ModR/M reg field when that field encodes a GPR, SSE, control or debug register
    // It is ignored when ModR/M specifies other registers or defines an extended opcode.
    r: u8,
    // This value represents a single bit and is an Extension of the SIB index field.
    x: u8,
    // This value represents a single bit and is an Extension of the ModR/M r/m fiels, SIB base
    // field, or Opcode reg field.
    b: u8,
}

impl Rex {
    pub fn from_byte(value: u8) -> Option<Rex> {
        match value {
            // This is the span of the REX prefixes as of March 2023 Intel Manual
            0x40..=0x4F => {
                // Fetch the needed prefix bits
                let w = (value >> 3) & 1;
                let r = (value >> 2) & 1;
                let x = (value >> 1) & 1;
                let b = value & 1;

                Some(Rex { w, r, x, b})
            }
            _ => None,
        }
    }
}
