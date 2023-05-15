use crate::{
    opcode::{Operand, Opcode, OpcodeType, OpcodeError},
    prefix::Prefix,
    rex::Rex,
    reader::{Reader, ReaderError},
    modrm::{Arch, ModRM},
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
    // operands: [Option<Operand>; 4],
    // The encoding, describes the type of operands, their sizes, location and how they are used in
    // the instruction
    //encoding: Encoding,
    // The addressing mode, used by the processor
    modrm: Option<ModRM>,
}

impl Instruction {
    pub fn from_reader(
        reader: &mut Reader,
        maybe_arch: Option<Arch>,
    ) -> Result<Self, InstructionError> {
        // We assume that there is no prefix
        let mut maybe_prefix = None;
        // We also assume that there is not REX prefix
        let mut maybe_rex = None;

        // Try and parse the byte as an Opcode
        let first_opcode = Opcode::from_reader(reader)?;

        // Based on wheather we have a prefix or not, we read the second opcode.
        let second_opcode = match first_opcode.ident {
            // If we got a prefix, try and parse the next bytes, taking into acount that we have a
            // prefix
            OpcodeType::Prefix(op_prefix) => {
                maybe_prefix = Some(op_prefix);
                Opcode::with_prefix(reader, op_prefix)?
            }
            _ => first_opcode
        };

        // At this point we know that the second opcode cannot be a normal prefix.
        // However, it can be a REX prefix, so we also want to check for that
        let third_opcode = match second_opcode.ident {
            // If we got a rex prefix, we read again the next opcode
            OpcodeType::Rex(op_rex) => {
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

        let modrm = match third_opcode.operands.contains(&Some(Operand::ModRM)) {
            true => {
                // We read the modrm byte
                let modrm_byte = reader.read::<u8>()?;

                // We parse it
                Some(ModRM::from_byte_with_arch(modrm_byte, maybe_arch, maybe_rex))
            }
            false => None,
        };

        
        Ok(Instruction {
            prefix: maybe_prefix,
            rex: maybe_rex,
            opcode: third_opcode,
            modrm: modrm,
        })
    }
}

/// Issues errors for instruction parsing
#[derive(Debug)]
pub enum InstructionError {
    OpcodeError(OpcodeError),
    ReaderError(ReaderError),
}

impl From<OpcodeError> for InstructionError {
    fn from(err: OpcodeError) -> Self {
        InstructionError::OpcodeError(err)
    }
}

impl From<ReaderError> for InstructionError {
    fn from(err: ReaderError) -> Self {
        InstructionError::ReaderError(err)
    }
}
