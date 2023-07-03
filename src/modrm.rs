//! Represents the ModR/M and SIB bytes parsing

use crate::imm::DispArch;
use crate::inst::SizedOperand;
use crate::opcode::{AddrSize, OpSize};
use crate::reg::Reg;
use crate::rex::Rex;

/// Made up of three parts:
/// - R/M, bits[0:3]
/// - Reg/Opcode, bits[3:6]
/// - Mod, bits[6:8]
///
/// The `mod` field combines with the r/m field to form 32 possible values: eight registers and 24
/// addressing modes.
/// The `reg/opcode` field specifies either a register number or three more bits of opcode
/// information. The purpose of the `reg/opcode` field is specified in the primary opcode.
/// The `r/m` field can specify a register as an operand or it can be combined with the mod field
/// to encode an addressing mode
#[derive(Debug)]
pub struct ModRM(pub Reg, pub Addressing);

impl ModRM {
    pub fn from_byte_with_arch(
        value: u8,
        maybe_arch: Option<Arch>,
        maybe_rex: Option<Rex>,
    ) -> Self {
        let reg = (value >> 3) & 0b111;

        // We compute the addressing form, based on what we are passed
        let (reg, addressing) = match maybe_arch {
            // If we have an architecture passed, we parse addressing based on that
            Some(arch) => {
                match arch {
                    Arch::Arch16 => (reg, Addressing::EffAddr16Bit(EffAddr16Bit::from(value))),
                    Arch::Arch32 => (reg, Addressing::EffAddr32Bit(EffAddr32Bit::from(value))),
                    Arch::Arch64 => {
                        let addr = Addressing::EffAddr64Bit(EffAddr64Bit::from_byte_with_rex(
                            value, maybe_rex,
                        ));

                        let reg = match maybe_rex {
                            Some(rex) =>
                            // No matter the case, the `r` field of the Rex prefix, will always have to be
                            // prepended to the Reg
                            {
                                (rex.r() << 3) | reg
                            }
                            None => reg,
                        };
                        (reg, addr)
                    }
                }
            }
            // If not, the default is 32 Bits
            None => (reg, Addressing::EffAddr32Bit(EffAddr32Bit::from(value))),
        };

        // Get Mod
        let mod_addr = value >> 6 & 0b11;

        Self(Reg::from_byte_with_arch(reg, maybe_arch), addressing)
    }

    /// Returns the register of the R/M field(from ModRM) if it represents a register,
    /// otherwise `None`
    pub fn rm_reg(&self) -> Option<Reg> {
        self.1.rm_reg()
    }

    /// Returns the effective address of the R/M field(from ModRM) if it represents a memory region,
    /// otherwise `None`
    pub fn rm_mem(&self) -> EffAddrType {
        self.1.rm_mem()
    }

    pub fn reg(&self) -> Reg {
        self.0
    }

    pub fn mod_bits(&self) -> u8 {
        self.1.mod_bits()
    }
}

#[derive(Debug)]
pub enum Addressing {
    EffAddr16Bit(EffAddr16Bit),
    EffAddr32Bit(EffAddr32Bit),
    EffAddr64Bit(EffAddr64Bit),
}

impl Addressing {
    /// Returns the displacement based on addressing type or `None` if it does not exist
    pub fn displacement(&self) -> Option<DispArch> {
        // Check for the addressing type
        match self {
            Self::EffAddr16Bit(addr_16bit) => addr_16bit.maybe_disp,
            Self::EffAddr32Bit(addr_32bit) => addr_32bit.maybe_disp,
            Self::EffAddr64Bit(addr_64bit) => addr_64bit.maybe_disp,
        }
    }

    /// Returns the displacement based on addressing type or `None` if it does not exist
    pub fn set_displacement(&mut self, disp: Option<DispArch>) {
        // Check for the addressing type
        match self {
            Self::EffAddr16Bit(addr_16bit) => addr_16bit.maybe_disp = disp,
            Self::EffAddr32Bit(addr_32bit) => addr_32bit.maybe_disp = disp,
            Self::EffAddr64Bit(addr_64bit) => addr_64bit.maybe_disp = disp,
        }
    }

    /// If the CPU is in 32-bit or 64-bit addressing form, there is a chance the SIB byte is
    /// present, to aid the creation of the address.
    pub fn has_sib(&self) -> bool {
        let eff_addr = match self {
            Self::EffAddr32Bit(eff_addr_32bit) => eff_addr_32bit.eff_addr,
            Self::EffAddr64Bit(eff_addr_64bit) => eff_addr_64bit.eff_addr,
            Self::EffAddr16Bit(_) => return false,
        };

        if let EffAddrType::Sib = eff_addr {
            return true;
        } else {
            return false;
        }
    }

    pub fn mod_bits(&self) -> u8 {
        match self {
            Self::EffAddr16Bit(eff_addr_16bit) => eff_addr_16bit.mod_addr,
            Self::EffAddr32Bit(eff_addr_32bit) => eff_addr_32bit.mod_addr,
            Self::EffAddr64Bit(eff_addr_64bit) => eff_addr_64bit.mod_addr,
        }
    }

    pub fn rm_mem(&self) -> EffAddrType {
        match self {
            Self::EffAddr32Bit(eff_addr_32bit) => eff_addr_32bit.eff_addr,
            Self::EffAddr64Bit(eff_addr_64bit) => eff_addr_64bit.eff_addr,
            Self::EffAddr16Bit(_) => {
                panic!("Override addressing in 16 bit mode is not implemented")
            }
        }
    }

    /// Returns the register of the R/M field(from ModRM) if it represents a register,
    /// otherwise `None`
    pub fn rm_reg(&self) -> Option<Reg> {
        match self {
            Addressing::EffAddr16Bit(eff_addr_16bit) => {
                return eff_addr_16bit.maybe_reg1;
            }
            Addressing::EffAddr32Bit(eff_addr_32bit) => match eff_addr_32bit.eff_addr {
                EffAddrType::Reg(reg) => return Some(reg),
                _ => return None,
            },
            Addressing::EffAddr64Bit(eff_addr_64bit) => match eff_addr_64bit.eff_addr {
                EffAddrType::Reg(reg) => return Some(reg),
                _ => return None,
            },
        }
    }
}

// TODO: This should be places in some cpu.rs or arch.rs file
#[derive(Debug, Clone, Copy)]
pub enum Arch {
    Arch16,
    Arch32,
    Arch64,
}

/// Represents an Effective Address using 16-bit mode Addressing
#[derive(Debug)]
pub struct EffAddr16Bit {
    maybe_reg1: Option<Reg>,
    maybe_reg2: Option<Reg>,
    maybe_disp: Option<DispArch>,
    mod_addr: u8,
}

impl From<u8> for EffAddr16Bit {
    fn from(value: u8) -> Self {
        // Get R/M
        let r_m = value & 0b111;
        // Get Mod
        let mod_addr = value >> 6 & 0b11;

        let eff_addr_16bit = match mod_addr {
            0b00 => {
                match r_m {
                    0b000 => (Some(Reg::BX), Some(Reg::SI), None),
                    0b001 => (Some(Reg::BX), Some(Reg::DI), None),
                    0b010 => (Some(Reg::BP), Some(Reg::SI), None),
                    0b011 => (Some(Reg::BP), Some(Reg::DI), None),
                    0b100 => (Some(Reg::SI), None, None),
                    0b101 => (Some(Reg::DI), None, None),
                    0b110 => (None, None, Some(DispArch::Bit16)),
                    0b111 => (Some(Reg::BX), None, None),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            0b01 => {
                match r_m {
                    0b000 => (Some(Reg::BX), Some(Reg::SI), Some(DispArch::Bit8)),
                    0b001 => (Some(Reg::BX), Some(Reg::DI), Some(DispArch::Bit8)),
                    0b010 => (Some(Reg::BP), Some(Reg::SI), Some(DispArch::Bit8)),
                    0b011 => (Some(Reg::BP), Some(Reg::DI), Some(DispArch::Bit8)),
                    0b100 => (Some(Reg::SI), None, Some(DispArch::Bit8)),
                    0b101 => (Some(Reg::DI), None, Some(DispArch::Bit8)),
                    0b110 => (Some(Reg::BP), None, Some(DispArch::Bit8)),
                    0b111 => (Some(Reg::BX), None, Some(DispArch::Bit8)),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            0b10 => {
                match r_m {
                    0b000 => (Some(Reg::BX), Some(Reg::SI), Some(DispArch::Bit16)),
                    0b001 => (Some(Reg::BX), Some(Reg::DI), Some(DispArch::Bit16)),
                    0b010 => (Some(Reg::BP), Some(Reg::SI), Some(DispArch::Bit16)),
                    0b011 => (Some(Reg::BP), Some(Reg::DI), Some(DispArch::Bit16)),
                    0b100 => (Some(Reg::SI), None, Some(DispArch::Bit16)),
                    0b101 => (Some(Reg::DI), None, Some(DispArch::Bit16)),
                    0b110 => (Some(Reg::BP), None, Some(DispArch::Bit16)),
                    0b111 => (Some(Reg::BX), None, Some(DispArch::Bit16)),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => (None, None, None),
                }
            }
            0b11 => {
                // The following registers are just placeholders for a set of registers
                match r_m {
                    // EAX/AX/AL/MM0/XMM0
                    0b000 => (Some(Reg::EAX), None, None),
                    // ECX/CX/CL/MM1/XMM1
                    0b001 => (Some(Reg::ECX), None, None),
                    // EDX/DX/DL/MM2/XMM2
                    0b010 => (Some(Reg::EDX), None, None),
                    // EBX/BX/BL/MM3/XMM3
                    0b011 => (Some(Reg::EBX), None, None),
                    // ESP/SP/AHMM4/XMM4
                    0b100 => (Some(Reg::ESP), None, None),
                    // EBP/BP/CH/MM5/XMM5
                    0b101 => (Some(Reg::EBP), None, None),
                    // ESI/SI/DH/MM6/XMM6
                    0b110 => (Some(Reg::ESI), None, None),
                    // EDI/DI/BH/MM7/XMM7
                    0b111 => (Some(Reg::EDI), None, None),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            // Since we know only the low 2 bits can have a value for Mod, this option should never
            // be accessed.
            _ => unreachable!(),
        };

        Self {
            maybe_reg1: eff_addr_16bit.0,
            maybe_reg2: eff_addr_16bit.1,
            maybe_disp: eff_addr_16bit.2,
            mod_addr,
        }
    }
}

/// Represents an Effective Address using 16-bit mode Addressing
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EffAddr32Bit {
    eff_addr: EffAddrType,
    maybe_disp: Option<DispArch>,
    mod_addr: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EffAddrType {
    // This means that the base of the effective address is backed by a register
    Reg(Reg),
    // This means that we have to use the SIB(Scale, Base, Index) that follows the ModR/M byte to
    // get the effective address.
    Sib,
    // No need for a register or a SIB byte
    None,
}

impl SizedOperand for EffAddrType {
    fn size(&self) -> OpSize {
        match self {
            EffAddrType::Reg(reg) => reg.size(),
            _ => OpSize::CpuMode,
        }
    }
}

impl EffAddrType {
    pub fn convert_with_addrsize(self, addr_size: AddrSize) -> Self {
        match self {
            Self::Reg(reg) => Self::Reg(reg.convert_with_opsize(&OpSize::from(addr_size))),
            Self::Sib => Self::Sib,
            Self::None => Self::None,
        }
    }
}

impl From<u8> for EffAddr32Bit {
    fn from(value: u8) -> Self {
        // Get R/M
        let r_m = value & 0b111;
        // Get Mod
        let mod_addr = value >> 6 & 0b11;

        let eff_addr_32bit = match mod_addr {
            0b00 => {
                match r_m {
                    0b000 => (EffAddrType::Reg(Reg::EAX), None),
                    0b001 => (EffAddrType::Reg(Reg::ECX), None),
                    0b010 => (EffAddrType::Reg(Reg::EDX), None),
                    0b011 => (EffAddrType::Reg(Reg::EBX), None),
                    0b100 => (EffAddrType::Sib, None),
                    0b101 => (EffAddrType::None, Some(DispArch::Bit32)),
                    0b110 => (EffAddrType::Reg(Reg::ESI), None),
                    0b111 => (EffAddrType::Reg(Reg::EDI), None),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            0b01 => {
                match r_m {
                    0b000 => (EffAddrType::Reg(Reg::EAX), Some(DispArch::Bit8)),
                    0b001 => (EffAddrType::Reg(Reg::ECX), Some(DispArch::Bit8)),
                    0b010 => (EffAddrType::Reg(Reg::EDX), Some(DispArch::Bit8)),
                    0b011 => (EffAddrType::Reg(Reg::EBX), Some(DispArch::Bit8)),
                    0b100 => (EffAddrType::Sib, Some(DispArch::Bit8)),
                    0b101 => (EffAddrType::Reg(Reg::EBP), Some(DispArch::Bit8)),
                    0b110 => (EffAddrType::Reg(Reg::ESI), Some(DispArch::Bit8)),
                    0b111 => (EffAddrType::Reg(Reg::EDI), Some(DispArch::Bit8)),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            0b10 => {
                match r_m {
                    0b000 => (EffAddrType::Reg(Reg::EAX), Some(DispArch::Bit32)),
                    0b001 => (EffAddrType::Reg(Reg::ECX), Some(DispArch::Bit32)),
                    0b010 => (EffAddrType::Reg(Reg::EDX), Some(DispArch::Bit32)),
                    0b011 => (EffAddrType::Reg(Reg::EBX), Some(DispArch::Bit32)),
                    0b100 => (EffAddrType::Sib, Some(DispArch::Bit32)),
                    0b101 => (EffAddrType::Reg(Reg::EBP), Some(DispArch::Bit32)),
                    0b110 => (EffAddrType::Reg(Reg::ESI), Some(DispArch::Bit32)),
                    0b111 => (EffAddrType::Reg(Reg::EDI), Some(DispArch::Bit32)),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            0b11 => {
                // The following registers are just placeholders for a set of registers
                match r_m {
                    // EAX/AX/AL/MM0/XMM0
                    0b000 => (EffAddrType::Reg(Reg::EAX), None),
                    // ECX/CX/CL/MM1/XMM1
                    0b001 => (EffAddrType::Reg(Reg::ECX), None),
                    // EDX/DX/DL/MM2/XMM2
                    0b010 => (EffAddrType::Reg(Reg::EDX), None),
                    // EBX/BX/BL/MM3/XMM3
                    0b011 => (EffAddrType::Reg(Reg::EBX), None),
                    // ESP/SP/AHMM4/XMM4
                    0b100 => (EffAddrType::Reg(Reg::ESP), None),
                    // EBP/BP/CH/MM5/XMM5
                    0b101 => (EffAddrType::Reg(Reg::EBP), None),
                    // ESI/SI/DH/MM6/XMM6
                    0b110 => (EffAddrType::Reg(Reg::ESI), None),
                    // EDI/DI/BH/MM7/XMM7
                    0b111 => (EffAddrType::Reg(Reg::EDI), None),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            // Since we know only the low 2 bits can have a value for Mod, this option should never
            // be accessed.
            _ => unreachable!(),
        };

        Self {
            eff_addr: eff_addr_32bit.0,
            maybe_disp: eff_addr_32bit.1,
            mod_addr,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EffAddr64Bit {
    eff_addr: EffAddrType,
    maybe_disp: Option<DispArch>,
    mod_addr: u8,
}

impl EffAddr64Bit {
    fn from_byte_with_rex(value: u8, maybe_rex: Option<Rex>) -> Self {
        // Get R/M
        let mut r_m = value & 0b111;

        // Since we may be using a REX, we have to extend the r/m byte to the desired register
        if let Some(rex) = maybe_rex {
            r_m = (rex.b() << 3) | r_m;
        }

        // Get Mod
        let mod_addr = value >> 6 & 0b11;

        let eff_addr_64bit = match mod_addr {
            0b00 => {
                match r_m {
                    0b0000 => (EffAddrType::Reg(Reg::RAX), None),
                    0b0001 => (EffAddrType::Reg(Reg::RCX), None),
                    0b0010 => (EffAddrType::Reg(Reg::RDX), None),
                    0b0011 => (EffAddrType::Reg(Reg::RBX), None),
                    0b0100 => (EffAddrType::Sib, None),
                    0b0101 => (EffAddrType::None, Some(DispArch::Bit32)),
                    0b0110 => (EffAddrType::Reg(Reg::RSI), None),
                    0b0111 => (EffAddrType::Reg(Reg::RDI), None),
                    0b1000 => (EffAddrType::Reg(Reg::R8), None),
                    0b1001 => (EffAddrType::Reg(Reg::R9), None),
                    0b1010 => (EffAddrType::Reg(Reg::R10), None),
                    0b1011 => (EffAddrType::Reg(Reg::R11), None),
                    0b1100 => (EffAddrType::Sib, None),
                    0b1101 => (EffAddrType::Reg(Reg::R13), Some(DispArch::Bit32)),
                    0b1110 => (EffAddrType::Reg(Reg::R14), None),
                    0b1111 => (EffAddrType::Reg(Reg::R15), None),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            0b01 => {
                match r_m {
                    0b0000 => (EffAddrType::Reg(Reg::RAX), Some(DispArch::Bit8)),
                    0b0001 => (EffAddrType::Reg(Reg::RCX), Some(DispArch::Bit8)),
                    0b0010 => (EffAddrType::Reg(Reg::RDX), Some(DispArch::Bit8)),
                    0b0011 => (EffAddrType::Reg(Reg::RBX), Some(DispArch::Bit8)),
                    0b0100 => (EffAddrType::Sib, Some(DispArch::Bit8)),
                    0b0101 => (EffAddrType::Reg(Reg::RBP), Some(DispArch::Bit8)),
                    0b0110 => (EffAddrType::Reg(Reg::RSI), Some(DispArch::Bit8)),
                    0b0111 => (EffAddrType::Reg(Reg::RDI), Some(DispArch::Bit8)),
                    0b1000 => (EffAddrType::Reg(Reg::R8), Some(DispArch::Bit8)),
                    0b1001 => (EffAddrType::Reg(Reg::R9), Some(DispArch::Bit8)),
                    0b1010 => (EffAddrType::Reg(Reg::R10), Some(DispArch::Bit8)),
                    0b1011 => (EffAddrType::Reg(Reg::R11), Some(DispArch::Bit8)),
                    0b1100 => (EffAddrType::Sib, Some(DispArch::Bit8)),
                    0b1101 => (EffAddrType::Reg(Reg::R13), Some(DispArch::Bit8)),
                    0b1110 => (EffAddrType::Reg(Reg::R14), Some(DispArch::Bit8)),
                    0b1111 => (EffAddrType::Reg(Reg::R15), Some(DispArch::Bit8)),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            0b10 => {
                match r_m {
                    0b0000 => (EffAddrType::Reg(Reg::RAX), Some(DispArch::Bit32)),
                    0b0001 => (EffAddrType::Reg(Reg::RCX), Some(DispArch::Bit32)),
                    0b0010 => (EffAddrType::Reg(Reg::RDX), Some(DispArch::Bit32)),
                    0b0011 => (EffAddrType::Reg(Reg::RBX), Some(DispArch::Bit32)),
                    0b0100 => (EffAddrType::Sib, Some(DispArch::Bit32)),
                    0b0101 => (EffAddrType::Reg(Reg::RBP), Some(DispArch::Bit32)),
                    0b0110 => (EffAddrType::Reg(Reg::RSI), Some(DispArch::Bit32)),
                    0b0111 => (EffAddrType::Reg(Reg::RDI), Some(DispArch::Bit32)),
                    0b1000 => (EffAddrType::Reg(Reg::R8), Some(DispArch::Bit32)),
                    0b1001 => (EffAddrType::Reg(Reg::R9), Some(DispArch::Bit32)),
                    0b1010 => (EffAddrType::Reg(Reg::R10), Some(DispArch::Bit32)),
                    0b1011 => (EffAddrType::Reg(Reg::R11), Some(DispArch::Bit32)),
                    0b1100 => (EffAddrType::Sib, Some(DispArch::Bit32)),
                    0b1101 => (EffAddrType::Reg(Reg::R13), Some(DispArch::Bit32)),
                    0b1110 => (EffAddrType::Reg(Reg::R14), Some(DispArch::Bit32)),
                    0b1111 => (EffAddrType::Reg(Reg::R15), Some(DispArch::Bit32)),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            0b11 => {
                // The following registers are just placeholders for a set of registers
                match r_m {
                    // EAX/AX/AL/MM0/XMM0
                    0b0000 => (EffAddrType::Reg(Reg::RAX), None),
                    // ECX/CX/CL/MM1/XMM1
                    0b0001 => (EffAddrType::Reg(Reg::RCX), None),
                    // EDX/DX/DL/MM2/XMM2
                    0b0010 => (EffAddrType::Reg(Reg::RDX), None),
                    // EBX/BX/BL/MM3/XMM3
                    0b0011 => (EffAddrType::Reg(Reg::RBX), None),
                    // ESP/SP/AHMM4/XMM4
                    0b0100 => (EffAddrType::Reg(Reg::RSP), None),
                    // EBP/BP/CH/MM5/XMM5
                    0b0101 => (EffAddrType::Reg(Reg::RBP), None),
                    // ESI/SI/DH/MM6/XMM6
                    0b0110 => (EffAddrType::Reg(Reg::RSI), None),
                    // EDI/DI/BH/MM7/XMM7
                    0b0111 => (EffAddrType::Reg(Reg::RDI), None),
                    // EAX/AX/AL/MM0/XMM0
                    0b1000 => (EffAddrType::Reg(Reg::R8), None),
                    // ECX/CX/CL/MM1/XMM1
                    0b1001 => (EffAddrType::Reg(Reg::R9), None),
                    // EDX/DX/DL/MM2/XMM2
                    0b1010 => (EffAddrType::Reg(Reg::R10), None),
                    // EBX/BX/BL/MM3/XMM3
                    0b1011 => (EffAddrType::Reg(Reg::R11), None),
                    // ESP/SP/AHMM4/XMM4
                    0b1100 => (EffAddrType::Reg(Reg::R12), None),
                    // EBP/BP/CH/MM5/XMM5
                    0b1101 => (EffAddrType::Reg(Reg::R13), None),
                    // ESI/SI/DH/MM6/XMM6
                    0b1110 => (EffAddrType::Reg(Reg::R14), None),
                    // EDI/DI/BH/MM7/XMM7
                    0b1111 => (EffAddrType::Reg(Reg::R15), None),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            // Since we know only the low 2 bits can have a value for Mod, this option should never
            // be accessed.
            _ => unreachable!(),
        };

        Self {
            eff_addr: eff_addr_64bit.0,
            maybe_disp: eff_addr_64bit.1,
            mod_addr,
        }
    }
}

/// Made up of also 3 parts:
/// - Base, bits[0:3], specifies the register number of the base register.
/// - Index, bits[3:6], specifies the register number of the index register.
/// - Scale, bits[6:8], specifies the scale factor.
///
/// Certain encodings of the ModR/M byte require a second addressing byte (the SIB byte). The
/// base-plus-index and scale-plus-index forms of 32-bit addressing require the SIB byte.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Sib {
    Sib32(Sib32),
    Sib64(Sib64),
}

impl SizedOperand for Sib {
    fn size(&self) -> OpSize {
        match self {
            Sib::Sib32(_) => OpSize::U32,
            Sib::Sib64(_) => OpSize::U64,
        }
    }
}

impl Sib {
    pub fn convert_with_addrsize(self, addr_size: AddrSize) -> Self {
        let op_size = OpSize::from(addr_size);
        match self {
            Self::Sib32(sib32) => {
                let base = match sib32.base {
                    Some(reg) => Some(reg.convert_with_opsize(&op_size)),
                    None => None,
                };
                let scaled_index = match sib32.scaled_index {
                    Some(reg) => Some(reg.convert_with_opsize(&op_size)),
                    None => None,
                };

                Self::Sib32(Sib32 {
                    base,
                    scaled_index,
                    scale: sib32.scale,
                })
            }
            Self::Sib64(sib64) => {
                let base = match sib64.base {
                    Some(reg) => Some(reg.convert_with_opsize(&op_size)),
                    None => None,
                };
                let scaled_index = match sib64.scaled_index {
                    Some(reg) => Some(reg.convert_with_opsize(&op_size)),
                    None => None,
                };

                Self::Sib64(Sib64 {
                    base,
                    scaled_index,
                    scale: sib64.scale,
                })
            }
        }
    }

    pub fn base(&self) -> Option<Reg> {
        match self {
            Self::Sib32(sib32) => sib32.base,
            Self::Sib64(sib64) => sib64.base,
        }
    }

    pub fn set_base(&mut self, base: Option<Reg>) {
        match self {
            Self::Sib32(sib32) => sib32.base = base,
            Self::Sib64(sib64) => sib64.base = base,
        };
    }
}

// This represents the top 2 bits(Scale parameter) of the SIB byte in an x86_64 instruction
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Scale(u8);

/// Represents a 32-bit Sib byte components
// TODO: We should make this version and the 64-bit version into generics
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Sib32 {
    base: Option<Reg>,
    scaled_index: Option<Reg>,
    scale: Option<Scale>,
}

impl From<u8> for Sib32 {
    fn from(value: u8) -> Self {
        let scale = (value >> 6) & 0b11;
        let idx = (value >> 3) & 0b111;
        let base = value & 0b111;

        let base = match base {
            0b000 => Some(Reg::EAX),
            0b001 => Some(Reg::ECX),
            0b010 => Some(Reg::EDX),
            0b011 => Some(Reg::EBX),
            0b100 => Some(Reg::ESP),
            0b101 => Some(Reg::EBP),
            0b110 => Some(Reg::ESI),
            0b111 => Some(Reg::EDI),
            _ => unreachable!(),
        };

        let scaled_index = match scale {
            0b00 => match idx {
                0b000 => (Some(Reg::EAX), None),
                0b001 => (Some(Reg::ECX), None),
                0b010 => (Some(Reg::EDX), None),
                0b011 => (Some(Reg::EBX), None),
                0b100 => (None, None),
                0b101 => (Some(Reg::EBP), None),
                0b110 => (Some(Reg::ESI), None),
                0b111 => (Some(Reg::EDI), None),
                _ => unreachable!(),
            },
            0b01 => match idx {
                0b000 => (Some(Reg::EAX), Some(Scale(2))),
                0b001 => (Some(Reg::ECX), Some(Scale(2))),
                0b010 => (Some(Reg::EDX), Some(Scale(2))),
                0b011 => (Some(Reg::EBX), Some(Scale(2))),
                0b100 => (None, None),
                0b101 => (Some(Reg::EBP), Some(Scale(2))),
                0b110 => (Some(Reg::ESI), Some(Scale(2))),
                0b111 => (Some(Reg::EDI), Some(Scale(2))),
                _ => unreachable!(),
            },
            0b10 => match idx {
                0b000 => (Some(Reg::EAX), Some(Scale(4))),
                0b001 => (Some(Reg::ECX), Some(Scale(4))),
                0b010 => (Some(Reg::EDX), Some(Scale(4))),
                0b011 => (Some(Reg::EBX), Some(Scale(4))),
                0b100 => (None, None),
                0b101 => (Some(Reg::EBP), Some(Scale(4))),
                0b110 => (Some(Reg::ESI), Some(Scale(4))),
                0b111 => (Some(Reg::EDI), Some(Scale(4))),
                _ => unreachable!(),
            },
            0b11 => match idx {
                0b000 => (Some(Reg::EAX), Some(Scale(8))),
                0b001 => (Some(Reg::ECX), Some(Scale(8))),
                0b010 => (Some(Reg::EDX), Some(Scale(8))),
                0b011 => (Some(Reg::EBX), Some(Scale(8))),
                0b100 => (None, None),
                0b101 => (Some(Reg::EBP), Some(Scale(8))),
                0b110 => (Some(Reg::ESI), Some(Scale(8))),
                0b111 => (Some(Reg::EDI), Some(Scale(8))),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

        Self {
            base,
            scaled_index: scaled_index.0,
            scale: scaled_index.1,
        }
    }
}

/// Represents a 64-bit scaled index
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Sib64 {
    base: Option<Reg>,
    scaled_index: Option<Reg>,
    scale: Option<Scale>,
}

impl Sib64 {
    pub fn from_byte_with_rex(value: u8, maybe_rex: Option<Rex>) -> Self {
        let scale = (value >> 6) & 0b11;

        let mut idx = (value >> 3) & 0b111;
        let mut base = value & 0b111;

        if let Some(rex) = maybe_rex {
            idx = (rex.x() << 3) | idx;
            base = (rex.b() << 3) | base;
        }

        println!("Base: {base:b}");

        let base = match base {
            0b0000 => Some(Reg::RAX),
            0b0001 => Some(Reg::RCX),
            0b0010 => Some(Reg::RDX),
            0b0011 => Some(Reg::RBX),
            0b0100 => Some(Reg::RSP),
            0b0101 => Some(Reg::RBP),
            0b0110 => Some(Reg::RSI),
            0b0111 => Some(Reg::RDI),
            0b1000 => Some(Reg::R8),
            0b1001 => Some(Reg::R9),
            0b1010 => Some(Reg::R10),
            0b1011 => Some(Reg::R11),
            0b1100 => Some(Reg::R12),
            0b1101 => Some(Reg::R13),
            0b1110 => Some(Reg::R14),
            0b1111 => Some(Reg::R15),
            _ => unreachable!(),
        };

        let scaled_index = match scale {
            0b00 => match idx {
                0b0000 => (Some(Reg::RAX), None),
                0b0001 => (Some(Reg::RCX), None),
                0b0010 => (Some(Reg::RDX), None),
                0b0011 => (Some(Reg::RBX), None),
                0b0100 => (None, None),
                0b0101 => (Some(Reg::RBP), None),
                0b0110 => (Some(Reg::RSI), None),
                0b0111 => (Some(Reg::RDI), None),
                0b1000 => (Some(Reg::R8), None),
                0b1001 => (Some(Reg::R9), None),
                0b1010 => (Some(Reg::R10), None),
                0b1011 => (Some(Reg::R11), None),
                0b1100 => (Some(Reg::R12), None),
                0b1101 => (Some(Reg::R13), None),
                0b1110 => (Some(Reg::R14), None),
                0b1111 => (Some(Reg::R15), None),
                _ => unreachable!(),
            },
            0b01 => match idx {
                0b0000 => (Some(Reg::RAX), Some(Scale(2))),
                0b0001 => (Some(Reg::RCX), Some(Scale(2))),
                0b0010 => (Some(Reg::RDX), Some(Scale(2))),
                0b0011 => (Some(Reg::RBX), Some(Scale(2))),
                0b0100 => (None, None),
                0b0101 => (Some(Reg::RBP), Some(Scale(2))),
                0b0110 => (Some(Reg::RSI), Some(Scale(2))),
                0b0111 => (Some(Reg::RDI), Some(Scale(2))),
                0b1000 => (Some(Reg::R8), Some(Scale(2))),
                0b1001 => (Some(Reg::R9), Some(Scale(2))),
                0b1010 => (Some(Reg::R10), Some(Scale(2))),
                0b1011 => (Some(Reg::R11), Some(Scale(2))),
                0b1100 => (Some(Reg::R12), Some(Scale(2))),
                0b1101 => (Some(Reg::R13), Some(Scale(2))),
                0b1110 => (Some(Reg::R14), Some(Scale(2))),
                0b1111 => (Some(Reg::R15), Some(Scale(2))),
                _ => unreachable!(),
            },
            0b10 => match idx {
                0b0000 => (Some(Reg::RAX), Some(Scale(4))),
                0b0001 => (Some(Reg::RCX), Some(Scale(4))),
                0b0010 => (Some(Reg::RDX), Some(Scale(4))),
                0b0011 => (Some(Reg::RBX), Some(Scale(4))),
                0b0100 => (None, None),
                0b0101 => (Some(Reg::RBP), Some(Scale(4))),
                0b0110 => (Some(Reg::RSI), Some(Scale(4))),
                0b0111 => (Some(Reg::RDI), Some(Scale(4))),
                0b1000 => (Some(Reg::R8), Some(Scale(4))),
                0b1001 => (Some(Reg::R9), Some(Scale(4))),
                0b1010 => (Some(Reg::R10), Some(Scale(4))),
                0b1011 => (Some(Reg::R11), Some(Scale(4))),
                0b1100 => (Some(Reg::R12), Some(Scale(4))),
                0b1101 => (Some(Reg::R13), Some(Scale(4))),
                0b1110 => (Some(Reg::R14), Some(Scale(4))),
                0b1111 => (Some(Reg::R15), Some(Scale(4))),
                _ => unreachable!(),
            },
            0b11 => match idx {
                0b0000 => (Some(Reg::RAX), Some(Scale(8))),
                0b0001 => (Some(Reg::RCX), Some(Scale(8))),
                0b0010 => (Some(Reg::RDX), Some(Scale(8))),
                0b0011 => (Some(Reg::RBX), Some(Scale(8))),
                0b0100 => (None, None),
                0b0101 => (Some(Reg::RBP), Some(Scale(8))),
                0b0110 => (Some(Reg::RSI), Some(Scale(8))),
                0b0111 => (Some(Reg::RDI), Some(Scale(8))),
                0b1000 => (Some(Reg::R8), Some(Scale(8))),
                0b1001 => (Some(Reg::R9), Some(Scale(8))),
                0b1010 => (Some(Reg::R10), Some(Scale(8))),
                0b1011 => (Some(Reg::R11), Some(Scale(8))),
                0b1100 => (Some(Reg::R12), Some(Scale(8))),
                0b1101 => (Some(Reg::R13), Some(Scale(8))),
                0b1110 => (Some(Reg::R14), Some(Scale(8))),
                0b1111 => (Some(Reg::R15), Some(Scale(8))),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

        Self {
            base,
            scaled_index: scaled_index.0,
            scale: scaled_index.1,
        }
    }
}
