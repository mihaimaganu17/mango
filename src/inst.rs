use crate::{
    imm::{DispArch, DispError, Displacement, ImmError, Immediate},
    modrm::{Arch, EffAddrType, ModRM, Sib, Sib32, Sib64},
    opcode::{
        AddrSize, OpSize, Opcode, OpcodeError, OpcodeType, Operand, OperandEncoding, RegFieldExt,
        RegFieldExtError,
    },
    prefix::Prefix,
    reader::{Reader, ReaderError},
    reg::{Reg, RegFamily, SegmentRegister},
    rex::Rex,
};
use core::fmt;

#[derive(Debug)]
pub struct Instruction {
    // Optional prefix that can alter the instruction behaviour or can be specified to give a
    // different instruction.
    prefixs: Vec<Prefix>,
    // Optional REX prefix, used to specify that the instruction needs and can be used in 64-bit
    // mode
    rex: Option<Rex>,
    // 1, 2, or 3-byte sequence that identifies the instruction type
    pub opcode: Opcode,
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
    pub operands: InstOperands,
}

#[derive(Debug)]
pub struct InstOperands {
    op_size: OpSize,
    operands: [Option<ResolvedOperand>; 4],
}

impl fmt::Display for InstOperands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut formatted_operands = String::new();
        for (idx, maybe_operand) in self.operands.iter().enumerate() {
            if let Some(operand) = maybe_operand {
                // If our operand is a memory operand, we have to specify its size
                if let ResolvedOperand::Mem((eff_addr, _, _)) = operand {
                    match self.op_size {
                        OpSize::U8 | OpSize::I8 => write!(f, "BYTE ")?,
                        OpSize::U16 | OpSize::I16 => write!(f, "WORD ")?,
                        OpSize::U32 | OpSize::I32 => write!(f, "DWORD ")?,
                        OpSize::U64 | OpSize::I64 => write!(f, "QWORD ")?,
                        OpSize::CpuMode => write!(f, "CpuMode ")?,
                    };
                    write!(f, "PTR ");
                }
                if idx == 0 {
                    formatted_operands = format!("{operand}");
                } else {
                    formatted_operands = format!("{formatted_operands}, {operand}");
                }
            }
        }

        write!(f, "{}", formatted_operands)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ResolvedOperand {
    Immediate(Immediate),
    Reg(Reg),
    Segment(SegmentRegister),
    Mem((EffAddrType, Option<Sib>, Option<Displacement>)),
    ToBeDecided,
}

impl fmt::Display for ResolvedOperand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolvedOperand::Immediate(imm) => write!(f, "{}", imm),
            ResolvedOperand::Reg(reg) => write!(f, "{}", reg), 
            ResolvedOperand::Segment(seg_reg) => write!(f, "{}", seg_reg),
            ResolvedOperand::Mem((eff_addr, maybe_sib, maybe_disp)) => {
                write!(f, "[");
                match eff_addr {
                    EffAddrType::Reg(reg) => write!(f, "{}", reg),
                    EffAddrType::Sib | EffAddrType::None => {
                        if let Some(sib) = maybe_sib {
                            write!(f, "{}", sib)?;
                            if maybe_disp.is_some() && !sib.is_empty() {
                                write!(f, "+")?;
                            }
                        }
                        if let Some(disp) = maybe_disp {
                            write!(f, "{}", disp)?;
                        }
                        Ok(())
                    }
                };
                write!(f, "]")
            }
            ResolvedOperand::ToBeDecided => write!(f, "UNKNOWN"),
        }
    }
}

pub trait SizedOperand {
    fn size(&self) -> OpSize;
}

impl SizedOperand for ResolvedOperand {
    fn size(&self) -> OpSize {
        match self {
            ResolvedOperand::Immediate(imm) => imm.size(),
            ResolvedOperand::Reg(reg) => reg.size(),
            ResolvedOperand::Mem((eff_addr, maybe_sib, _)) => {
                let eff_addr_size = eff_addr.size();
                match eff_addr_size {
                    OpSize::CpuMode => {
                        let sib_size = if let Some(sib) = maybe_sib {
                            sib.size()
                        } else {
                            eff_addr_size
                        };
                        sib_size
                    }
                    _ => eff_addr_size,
                }
            }
            _ => OpSize::CpuMode,
        }
    }
}

impl Instruction {
    pub fn from_reader(
        reader: &mut Reader,
        maybe_arch: Option<Arch>,
    ) -> Result<Self, InstructionError> {
        // We assume that there is no prefix
        let mut prefixs = vec![];
        // We also assume that there is not REX prefix
        let mut maybe_rex = None;
        // Declare the default CPU mode
        let cpu_mode = match maybe_arch {
            Some(arch) => arch,
            None => Arch::Arch32,
        };

        // Try and parse the byte as an Opcode
        let mut first_opcode = Opcode::with_prefix_arch(reader, &prefixs, cpu_mode)?;

        let mut prefix_idx = 0;
        while let OpcodeType::Prefix(op_prefix) = first_opcode.ident {
            prefixs.push(op_prefix);
            first_opcode = Opcode::with_prefix_arch(reader, &prefixs, cpu_mode)?;
            prefix_idx += 1;

            if prefix_idx == 3 {
                break;
            }
        }

        // We may not need this anymore
        let second_opcode = if let OpcodeType::Prefix(_) = first_opcode.ident {
            Opcode::with_prefix_arch(reader, &prefixs, cpu_mode)?
        } else {
            first_opcode
        };

        // At this point we know that the second opcode cannot be a normal prefix.
        // However, it can be a REX prefix, so we also want to check for that
        let mut third_opcode = match second_opcode.ident {
            // If we got a rex prefix, we read again the next opcode
            OpcodeType::Rex(op_rex) => {
                // Initialize our own REX
                maybe_rex = Some(op_rex);
                Opcode::with_prefix_arch(reader, &prefixs, cpu_mode)?
            }
            _ => second_opcode,
        };

        // Save the ident in a local variable
        let ident = third_opcode.ident;


        // We need to filter the opcode, yet again to check if we need an extension from the
        // ModRM byte, which is the next byte
        if let OpcodeType::NeedsModRMExtension(_) = ident {
            // We just peak the modrm byte
            let modrm_byte = reader.peek::<u8>()?;

            // Get the reg part from the ModRM byte
            let reg = (modrm_byte >> 3) & 0b111;

            third_opcode.convert_with_ext_arch(RegFieldExt::try_from(reg)?, cpu_mode)?;
        }

        let modrm_encodings = [
            OperandEncoding::M,
            OperandEncoding::MI,
            OperandEncoding::MR,
            OperandEncoding::RM,
        ];

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
                let mut modrm = ModRM::from_byte_with_arch(modrm_byte, maybe_arch, maybe_rex);

                // Based on the addressing mode of the CPU, we have to/or not read the SIB byte
                if let Some(arch) = maybe_arch {
                    match arch {
                        // If we have a 32-bit or 64-bit addressing mode, there is a possibility
                        // that we have a SIB byte
                        Arch::Arch32 => {
                            if modrm.1.has_sib() {
                                let sib_byte = reader.read::<u8>()?;
                                let mut sib = Sib::Sib32(Sib32::from(sib_byte));
                                // We know that we have a SIB, so we must take care now of how we
                                // compute the effective address
                                if modrm.1.mod_bits() == 0b00 {
                                    if let Some(Reg::EBP) = sib.base() {
                                        sib.set_base(None);
                                        modrm.1.set_displacement(Some(DispArch::Bit32));
                                    }
                                }

                                maybe_sib = Some(sib);
                            }
                        }
                        Arch::Arch64 => {
                            if modrm.1.has_sib() {
                                let sib_byte = reader.read::<u8>()?;
                                let mut sib =
                                    Sib::Sib64(Sib64::from_byte_with_rex(sib_byte, maybe_rex));
                                // We know that we have a SIB, so we must take care now of how we
                                // compute the effective address
                                if modrm.1.mod_bits() == 0b00 {
                                    if let Some(Reg::RBP) = sib.base() {
                                        sib.set_base(None);
                                        modrm.1.set_displacement(Some(DispArch::Bit32));
                                    }
                                }

                                maybe_sib = Some(sib);
                            } else {
                                // If we do not have a sib, then we must augment the `Reg` from
                                // the ModRM byte with the REX.B value
                            }
                        }
                        _ => maybe_sib = None,
                    };
                }

                if let Some(disp_arch) = modrm.1.displacement() {
                    let displacement = disp_arch.read(reader)?;
                    maybe_disp = Some(displacement);
                } else {
                }

                maybe_modrm = Some(modrm);
            }
        }

        // Initialize the immediate value
        let mut maybe_imm = None;

        // Search if there are any immediates in the operands
        let mut resolved_operands: [Option<ResolvedOperand>; 4] = [None; 4];

        // We need to take into consideration the Operand Size override prefix, when resolving
        // the operands. This switches the size of the operand depending on the CPU mode and
        // also the REX prefix
        let mut op_size_override = OpSize::from_cpu_opcode(cpu_mode, third_opcode.ident);

        // We also need to take into consideration the AddressSize override prefix, when
        // resolving operands which refer to memory.
        let mut addr_size_override = AddrSize::from(cpu_mode);

        for (idx, op) in third_opcode.operands.iter().enumerate() {
            // We just ignore operands which are `None`
            if op.is_none() {
                continue;
            }

            if prefixs.contains(&Prefix::OpSize) {
                op_size_override = match cpu_mode {
                    // If we are in 16-bit mode, we use 32-bit operand size
                    Arch::Arch16 => OpSize::U32,
                    // If we are in 32-bit mode, we use 16-bit operand size
                    Arch::Arch32 => OpSize::U16,
                    // If we are in 64-bit mode, we use 16-bit operand size, however, the prefix
                    // is ignored if there is a REX prefix with the field REX.X = 1 set.
                    Arch::Arch64 => OpSize::U16,
                }
            }
            if prefixs.contains(&Prefix::AddrSize) {
                addr_size_override = match cpu_mode {
                    // If we are in 16-bit mode, we use 32-bit operand size
                    Arch::Arch32 | Arch::Arch64 => AddrSize::Addr32Bit,
                    _ => panic!("Instruction is illegal with the prefix"),
                }
            }

            // If we have a prefix, with the REX.X = 1 field set, the operand override prefix is
            // ignored
            if let Some(rex) = maybe_rex {
                if rex.w() == 1 {
                    op_size_override = OpSize::U64;
                }
            }

            let overridable_op_size = [OpSize::CpuMode, OpSize::U16, OpSize::U32, OpSize::U64];
            let overridable_addr_size = [AddrSize::Addr64Bit];

            match op {
                Some(Operand::Immediate(op_size)) => {
                    let mut imm = match overridable_op_size.contains(op_size) {
                        true => Immediate::parse(&op_size_override, reader)?,
                        false => Immediate::parse(op_size, reader)?,
                    };
                    // We check the size of the last operand, if it was smaller, we extend our
                    // immediate
                    if idx > 0 {
                        if let Some(res_op) = resolved_operands[idx - 1] {
                            let previous_op_size = res_op.size();
                            if previous_op_size > imm.size() {
                                imm = imm.convert_with_opsize(previous_op_size);
                            }
                        }
                    }
                    resolved_operands[idx] = Some(ResolvedOperand::Immediate(imm));
                }
                // Handle the family
                Some(Operand::RegInOpcode(opcode_byte)) => {
                    let lower_3bits = opcode_byte & 0b111;
                    let reg_64bit_encoding = if let Some(rex) = maybe_rex {
                        lower_3bits | (rex.b() << 3)
                    } else {
                        lower_3bits
                    };
                    // Unfortunately, depending on the size and the default mode, we can only
                    // encode certain types of operands depending on architecture
                    let arch = match (cpu_mode, op_size_override) {
                        (Arch::Arch16, OpSize::U16)
                        | (Arch::Arch32, OpSize::U16)
                        | (Arch::Arch64, OpSize::U16) => Arch::Arch16,
                        (Arch::Arch32, OpSize::U16) => Arch::Arch16,
                        (Arch::Arch32, OpSize::U32) => Arch::Arch32,
                        (Arch::Arch64, OpSize::U32) | (Arch::Arch64, OpSize::U64) => Arch::Arch64,
                        (_, _) => Arch::Arch64,
                    };
                    let reg_family = RegFamily::from(reg_64bit_encoding);
                    let reg = reg_family.reg_from_arch(&arch);
                    resolved_operands[idx] = Some(ResolvedOperand::Reg(reg));
                }
                Some(Operand::RegFamily(family)) => {
                    let reg = family.reg_from(&op_size_override);
                    resolved_operands[idx] = Some(ResolvedOperand::Reg(reg));
                }
                Some(Operand::Segment(seg_reg)) => {
                    resolved_operands[idx] = Some(ResolvedOperand::Segment(*seg_reg))
                }
                Some(Operand::Reg(reg)) => {
                    resolved_operands[idx] = Some(ResolvedOperand::Reg(*reg))
                }
                Some(Operand::ModRM(op_size, addr_size)) => {
                    let mut modrm = maybe_modrm
                        .as_mut()
                        .ok_or(InstructionError::InvalidModRMError)?;
                    if modrm.mod_bits() == 0b11 {
                        let reg = modrm.rm_reg().ok_or(InstructionError::InvalidModRMError)?;
                        let reg = match overridable_op_size.contains(op_size) {
                            true => reg.convert_with_opsize(&op_size_override),
                            false => reg.convert_with_opsize(op_size),
                        };
                        resolved_operands[idx] = Some(ResolvedOperand::Reg(reg));
                    } else {
                        let mem = modrm.rm_mem();
                        let mem = match overridable_addr_size.contains(addr_size) {
                            true => {
                                let eff_addr = mem.convert_with_addrsize(addr_size_override);
                                let sib = if let Some(inner_sib) = maybe_sib {
                                    Some(inner_sib.convert_with_addrsize(addr_size_override))
                                } else {
                                    None
                                };
                                (eff_addr, sib, maybe_disp)
                            }
                            false => {
                                let eff_addr = mem.convert_with_addrsize(*addr_size);
                                let sib = if let Some(inner_sib) = maybe_sib {
                                    Some(inner_sib.convert_with_addrsize(*addr_size))
                                } else {
                                    None
                                };
                                (eff_addr, sib, maybe_disp)
                            }
                        };

                        resolved_operands[idx] = Some(ResolvedOperand::Mem(mem));
                    }
                }
                Some(Operand::ModReg(op_size)) => {
                    let modrm = maybe_modrm
                        .as_ref()
                        .ok_or(InstructionError::InvalidModRMError)?;
                    let reg = modrm.reg();
                    let reg = match overridable_op_size.contains(&op_size) {
                        true => reg.convert_with_opsize(&op_size_override),
                        false => reg.convert_with_opsize(&op_size),
                    };
                    resolved_operands[idx] = Some(ResolvedOperand::Reg(reg));
                }
                _ => resolved_operands[idx] = Some(ResolvedOperand::ToBeDecided),
            };
        }

        Ok(Instruction {
            prefixs,
            rex: maybe_rex,
            opcode: third_opcode,
            modrm: maybe_modrm,
            sib: maybe_sib,
            disp: maybe_disp,
            imm: maybe_imm,
            operands: InstOperands { operands: resolved_operands, op_size: op_size_override},
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
    InvalidModRMError,
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
