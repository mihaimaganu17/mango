use crate::{
    opcode::{Operand, Opcode, OpcodeType, OpcodeError, OperandEncoding, OpSize, RegFieldExt, RegFieldExtError},
    prefix::Prefix,
    rex::Rex,
    reg::Reg,
    reader::{Reader, ReaderError},
    modrm::{Arch, ModRM, Sib, Sib32, Sib64},
    imm::{Displacement, DispError, Immediate, ImmError},
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
    // Certain encodings of the ModRM require an extra `Scale-Index-Base` to compute the address.
    // This is reffered to as the Sib byte, which comes after the ModRM byte if is required.
    sib: Option<Sib>,
    // Certain addressing needs additional data, encoded as a displacement which follows the ModRM
    // or the SIB byte
    disp: Option<Displacement>,
    // Number are stored as immediates in an opcode, and there are instruction which also encode
    // them as a 1, 2, 4 or in rare cases 8 bytes.
    imm: Option<Immediate>,
    // After gathering all the required information about parsing the instruction, we need to
    // resolve to the actual operands of the instruction
    operands: Vec<ResolvedOperand>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResolvedOperand {
    Immediate(Immediate),
    Reg(Reg),
    ToBeDecided,
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
        // Declare the default CPU mode
        let cpu_mode = match maybe_arch {
            Some(arch) => arch,
            None => Arch::Arch32,
        };

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

        let modrm_encodings = [OperandEncoding::MI(RegFieldExt::try_from(6)?), OperandEncoding::MR, OperandEncoding::RM, OperandEncoding::ZO];

        // Initialize the ModRM field
        let mut maybe_modrm = None;
        // Initialize the SIB byte
        let mut maybe_sib = None;
        // Initialize the Displacement
        let mut maybe_disp = None;

        if let Some(encoding) = third_opcode.encoding {
            if modrm_encodings.contains(&encoding) {
                // We read the modrm byte
                let modrm_byte = reader.read::<u8>()?;

                // Parse the ModRM byte
                let modrm = ModRM::from_byte_with_arch(modrm_byte, maybe_arch, maybe_rex);


                // Based on the addressing mode of the CPU, we have to/or not read the SIB byte
                if let Some(arch) = maybe_arch {
                    match arch {
                        // If we have a 32-bit or 64-bit addressing mode, there is a possibility
                        // that we have a SIB byte
                        Arch::Arch32 => {
                            if modrm.1.has_sib() {
                                let sib_byte = reader.read::<u8>()?;
                                maybe_sib = Some(Sib::Sib32(Sib32::from(sib_byte)));
                            }
                        }
                        Arch::Arch64 => {
                            if modrm.1.has_sib() {
                                let sib_byte = reader.read::<u8>()?;
                                maybe_sib = Some(Sib::Sib64(Sib64::from_byte_with_rex(sib_byte, maybe_rex)));
                            }
                        }
                        _ => maybe_sib = None,
                    };
                }

                if let Some(disp_arch) = modrm.1.displacement() {
                    let displacement = disp_arch.read(reader)?;
                    maybe_disp = Some(displacement);
                }

                maybe_modrm = Some(modrm);
            }
        }

        // Initialize the immediate value
        let mut maybe_imm = None;

        // Construct an iterator over all the initialized operands
        let op_iter = third_opcode.operands.iter().filter(|op| op.is_some()).map(|op| op.as_ref().unwrap());

        // Search if there are any immediates in the operands
        let resolved_operands = op_iter.map(|op| {
            // We need to take into consideration the Operand Size override prefix, when resolving
            // the operands. This switches the size of the operand depending on the CPU mode and
            // also the REX prefix
            let mut op_size_override = OpSize::from(cpu_mode);

            if let Some(Prefix::OpSize) = maybe_prefix {
                op_size_override = match cpu_mode {
                    // If we are in 16-bit mode, we use 32-bit operand size
                    Arch::Arch16 => OpSize::U32,
                    // If we are in 32-bit mode, we use 16-bit operand size 
                    Arch::Arch32 => OpSize::U16,
                    // If we are in 64-bit mode, we use 16-bit operand size, however, the prefix
                    // is ignored if there is a REX prefix with the field REX.X = 1 set.
                    Arch::Arch64 => OpSize::U16,
                }
            };

            // If we have a prefix, with the REX.X = 1 field set, the operand override prefix is
            // ignored
            if let Some(rex) = maybe_rex {
                if rex.x() == 1 {
                    op_size_override = OpSize::U64;
                }
            }

            let overridable_op_size = [OpSize::U16, OpSize::U32, OpSize::U64];

            match op {
                Operand::Immediate(op_size) => {
                    let imm = match overridable_op_size.contains(op_size) { 
                        true => Immediate::parse(&op_size_override, reader).expect("Cannot parse immediate"),
                        false => Immediate::parse(op_size, reader).expect("Cannot parse immediate"),
                    };
                    ResolvedOperand::Immediate(imm)
                }
                Operand::RegFamily(family) => {
                    let reg = family.reg_from(&op_size_override);
                    ResolvedOperand::Reg(reg)
                }
                Operand::Reg(reg) => ResolvedOperand::Reg(*reg),
                _ => ResolvedOperand::ToBeDecided,
            }
        }).collect::<Vec<_>>();
 
        Ok(Instruction {
            prefix: maybe_prefix,
            rex: maybe_rex,
            opcode: third_opcode,
            modrm: maybe_modrm,
            sib: maybe_sib,
            disp: maybe_disp,
            imm: maybe_imm,
            operands: resolved_operands,
        })
    }
}

/// Issues errors for instruction parsing
#[derive(Debug)]
pub enum InstructionError {
    OpcodeError(OpcodeError),
    ReaderError(ReaderError),
    RegFieldExtError(RegFieldExtError),
    DispError(DispError),
    ImmError(ImmError),
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

impl From<RegFieldExtError> for InstructionError {
    fn from(err: RegFieldExtError) -> Self {
        InstructionError::RegFieldExtError(err)
    }
}

impl From<DispError> for InstructionError {
    fn from(err: DispError) -> Self {
        InstructionError::DispError(err)
    }
}

impl From<ImmError> for InstructionError {
    fn from(err: ImmError) -> Self {
        InstructionError::ImmError(err)
    }
}
