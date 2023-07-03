//! Module that takes care of parsing the Opcode field in an instruction.
use crate::{
    modrm::Arch,
    prefix::{Group1, Prefix},
    reader::{Reader, ReaderError},
    reg::{Accumulator, Gpr, Reg, RegFamily, SegmentRegister},
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpcodeType {
    // A prefix byte for special operations or extending the instruction encoding
    Prefix(Prefix),
    // A REX prefix used to configure 64-bit mode operations
    Rex(Rex),
    Add,
    Or,
    Adc,
    Sbb,
    And,
    Sub,
    Cmp,
    Lea,
    Inc,
    Dec,
    CallNear,
    CallFar,
    JmpNear,
    JmpFar,
    Push,
    // A bitwise XOR between 2 operands
    Xor,
    // The opcode alone is not enough and it needs an Extension from a ModRM field
    NeedsModRMExtension(u8),
    // Terminate an indirect branch in 32 bit and compatibility mode.
    EndBr32,
    // Terminate an indirect branch in 64 bit mode.
    EndBr64,
    // Specifies and unknown opcode
    Unknown,
}

#[derive(Debug)]
pub struct Opcode {
    pub ident: OpcodeType,
    pub operands: [Option<Operand>; 4],
    pub encoding: Option<OperandEncoding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMethod {
    // A ModR/M byte follows the opcode and specifies the operand. The operand is either a
    // general-purpose register or a memory address.
    E,
    // The reg field of the ModR/M byte selects a general register (for example, AX (000))
    G,
    // Immediate data: the operand value is encoded in subsequent bytes of the instruction.
    I,
    // The r/m part of the ModRM byte, represents only a memory address
    M,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandType {
    // Byte, regardless of operand-size attribute.
    B,
    // Doubleword, regardless of operand-size attribute.
    D,
    // Word, doubleword or quadword (in 64-bit mode), depending on operand-size attribute
    V,
    // Word for 16-bit operand-size or doubleword for 32 or 64-bit operand-size.
    Z,
}

/// Describes the different encodings for the instruction operands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandEncoding {
    // Op1 = AL/AX/EAX/RAX, Op2 = imm8/16/32
    I,
    // Op1 = ModRM:r/m(r)
    M,
    // Op1 is encoded in the lower 3 bits of the opcode
    O,
    // Op1 = ModRM:r/m(r, w), Op2 = imm8/16/32
    MI,
    // Op1 = ModRM:r/m(r, w), Op2 = ModRM:reg(r)
    MR,
    // Op1 = ModRM:reg(r, w), Op2 = ModRM:r/m(r)
    RM,
    // Zero operators
    ZO,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct RegFieldExt(u8);

#[derive(Debug)]
pub enum RegFieldExtError {
    CannotConvertFrom(u8),
}

impl TryFrom<u8> for RegFieldExt {
    type Error = RegFieldExtError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0..=7 => Ok(Self(value)),
            _ => Err(RegFieldExtError::CannotConvertFrom(value)),
        }
    }
}

/// The operator size of the opcode is determined by 2 characteristics:
/// - The CPU Mode
/// - The OperandSize override prefix, which alternates the state between the 16-bit and the 32-bit
/// states of the CPU
/// - The Opcode identifier itself.
/// The current module, only controls the last one and the first 2 have to be addressed in the
/// `Intruction` module
/// Pay attention to the variants as their order matter, since they derive the `PartialOrd` trait
#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum OpSize {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    CpuMode,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AddrSize {
    Addr16Bit,
    Addr32Bit,
    Addr64Bit,
}

impl From<AddrSize> for OpSize {
    fn from(value: AddrSize) -> OpSize {
        match value {
            AddrSize::Addr16Bit => OpSize::U16,
            AddrSize::Addr32Bit => OpSize::U32,
            AddrSize::Addr64Bit => OpSize::U64,
        }
    }
}

impl From<Arch> for OpSize {
    fn from(value: Arch) -> Self {
        match value {
            Arch::Arch16 => Self::U16,
            // In both 32-bit and 64-bit mode, the default operand size, is 32-bit,
            Arch::Arch32 | Arch::Arch64 => Self::U32,
        }
    }
}

impl From<Arch> for AddrSize {
    fn from(value: Arch) -> Self {
        match value {
            Arch::Arch16 => Self::Addr16Bit,
            Arch::Arch32 => Self::Addr32Bit,
            Arch::Arch64 => Self::Addr64Bit,
        }
    }
}

/// Defines a list of maximum 4 operands that can be used by an instruction.
#[derive(Debug, PartialEq, Eq)]
pub struct OperandList(Operand, Operand, Operand, Operand);

#[derive(Debug, PartialEq, Eq)]
pub enum Operand {
    // Represents a register or a memory operand found in the R/M field of ModR/M
    ModRM(OpSize, AddrSize),
    // Represents a register from the `reg` part of the ModRM field
    ModReg(OpSize),
    // The operand is embedded in the opcode
    Opcode(OpSize),
    // There is an Immediate integer following the opcode that represents the operand
    Immediate(OpSize),
    // There is a Signed Immediate integer following the opcode that represents the operand
    SignedImmediate(OpSize),
    // The operand is a specific register or a set of registers
    Reg(Reg),
    // The operand is a family of registers and reffers to General Purpose Registers
    RegFamily(RegFamily),
    // The operand is a register enclosed in the opcode
    RegInOpcode(u8),
    // The operand represents a segment selector
    Segment(SegmentRegister),
}

impl Operand {
    pub fn from_map(addr_meth: AddressingMethod, op_type: OperandType, arch: Arch) -> Self {
        let op_size = match op_type {
            OperandType::B => OpSize::U8,
            OperandType::V => OpSize::CpuMode,
            OperandType::Z => match arch {
                Arch::Arch16 => OpSize::U16,
                Arch::Arch32 | Arch::Arch64 => OpSize::U32,
            },
            OperandType::D => OpSize::U32,
        };

        match addr_meth {
            AddressingMethod::E => Operand::ModRM(op_size, AddrSize::from(arch)),
            AddressingMethod::M => Operand::ModRM(op_size, AddrSize::from(arch)),
            AddressingMethod::G => Operand::ModReg(op_size),
            AddressingMethod::I => Operand::Immediate(op_size),
        }
    }
}

#[derive(Debug)]
pub enum OpcodeError {
    ReaderError(ReaderError),
    InvalidPrefix(Prefix),
    InexistentPrefix,
    InvalidOpcode(u8),
    Invalid3ByteOpcode(u8, u8, u8),
}

impl From<ReaderError> for OpcodeError {
    fn from(err: ReaderError) -> Self {
        Self::ReaderError(err)
    }
}

// TODO: We can use arch as a generic over this fields maybe, since all of them need it
impl Opcode {
    /// Reads one byte from the passed reader and parses it
    pub fn from_reader_with_arch(reader: &mut Reader, arch: Arch) -> Result<Self, OpcodeError> {
        // Read the first byte from the `reader`
        let byte = reader.read::<u8>()?;

        Self::from_byte_with_arch(byte, arch)
    }

    /// Parse the next `Opcode` from the `reader`, given the prefix. We need to pass the `reader`
    /// to this function, since we do not know if the opcode is 1, 2 or 3 bytes
    pub fn from_byte_with_arch(byte: u8, arch: Arch) -> Result<Self, OpcodeError> {
        // We first try and parse the byte for a prefix
        let maybe_prefix = Prefix::from_byte(byte);

        // If we do get a prefix, we return and it is the caller job, to do something with it
        if let Some(prefix) = maybe_prefix {
            return Ok(Opcode {
                ident: OpcodeType::Prefix(prefix),
                operands: [None, None, None, None],
                encoding: None,
            });
        }

        // If it is not a prefix, we still need to check for a REX prefix
        let maybe_rex = Rex::from_byte(byte);

        // If we do get a REX prefix, we return and it is the caller's job to call opcode parsing
        // again for the next byte
        if let Some(rex) = maybe_rex {
            return Ok(Opcode {
                ident: OpcodeType::Rex(rex),
                operands: [None, None, None, None],
                encoding: None,
            });
        }

        // This(soon to be gigantic match) will check the byte for the appropriate instruction.
        // It is the job of this match to make sure we propagate the information upwards, that the
        // calling function needs, in order to parse the rest of the bytes
        match byte {
            // ADD opcodes
            0x00 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::from_map(AddressingMethod::E, OperandType::B, arch));
                operands[1] = Some(Operand::from_map(AddressingMethod::G, OperandType::B, arch));
                let encoding = Some(OperandEncoding::MR);
                Ok(Opcode {
                    ident: OpcodeType::Add,
                    operands,
                    encoding,
                })
            }
            0x01 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::from_map(AddressingMethod::E, OperandType::V, arch));
                operands[1] = Some(Operand::from_map(AddressingMethod::G, OperandType::V, arch));
                let encoding = Some(OperandEncoding::MR);
                Ok(Opcode {
                    ident: OpcodeType::Add,
                    operands,
                    encoding,
                })
            }
            0x02 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::from_map(AddressingMethod::G, OperandType::B, arch));
                operands[1] = Some(Operand::from_map(AddressingMethod::E, OperandType::B, arch));
                let encoding = Some(OperandEncoding::RM);
                Ok(Opcode {
                    ident: OpcodeType::Add,
                    operands,
                    encoding,
                })
            }
            0x03 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::from_map(AddressingMethod::G, OperandType::V, arch));
                operands[1] = Some(Operand::from_map(AddressingMethod::E, OperandType::V, arch));
                let encoding = Some(OperandEncoding::RM);
                Ok(Opcode {
                    ident: OpcodeType::Add,
                    operands,
                    encoding,
                })
            }
            0x04 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::Reg(Reg::AL));
                operands[1] = Some(Operand::from_map(AddressingMethod::I, OperandType::B, arch));
                let encoding = Some(OperandEncoding::I);
                Ok(Opcode {
                    ident: OpcodeType::Add,
                    operands,
                    encoding,
                })
            }
            0x05 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::RegFamily(RegFamily::Accumulator));
                operands[1] = Some(Operand::from_map(AddressingMethod::I, OperandType::Z, arch));
                let encoding = Some(OperandEncoding::I);
                Ok(Opcode {
                    ident: OpcodeType::Add,
                    operands,
                    encoding,
                })
            }
            // Push Extra Selector
            0x06 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::Segment(SegmentRegister::ES));
                let encoding = Some(OperandEncoding::ZO);
                Ok(Opcode {
                    ident: OpcodeType::Push,
                    operands,
                    encoding,
                })
            }
            // Push Code Selector
            0x0e => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::Segment(SegmentRegister::CS));
                let encoding = Some(OperandEncoding::ZO);
                Ok(Opcode {
                    ident: OpcodeType::Push,
                    operands,
                    encoding,
                })
            }
            // ADC opcodes
            0x10 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::from_map(AddressingMethod::E, OperandType::B, arch));
                operands[1] = Some(Operand::from_map(AddressingMethod::G, OperandType::B, arch));
                let encoding = Some(OperandEncoding::MR);
                Ok(Opcode {
                    ident: OpcodeType::Adc,
                    operands,
                    encoding,
                })
            }
            0x11 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::from_map(AddressingMethod::E, OperandType::V, arch));
                operands[1] = Some(Operand::from_map(AddressingMethod::G, OperandType::V, arch));
                let encoding = Some(OperandEncoding::MR);
                Ok(Opcode {
                    ident: OpcodeType::Adc,
                    operands,
                    encoding,
                })
            }
            0x12 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::from_map(AddressingMethod::G, OperandType::B, arch));
                operands[1] = Some(Operand::from_map(AddressingMethod::E, OperandType::B, arch));
                let encoding = Some(OperandEncoding::RM);
                Ok(Opcode {
                    ident: OpcodeType::Adc,
                    operands,
                    encoding,
                })
            }
            0x13 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::from_map(AddressingMethod::G, OperandType::V, arch));
                operands[1] = Some(Operand::from_map(AddressingMethod::E, OperandType::V, arch));
                let encoding = Some(OperandEncoding::RM);
                Ok(Opcode {
                    ident: OpcodeType::Adc,
                    operands,
                    encoding,
                })
            }
            0x14 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::Reg(Reg::AL));
                operands[1] = Some(Operand::from_map(AddressingMethod::I, OperandType::B, arch));
                let encoding = Some(OperandEncoding::I);
                Ok(Opcode {
                    ident: OpcodeType::Adc,
                    operands,
                    encoding,
                })
            }
            0x15 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::RegFamily(RegFamily::Accumulator));
                operands[1] = Some(Operand::from_map(AddressingMethod::I, OperandType::Z, arch));
                let encoding = Some(OperandEncoding::I);
                Ok(Opcode {
                    ident: OpcodeType::Adc,
                    operands,
                    encoding,
                })
            }
            // Push Stack Selector
            0x16 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::Segment(SegmentRegister::SS));
                let encoding = Some(OperandEncoding::ZO);
                Ok(Opcode {
                    ident: OpcodeType::Push,
                    operands,
                    encoding,
                })
            }
            // Push Data Selector
            0x1e => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::Segment(SegmentRegister::DS));
                let encoding = Some(OperandEncoding::ZO);
                Ok(Opcode {
                    ident: OpcodeType::Push,
                    operands,
                    encoding,
                })
            }
            // AND opcodes
            0x20 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::from_map(AddressingMethod::E, OperandType::B, arch));
                operands[1] = Some(Operand::from_map(AddressingMethod::G, OperandType::B, arch));
                let encoding = Some(OperandEncoding::MR);
                Ok(Opcode {
                    ident: OpcodeType::And,
                    operands,
                    encoding,
                })
            }
            0x21 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::from_map(AddressingMethod::E, OperandType::V, arch));
                operands[1] = Some(Operand::from_map(AddressingMethod::G, OperandType::V, arch));
                let encoding = Some(OperandEncoding::MR);
                Ok(Opcode {
                    ident: OpcodeType::And,
                    operands,
                    encoding,
                })
            }
            0x22 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::from_map(AddressingMethod::G, OperandType::B, arch));
                operands[1] = Some(Operand::from_map(AddressingMethod::E, OperandType::B, arch));
                let encoding = Some(OperandEncoding::RM);
                Ok(Opcode {
                    ident: OpcodeType::And,
                    operands,
                    encoding,
                })
            }
            0x23 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::from_map(AddressingMethod::G, OperandType::V, arch));
                operands[1] = Some(Operand::from_map(AddressingMethod::E, OperandType::V, arch));
                let encoding = Some(OperandEncoding::RM);
                Ok(Opcode {
                    ident: OpcodeType::And,
                    operands,
                    encoding,
                })
            }
            0x24 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::Reg(Reg::AL));
                operands[1] = Some(Operand::from_map(AddressingMethod::I, OperandType::B, arch));
                let encoding = Some(OperandEncoding::I);
                Ok(Opcode {
                    ident: OpcodeType::And,
                    operands,
                    encoding,
                })
            }
            0x25 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::RegFamily(RegFamily::Accumulator));
                operands[1] = Some(Operand::from_map(AddressingMethod::I, OperandType::Z, arch));
                let encoding = Some(OperandEncoding::I);
                Ok(Opcode {
                    ident: OpcodeType::And,
                    operands,
                    encoding,
                })
            }
            // XOR opcodes
            0x30 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::from_map(AddressingMethod::E, OperandType::B, arch));
                operands[1] = Some(Operand::from_map(AddressingMethod::G, OperandType::B, arch));
                let encoding = Some(OperandEncoding::MR);
                Ok(Opcode {
                    ident: OpcodeType::Xor,
                    operands,
                    encoding,
                })
            }
            0x31 => Ok(Opcode {
                ident: OpcodeType::Xor,
                operands: [
                    Some(Operand::ModRM(OpSize::CpuMode, AddrSize::from(arch))),
                    Some(Operand::ModReg(OpSize::CpuMode)),
                    None,
                    None,
                ],
                encoding: Some(OperandEncoding::MR),
            }),
            0x34 => Ok(Opcode {
                ident: OpcodeType::Xor,
                operands: [
                    Some(Operand::Reg(Accumulator::Reg8BitLo)),
                    Some(Operand::Immediate(OpSize::U8)),
                    None,
                    None,
                ],
                encoding: Some(OperandEncoding::I),
            }),
            0x35 => Ok(Opcode {
                ident: OpcodeType::Xor,
                operands: [
                    Some(Operand::RegFamily(RegFamily::Accumulator)),
                    Some(Operand::Immediate(OpSize::U32)),
                    None,
                    None,
                ],
                encoding: Some(OperandEncoding::I),
            }),
            // Push Opcode with general register
            0x50 | 0x51 | 0x52 | 0x53 | 0x54 | 0x55 | 0x56 | 0x57 => Ok(Opcode {
                ident: OpcodeType::Push,
                operands: [Some(Operand::RegInOpcode(byte)), None, None, None],
                encoding: Some(OperandEncoding::O),
            }),
            // Push Opcode for immediates
            0x68 => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::from_map(AddressingMethod::I, OperandType::Z, arch));
                let encoding = Some(OperandEncoding::I);
                Ok(Opcode {
                    ident: OpcodeType::Push,
                    operands,
                    encoding,
                })
            }
            0x6A => {
                let mut operands = [None, None, None, None];
                operands[0] = Some(Operand::from_map(AddressingMethod::I, OperandType::B, arch));
                let encoding = Some(OperandEncoding::I);
                Ok(Opcode {
                    ident: OpcodeType::Push,
                    operands,
                    encoding,
                })
            }
            // Immediate Group 1, which needs extension from ModRM in order to get the opcode
            0x80 | 0x81 | 0x82 | 0x83 | 0xFF => Ok(Opcode {
                ident: OpcodeType::NeedsModRMExtension(byte),
                operands: [None, None, None, None],
                encoding: None,
            }),
            // LEA
            0x8D => Ok(Opcode {
                ident: OpcodeType::Lea,
                operands: [
                    Some(Operand::ModReg(OpSize::CpuMode)),
                    Some(Operand::ModRM(OpSize::CpuMode, AddrSize::from(arch))),
                    None,
                    None,
                ],
                encoding: Some(OperandEncoding::RM),
            }),
            _ => Ok(Opcode {
                ident: OpcodeType::Unknown,
                operands: [None, None, None, None],
                encoding: None,
            }),
        }
    }

    pub fn convert_with_ext_arch(
        &mut self,
        ext: RegFieldExt,
        arch: Arch,
    ) -> Result<(), OpcodeError> {
        // We know the following extensions only have 2 operands
        match self.ident {
            OpcodeType::NeedsModRMExtension(byte) => match byte {
                0x80 => {
                    self.operands[0] =
                        Some(Operand::from_map(AddressingMethod::E, OperandType::B, arch));
                    self.operands[1] =
                        Some(Operand::from_map(AddressingMethod::I, OperandType::B, arch));
                    self.encoding = Some(OperandEncoding::MI);
                }
                0x81 => {
                    self.operands[0] =
                        Some(Operand::from_map(AddressingMethod::E, OperandType::V, arch));
                    self.operands[1] =
                        Some(Operand::from_map(AddressingMethod::I, OperandType::Z, arch));
                    self.encoding = Some(OperandEncoding::MI);
                }
                0x82 => {
                    self.operands[0] =
                        Some(Operand::from_map(AddressingMethod::E, OperandType::B, arch));
                    self.operands[1] =
                        Some(Operand::from_map(AddressingMethod::I, OperandType::B, arch));
                    self.encoding = Some(OperandEncoding::MI);
                }
                0x83 => {
                    self.operands[0] =
                        Some(Operand::from_map(AddressingMethod::E, OperandType::V, arch));
                    self.operands[1] =
                        Some(Operand::from_map(AddressingMethod::I, OperandType::B, arch));
                    self.encoding = Some(OperandEncoding::MI);
                }
                0xFF => {
                    self.operands[0] =
                        Some(Operand::from_map(AddressingMethod::E, OperandType::V, arch));
                    self.encoding = Some(OperandEncoding::MI);
                }
                _ => {}
            },
            _ => {}
        };

        if let OpcodeType::NeedsModRMExtension(byte) = self.ident {
            // Depending on the opcode, we have specific identificators for instructions
            match byte {
                0x80 | 0x81 | 0x82 | 0x83 => {
                    self.ident = match ext.0 {
                        0 => OpcodeType::Add,
                        1 => OpcodeType::Or,
                        2 => OpcodeType::Adc,
                        3 => OpcodeType::Sbb,
                        4 => OpcodeType::And,
                        5 => OpcodeType::Sub,
                        6 => OpcodeType::Xor,
                        7 => OpcodeType::Cmp,
                        _ => unreachable!(),
                    };
                }
                0xFF => {
                    self.ident = match ext.0 {
                        0 => OpcodeType::Inc,
                        1 => OpcodeType::Dec,
                        2 => OpcodeType::CallNear,
                        3 => OpcodeType::CallFar,
                        4 => OpcodeType::JmpNear,
                        5 => OpcodeType::JmpFar,
                        6 => OpcodeType::Push,
                        _ => unreachable!(),
                    };
                }
                _ => todo!(),
            }
        }

        Ok(())
    }

    /// Special function that returns results based on the read prefix. This typically, and
    /// practically implies that the Opcode will be 2 or 3-bytes long.
    /// This function does not handle REX prefixes. It is the job of the caller to do that.
    pub fn with_prefix_arch(
        reader: &mut Reader,
        prefixs: &[Prefix],
        arch: Arch,
    ) -> Result<Self, OpcodeError> {
        // Read the first byte from the `reader`
        let first_byte = reader.read::<u8>()?;

        // Check where the first byte we read is an escaped code or not.
        match first_byte {
            // If we found an escape code, than we know that the Opcode is 2 or 3 bytes long
            opcode_prefix::ESCAPE_CODE => {
                match prefixs.len() {
                    0 => {
                        let second_byte = reader.read::<u8>()?;
                        match second_byte {
                            // Push FS Selector
                            0xA0 => {
                                let mut operands = [None, None, None, None];
                                operands[0] = Some(Operand::Segment(SegmentRegister::FS));
                                let encoding = Some(OperandEncoding::ZO);
                                Ok(Opcode {
                                    ident: OpcodeType::Push,
                                    operands,
                                    encoding,
                                })
                            }
                            // Push GS Selector
                            0xA8 => {
                                let mut operands = [None, None, None, None];
                                operands[0] = Some(Operand::Segment(SegmentRegister::GS));
                                let encoding = Some(OperandEncoding::ZO);
                                Ok(Opcode {
                                    ident: OpcodeType::Push,
                                    operands,
                                    encoding,
                                })
                            }
                            _ => Err(OpcodeError::InvalidOpcode(second_byte)),
                        }
                    }
                    _ => {
                        let prefix = prefixs[0];
                        match prefix {
                            Prefix::Group1(gr1) => {
                                match gr1 {
                                    Group1::RepNE => Ok(Opcode {
                                        ident: OpcodeType::Unknown,
                                        operands: [None, None, None, None],
                                        encoding: None,
                                    }),
                                    Group1::Rep => {
                                        let second_byte = reader.read::<u8>()?;
                                        match second_byte {
                                            // This is the byte that indicates an ENDBR
                                            0x1E => {
                                                // We have to read a 3rd byte
                                                let third_byte = reader.read::<u8>()?;
                                                match third_byte {
                                                    0xFB => Ok(Opcode {
                                                        ident: OpcodeType::EndBr32,
                                                        operands: [None, None, None, None],
                                                        encoding: Some(OperandEncoding::ZO),
                                                    }),
                                                    0xFA => Ok(Opcode {
                                                        ident: OpcodeType::EndBr64,
                                                        operands: [None, None, None, None],
                                                        encoding: Some(OperandEncoding::ZO),
                                                    }),
                                                    _ => Err(OpcodeError::Invalid3ByteOpcode(
                                                        first_byte,
                                                        second_byte,
                                                        third_byte,
                                                    )),
                                                }
                                            }
                                            _ => Ok(Opcode {
                                                ident: OpcodeType::Unknown,
                                                operands: [None, None, None, None],
                                                encoding: None,
                                            }),
                                        }
                                    }
                                    _ => Err(OpcodeError::InvalidPrefix(prefix)),
                                }
                            }
                            Prefix::OpSize => Ok(Opcode {
                                ident: OpcodeType::Unknown,
                                operands: [None, None, None, None],
                                encoding: None,
                            }),
                            // If we have an escape code, any other prefix is invalid for a 2-byte, 3-byte
                            // opcode
                            _ => Err(OpcodeError::InvalidPrefix(prefix)),
                        }
                    }
                }
            }
            // If the byte is not an escape code, that means it is just a 1-byte
            // opcode, that we have to parse.
            _ => Self::from_byte_with_arch(first_byte, arch),
        }
    }
}

mod opcode_prefix {
    pub const ESCAPE_CODE: u8 = 0x0F;
}
