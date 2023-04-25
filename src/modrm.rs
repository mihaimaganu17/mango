//! Represents the ModR/M and SIB bytes parsing

use crate::imm::DispArch;
use crate::reg::Reg;

/// Made up of three parts:
/// - R/M, bits[0:3]
/// - Reg/Opcode, bits[3:6]
/// - Mod, bits[6:8]
///
/// The `mod` field combines with the r/m field to form 32 possible values: eight registers and 24
/// addressing modes.
/// The `reg/opcode` field specifies either a register number or three more bits of opcode
// TODO: We need to parse addresses based on 16-bit and 32-bit addressing forms.
/// information. The purpose of the `reg/opcode` field is specified in the primary opcode.
/// The `r/m` field can specify a register as an operand or it can be combined with the mod field
/// to encode an addressing mode
pub struct ModRM(u8);

/// Represents an Effective Address using 16-bit mode Addressing
#[derive(Debug)]
pub struct EffAddr16Bit(Option<Reg>, Option<Reg>, Option<DispArch>);

impl From<u8> for EffAddr16Bit {
    fn from(value: u8) -> Self{
        // Get R/M
        let r_m = value & 0b111;
        // Get Mod
        let mod_addr = value >> 6 & 0b11;

        let eff_addr_16bit = match mod_addr {
            0b00 => {
                match r_m {
                    0b000 => Self(Some(Reg::BX), Some(Reg::SI), None),
                    0b001 => Self(Some(Reg::BX), Some(Reg::DI), None),
                    0b010 => Self(Some(Reg::BP), Some(Reg::SI), None),
                    0b011 => Self(Some(Reg::BP), Some(Reg::DI), None),
                    0b100 => Self(Some(Reg::SI), None, None),
                    0b101 => Self(Some(Reg::DI), None, None),
                    0b110 => Self(None, None, Some(DispArch::Bit16)),
                    0b111 => Self(Some(Reg::BX), None, None),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            0b01 => {
                match r_m {
                    0b000 => Self(Some(Reg::BX), Some(Reg::SI), Some(DispArch::Bit8)),
                    0b001 => Self(Some(Reg::BX), Some(Reg::DI), Some(DispArch::Bit8)),
                    0b010 => Self(Some(Reg::BP), Some(Reg::SI), Some(DispArch::Bit8)),
                    0b011 => Self(Some(Reg::BP), Some(Reg::DI), Some(DispArch::Bit8)),
                    0b100 => Self(Some(Reg::SI), None, Some(DispArch::Bit8)),
                    0b100 => Self(Some(Reg::DI), None, Some(DispArch::Bit8)),
                    0b100 => Self(Some(Reg::BP), None, Some(DispArch::Bit8)),
                    0b100 => Self(Some(Reg::BX), None, Some(DispArch::Bit8)),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            0b01 => {
                match r_m {
                    0b000 => Self(Some(Reg::BX), Some(Reg::SI), Some(DispArch::Bit8)),
                    0b001 => Self(Some(Reg::BX), Some(Reg::DI), Some(DispArch::Bit8)),
                    0b010 => Self(Some(Reg::BP), Some(Reg::SI), Some(DispArch::Bit8)),
                    0b011 => Self(Some(Reg::BP), Some(Reg::DI), Some(DispArch::Bit8)),
                    0b100 => Self(Some(Reg::SI), None, Some(DispArch::Bit8)),
                    0b101 => Self(Some(Reg::DI), None, Some(DispArch::Bit8)),
                    0b110 => Self(Some(Reg::BP), None, Some(DispArch::Bit8)),
                    0b111 => Self(Some(Reg::BX), None, Some(DispArch::Bit8)),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            0b10 => {
                match r_m {
                    0b000 => Self(Some(Reg::BX), Some(Reg::SI), Some(DispArch::Bit16)),
                    0b001 => Self(Some(Reg::BX), Some(Reg::DI), Some(DispArch::Bit16)),
                    0b010 => Self(Some(Reg::BP), Some(Reg::SI), Some(DispArch::Bit16)),
                    0b011 => Self(Some(Reg::BP), Some(Reg::DI), Some(DispArch::Bit16)),
                    0b100 => Self(Some(Reg::SI), None, Some(DispArch::Bit16)),
                    0b101 => Self(Some(Reg::DI), None, Some(DispArch::Bit16)),
                    0b110 => Self(Some(Reg::BP), None, Some(DispArch::Bit16)),
                    0b111 => Self(Some(Reg::BX), None, Some(DispArch::Bit16)),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => Self(None, None, None),
                }
            }
            0b11 => {
                // The following registers are just placeholders for a set of registers
                match r_m {
                    // EAX/AX/AL/MM0/XMM0
                    0b000 => Self(Some(Reg::EAX), None, None),
                    // ECX/CX/CL/MM1/XMM1
                    0b000 => Self(Some(Reg::ECX), None, None),
                    // EDX/DX/DL/MM2/XMM2
                    0b000 => Self(Some(Reg::EDX), None, None),
                    // EBX/BX/BL/MM3/XMM3
                    0b000 => Self(Some(Reg::EBX), None, None),
                    // ESP/SP/AHMM4/XMM4
                    0b000 => Self(Some(Reg::ESP), None, None),
                    // EBP/BP/CH/MM5/XMM5
                    0b000 => Self(Some(Reg::EBP), None, None),
                    // ESI/SI/DH/MM6/XMM6
                    0b000 => Self(Some(Reg::ESI), None, None),
                    // EDI/DI/BH/MM7/XMM7
                    0b000 => Self(Some(Reg::EDI), None, None),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            // Since we know only the low 2 bits can have a value for Mod, this option should never
            // be accessed.
            _ => unreachable!(),
        };

        eff_addr_16bit
    }
}

/// Represents an Effective Address using 16-bit mode Addressing
#[derive(Debug)]
pub struct EffAddr32Bit(Option<Reg>, Option<Reg>, Option<DispArch>);

impl From<u8> for EffAddr16Bit {
    fn from(value: u8) -> Self{
        // Get R/M
        let r_m = value & 0b111;
        // Get Mod
        let mod_addr = value >> 6 & 0b11;

        let eff_addr_16bit = match mod_addr {
            0b00 => {
                match r_m {
                    0b000 => Self(Some(Reg::BX), Some(Reg::SI), None),
                    0b001 => Self(Some(Reg::BX), Some(Reg::DI), None),
                    0b010 => Self(Some(Reg::BP), Some(Reg::SI), None),
                    0b011 => Self(Some(Reg::BP), Some(Reg::DI), None),
                    0b100 => Self(Some(Reg::SI), None, None),
                    0b101 => Self(Some(Reg::DI), None, None),
                    0b110 => Self(None, None, Some(DispArch::Bit16)),
                    0b111 => Self(Some(Reg::BX), None, None),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            0b01 => {
                match r_m {
                    0b000 => Self(Some(Reg::BX), Some(Reg::SI), Some(DispArch::Bit8)),
                    0b001 => Self(Some(Reg::BX), Some(Reg::DI), Some(DispArch::Bit8)),
                    0b010 => Self(Some(Reg::BP), Some(Reg::SI), Some(DispArch::Bit8)),
                    0b011 => Self(Some(Reg::BP), Some(Reg::DI), Some(DispArch::Bit8)),
                    0b100 => Self(Some(Reg::SI), None, Some(DispArch::Bit8)),
                    0b100 => Self(Some(Reg::DI), None, Some(DispArch::Bit8)),
                    0b100 => Self(Some(Reg::BP), None, Some(DispArch::Bit8)),
                    0b100 => Self(Some(Reg::BX), None, Some(DispArch::Bit8)),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            0b01 => {
                match r_m {
                    0b000 => Self(Some(Reg::BX), Some(Reg::SI), Some(DispArch::Bit8)),
                    0b001 => Self(Some(Reg::BX), Some(Reg::DI), Some(DispArch::Bit8)),
                    0b010 => Self(Some(Reg::BP), Some(Reg::SI), Some(DispArch::Bit8)),
                    0b011 => Self(Some(Reg::BP), Some(Reg::DI), Some(DispArch::Bit8)),
                    0b100 => Self(Some(Reg::SI), None, Some(DispArch::Bit8)),
                    0b101 => Self(Some(Reg::DI), None, Some(DispArch::Bit8)),
                    0b110 => Self(Some(Reg::BP), None, Some(DispArch::Bit8)),
                    0b111 => Self(Some(Reg::BX), None, Some(DispArch::Bit8)),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            0b10 => {
                match r_m {
                    0b000 => Self(Some(Reg::BX), Some(Reg::SI), Some(DispArch::Bit16)),
                    0b001 => Self(Some(Reg::BX), Some(Reg::DI), Some(DispArch::Bit16)),
                    0b010 => Self(Some(Reg::BP), Some(Reg::SI), Some(DispArch::Bit16)),
                    0b011 => Self(Some(Reg::BP), Some(Reg::DI), Some(DispArch::Bit16)),
                    0b100 => Self(Some(Reg::SI), None, Some(DispArch::Bit16)),
                    0b101 => Self(Some(Reg::DI), None, Some(DispArch::Bit16)),
                    0b110 => Self(Some(Reg::BP), None, Some(DispArch::Bit16)),
                    0b111 => Self(Some(Reg::BX), None, Some(DispArch::Bit16)),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => Self(None, None, None),
                }
            }
            0b11 => {
                // The following registers are just placeholders for a set of registers
                match r_m {
                    // EAX/AX/AL/MM0/XMM0
                    0b000 => Self(Some(Reg::EAX), None, None),
                    // ECX/CX/CL/MM1/XMM1
                    0b000 => Self(Some(Reg::ECX), None, None),
                    // EDX/DX/DL/MM2/XMM2
                    0b000 => Self(Some(Reg::EDX), None, None),
                    // EBX/BX/BL/MM3/XMM3
                    0b000 => Self(Some(Reg::EBX), None, None),
                    // ESP/SP/AHMM4/XMM4
                    0b000 => Self(Some(Reg::ESP), None, None),
                    // EBP/BP/CH/MM5/XMM5
                    0b000 => Self(Some(Reg::EBP), None, None),
                    // ESI/SI/DH/MM6/XMM6
                    0b000 => Self(Some(Reg::ESI), None, None),
                    // EDI/DI/BH/MM7/XMM7
                    0b000 => Self(Some(Reg::EDI), None, None),
                    // Since we know only the low 3 bits can have a value in R/M, this option is
                    // only needed by the Rust compiler and something very wrong happened
                    _ => unreachable!(),
                }
            }
            // Since we know only the low 2 bits can have a value for Mod, this option should never
            // be accessed.
            _ => unreachable!(),
        };

        eff_addr_16bit
    }
}

pub struct Addressing16Bit {
    mod_rm: ModRM,
    eff_addr: EffAddr16Bit,
}

/// Made up of also 3 parts:
/// - Base, bits[0:3], specifies the register number of the base register.
/// - Index, bits[3:6], specifies the register number of the index register.
/// - Scale, bits[6:8], specifies the scale factor.
///
/// Certain encodings of the ModR/M byte require a second addressing byte (the SIB byte). The
/// base-plus-index and scale-plus-index forms of 32-bit addressing require the SIB byte.
pub struct SIB(u8);