pub struct Instruction {
    // Optional prefix that can alter the instruction behaviour or can be specified to give a
    // different instruction.
    prefix: Prefix,
    // Optiona REX prefix, used to specify that the instruction needs and can be used in 64-bit
    // mode
    rex: Rex,
    // 1, 2, or 3-byte sequence that identifies the instruction type
    opcode: Opcode,
    // A list of, maximum 4 operands, or a minumum of 0 operands that are used by the instruction.
    operands: [Option<Operand>; 4],
    // The encoding, describes the type of operands, their sizes, location and how they are used in
    // the instruction
    encoding: Encoding,
    // The addressing mode, used by the processor
    mode: Mode,
}

impl Instruction {
    pub fn from_reader(reader: &mut Reader) -> Result<Self, InstructionError> {
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

    }
}
