use crate::{
    opcode::{Opcode, OpcodeError},
    prefix::Prefix,
    rex::Rex,
    reader::Reader,
};

#[derive(Debug)]
pub struct Instruction {
    // Optional prefix that can alter the instruction behaviour or can be specified to give a
    // different instruction.
    prefix: Option<Prefix>,
    // Optional REX prefix, used to specify that the instruction needs and can be used in 64-bit
    // mode
    rex: Option<Rex>,
    // 1, 2, or 3-byte sequence that identifies the instruction type
    opcode: Opcode,
    // A list of, maximum 4 operands, or a minumum of 0 operands that are used by the instruction.
    //operands: [Option<Operand>; 4],
    // The encoding, describes the type of operands, their sizes, location and how they are used in
    // the instruction
    //encoding: Encoding,
    // The addressing mode, used by the processor
    //mode: Mode,
}

impl Instruction {
    pub fn from_reader(reader: &mut Reader) -> Result<Self, InstructionError> {
        // We assume that there is no prefix
        let mut maybe_prefix = None;
        // We also assume that there is not REX prefix
        let mut maybe_rex = None;

        // Try and parse the byte as an Opcode
        let first_opcode = Opcode::from_reader(reader)?;

        // Based on wheather we have a prefix or not, we read the second opcode.
        let second_opcode = match first_opcode {
            // If we got a prefix, try and parse the next bytes, taking into acount that we have a
            // prefix
            Opcode::Prefix(op_prefix) => {
                maybe_prefix = Some(op_prefix);
                Opcode::with_prefix(reader, op_prefix)?
            }
            _ => first_opcode
        };

        // At this point we know that the second opcode cannot be a normal prefix.
        // However, it can be a REX prefix, so we also want to check for that
        let third_opcode = match second_opcode {
            // If we got a rex prefix, we read again the next opcode
            Opcode::Rex(op_rex) => {
                // Initialize our own REX
                maybe_rex = Some(op_rex);
                 
                // At this point we need to take into acount if we do have a prefix or not. This is
                // because the prefix can change the opcode and the instruction
                match maybe_prefix {
                    Some(prefix) => Opcode::with_prefix(reader, prefix)?, 
                    None => Opcode::from_reader(reader)?,
                }
            }
            _ => second_opcode,
        };
        
        Ok(Instruction {
            prefix: maybe_prefix,
            rex: maybe_rex,
            opcode: third_opcode,
        })
    }
}

/// Issues errors for instruction parsing
#[derive(Debug)]
pub enum InstructionError {
    OpcodeError(OpcodeError),
}

impl From<OpcodeError> for InstructionError {
    fn from(err: OpcodeError) -> Self {
        InstructionError::OpcodeError(err)
    }
}
