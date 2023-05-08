use crate::modrm::ModRM;

#[derive(Debug)]
pub enum Reg {
    AL,
    AX,
    EAX,
    MM0,
    XMM0,
    CL,
    CX,
    ECX,
    MM1,
    XMM1,
    DL,
    DX,
    EDX,
    MM2,
    XMM2,
    BL,
    BX,
    EBX,
    MM3,
    XMM3,
    AH,
    SP,
    ESP,
    MM4,
    XMM4,
    CH,
    BP,
    EBP,
    MM5,
    XMM5,
    DH,
    SI,
    ESI,
    MM6,
    XMM6,
    BH,
    DI,
    EDI,
    MM7,
    XMM7,
}

impl Reg {
    // Convert the value to a register, specified by r/m16
    // A word general-purpose register or memory operand used for instructions whose operand-size
    // attribute is 16 bits. The word general-purpose registers are: AX, CX, DX, BX, SP, BP, SI,
    // DI. The contents of memory are found at the address provided by the effective address
    // computation. Word registers R8W - R15W are available using REX.R in 64-bit mode.
    pub fn from_rm16(value: u8) -> Self {
        // We make sure that value can have only the lower 3 bits set
        let value = value & 0b111;
        match value {
            0 => Self::AX,
            1 => Self::CX,
            2 => Self::DX,
            3 => Self::BX,
            4 => Self::SP,
            5 => Self::BP,
            6 => Self::SI,
            7 => Self::DI,
            _ => unreachable!(),
        }
    }

    // Convert the value to a register, specified by r/m32
    // A word general-purpose register or memory operand used for instructions whose operand-size
    // attribute is 16 bits. The word general-purpose registers are: EAX, ECX, EDX, EBX, ESP, EBP,
    // ESI, EDI. The contents of memory are found at the address provided by the effective address
    // computation. Word registers R8D - R15D are available using REX.R in 64-bit mode.
    pub fn from_rm32(value: u8) -> Self {
        // We make sure that value can have only the lower 3 bits set
        let value = value & 0b111;
        match value {
            0 => Self::EAX,
            1 => Self::ECX,
            2 => Self::EDX,
            3 => Self::EBX,
            4 => Self::ESP,
            5 => Self::EBP,
            6 => Self::ESI,
            7 => Self::EDI,
            _ => unreachable!(),
        }
    }
}

