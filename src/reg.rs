use crate::{opcode::OpSize, modrm::Arch, inst::SizedOperand};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
    RAX,
    RCX,
    RDX,
    RBX,
    RSP,
    RBP,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    R8b,
    R9b,
    R10b,
    R11b,
    R12b,
    R13b,
    R14b,
    R15b,
    R8w,
    R9w,
    R10w,
    R11w,
    R12w,
    R13w,
    R14w,
    R15w,
    R8d,
    R9d,
    R10d,
    R11d,
    R12d,
    R13d,
    R14d,
    R15d,
    SIL,
    DIL,
    SPL,
    BPL,
}

impl SizedOperand for Reg {
    fn size(&self) -> OpSize {
        match self {
            Reg::AL
            | Reg::CL
            | Reg::DL
            | Reg::BL
            | Reg::SPL
            | Reg::BPL
            | Reg::SIL
            | Reg::DIL
            | Reg::R8b
            | Reg::R9b
            | Reg::R10b 
            | Reg::R11b 
            | Reg::R12b
            | Reg::R13b 
            | Reg::R14b 
            | Reg::R15b => OpSize::U8,
            Reg::AX
            | Reg::CX
            | Reg::DX
            | Reg::BX
            | Reg::SP
            | Reg::BP
            | Reg::SI
            | Reg::DI
            | Reg::R8w
            | Reg::R9w
            | Reg::R10w 
            | Reg::R11w 
            | Reg::R12w
            | Reg::R13w 
            | Reg::R14w 
            | Reg::R15w => OpSize::U16,
            Reg::EAX
            | Reg::ECX
            | Reg::EDX
            | Reg::EBX
            | Reg::ESP
            | Reg::EBP
            | Reg::ESI
            | Reg::EDI
            | Reg::R8d
            | Reg::R9d
            | Reg::R10d 
            | Reg::R11d 
            | Reg::R12d
            | Reg::R13d 
            | Reg::R14d 
            | Reg::R15d => OpSize::U32,
            Reg::RAX
            | Reg::RCX
            | Reg::RDX
            | Reg::RBX
            | Reg::RSP
            | Reg::RBP
            | Reg::RSI
            | Reg::RDI
            | Reg::R8
            | Reg::R9
            | Reg::R10 
            | Reg::R11 
            | Reg::R12
            | Reg::R13 
            | Reg::R14 
            | Reg::R15 => OpSize::U64,
            _ => OpSize::CpuMode,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegFamily {
    Accumulator,
}

impl RegFamily {
    pub fn reg_from(&self, op_size: &OpSize) -> Reg {
        match self {
            Self::Accumulator => Accumulator::from_opsize(op_size),
        }
    }
}

pub trait Gpr {
    const Reg8BitLo: Reg;
    const Reg8BitHi: Option<Reg>;
    const Reg16Bit: Reg;
    const Reg32Bit: Reg;
    const Reg64Bit: Reg;

    fn from_opsize(op_size: &OpSize) -> Reg {
        match op_size {
            OpSize::U8 => Self::Reg8BitLo,
            OpSize::U16 => Self::Reg16Bit,
            OpSize::U32 => Self::Reg32Bit,
            OpSize::U64 => Self::Reg64Bit, 
            _ => Self::Reg32Bit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Accumulator;

impl Gpr for Accumulator {
    const Reg8BitLo: Reg = Reg::AL;
    const Reg8BitHi: Option<Reg> = Some(Reg::AH);
    const Reg16Bit: Reg = Reg::AX;
    const Reg32Bit: Reg = Reg::EAX;
    const Reg64Bit: Reg = Reg::RAX;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counter;

impl Gpr for Counter {
    const Reg8BitLo: Reg = Reg::CL;
    const Reg8BitHi: Option<Reg> = Some(Reg::CH);
    const Reg16Bit: Reg = Reg::CX;
    const Reg32Bit: Reg = Reg::ECX;
    const Reg64Bit: Reg = Reg::RCX;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Data;

impl Gpr for Data {
    const Reg8BitLo: Reg = Reg::DL;
    const Reg8BitHi: Option<Reg> = Some(Reg::DH);
    const Reg16Bit: Reg = Reg::DX;
    const Reg32Bit: Reg = Reg::EDX;
    const Reg64Bit: Reg = Reg::RDX;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Base;

impl Gpr for Base {
    const Reg8BitLo: Reg = Reg::BL;
    const Reg8BitHi: Option<Reg> = Some(Reg::BH);
    const Reg16Bit: Reg = Reg::BX;
    const Reg32Bit: Reg = Reg::EBX;
    const Reg64Bit: Reg = Reg::RBX;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackPointer;

impl Gpr for StackPointer {
    const Reg8BitLo: Reg = Reg::SPL;
    const Reg8BitHi: Option<Reg> = None;
    const Reg16Bit: Reg = Reg::SP;
    const Reg32Bit: Reg = Reg::ESP;
    const Reg64Bit: Reg = Reg::RSP;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BasePointer;

impl Gpr for BasePointer {
    const Reg8BitLo: Reg = Reg::BPL;
    const Reg8BitHi: Option<Reg> = None;
    const Reg16Bit: Reg = Reg::BP;
    const Reg32Bit: Reg = Reg::EBP;
    const Reg64Bit: Reg = Reg::RBP;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Source;

impl Gpr for Source {
    const Reg8BitLo: Reg = Reg::SIL;
    const Reg8BitHi: Option<Reg> = None;
    const Reg16Bit: Reg = Reg::SI;
    const Reg32Bit: Reg = Reg::ESI;
    const Reg64Bit: Reg = Reg::RSI;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Destination;

impl Gpr for Destination {
    const Reg8BitLo: Reg = Reg::DIL;
    const Reg8BitHi: Option<Reg> = None;
    const Reg16Bit: Reg = Reg::DI;
    const Reg32Bit: Reg = Reg::EDI;
    const Reg64Bit: Reg = Reg::RDI;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct R8Reg;

impl Gpr for R8Reg {
    const Reg8BitLo: Reg = Reg::R8b;
    const Reg8BitHi: Option<Reg> = None;
    const Reg16Bit: Reg = Reg::R8w;
    const Reg32Bit: Reg = Reg::R8d;
    const Reg64Bit: Reg = Reg::R8;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct R9Reg;

impl Gpr for R9Reg {
    const Reg8BitLo: Reg = Reg::R9b;
    const Reg8BitHi: Option<Reg> = None;
    const Reg16Bit: Reg = Reg::R9w;
    const Reg32Bit: Reg = Reg::R9d;
    const Reg64Bit: Reg = Reg::R9;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct R10Reg;

impl Gpr for R10Reg {
    const Reg8BitLo: Reg = Reg::R10b;
    const Reg8BitHi: Option<Reg> = None;
    const Reg16Bit: Reg = Reg::R10w;
    const Reg32Bit: Reg = Reg::R10d;
    const Reg64Bit: Reg = Reg::R10;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct R11Reg;

impl Gpr for R11Reg {
    const Reg8BitLo: Reg = Reg::R11b;
    const Reg8BitHi: Option<Reg> = None;
    const Reg16Bit: Reg = Reg::R11w;
    const Reg32Bit: Reg = Reg::R11d;
    const Reg64Bit: Reg = Reg::R11;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct R12Reg;

impl Gpr for R12Reg {
    const Reg8BitLo: Reg = Reg::R12b;
    const Reg8BitHi: Option<Reg> = None;
    const Reg16Bit: Reg = Reg::R12w;
    const Reg32Bit: Reg = Reg::R12d;
    const Reg64Bit: Reg = Reg::R12;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct R13Reg;

impl Gpr for R13Reg {
    const Reg8BitLo: Reg = Reg::R13b;
    const Reg8BitHi: Option<Reg> = None;
    const Reg16Bit: Reg = Reg::R13w;
    const Reg32Bit: Reg = Reg::R13d;
    const Reg64Bit: Reg = Reg::R13;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct R14Reg;

impl Gpr for R14Reg {
    const Reg8BitLo: Reg = Reg::R14b;
    const Reg8BitHi: Option<Reg> = None;
    const Reg16Bit: Reg = Reg::R14w;
    const Reg32Bit: Reg = Reg::R14d;
    const Reg64Bit: Reg = Reg::R14;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct R15Reg;

impl Gpr for R15Reg {
    const Reg8BitLo: Reg = Reg::R15b;
    const Reg8BitHi: Option<Reg> = None;
    const Reg16Bit: Reg = Reg::R15w;
    const Reg32Bit: Reg = Reg::R15d;
    const Reg64Bit: Reg = Reg::R15;
}

impl Reg {
    pub fn convert_with_opsize(self, op_size: &OpSize) -> Reg {
        match self {
            Reg::AL | Reg::AH | Reg::AX | Reg::EAX | Reg::RAX => Accumulator::from_opsize(op_size),
            Reg::CL | Reg::CH | Reg::CX | Reg::ECX | Reg::RCX => Counter::from_opsize(op_size),
            Reg::DL | Reg::DH | Reg::DX | Reg::EDX | Reg::RDX => Data::from_opsize(op_size),
            Reg::BL | Reg::BH | Reg::BX | Reg::EBX | Reg::RBX => Base::from_opsize(op_size),
            Reg::SPL | Reg::SP | Reg::ESP | Reg::RSP => StackPointer::from_opsize(op_size),
            Reg::BPL | Reg::BP | Reg::EBP | Reg::RBP => BasePointer::from_opsize(op_size),
            Reg::SIL | Reg::SI | Reg::ESI | Reg::RSI => Source::from_opsize(op_size),
            Reg::DIL | Reg::DI | Reg::EDI | Reg::RDI => Destination::from_opsize(op_size),
            Reg::R8b | Reg::R8w | Reg::R8d | Reg::R8 => R8Reg::from_opsize(op_size),
            Reg::R9b | Reg::R9w | Reg::R9d | Reg::R9 => R9Reg::from_opsize(op_size),
            Reg::R10b | Reg::R10w | Reg::R10d | Reg::R10 => R10Reg::from_opsize(op_size),
            Reg::R11b | Reg::R8w | Reg::R8d | Reg::R8 => R8Reg::from_opsize(op_size),
            Reg::R12b | Reg::R12w | Reg::R12d | Reg::R12 => R12Reg::from_opsize(op_size),
            Reg::R13b | Reg::R13w | Reg::R13d | Reg::R13 => R13Reg::from_opsize(op_size),
            Reg::R14b | Reg::R14w | Reg::R14d | Reg::R14 => R14Reg::from_opsize(op_size),
            Reg::R15b | Reg::R15w | Reg::R15d | Reg::R15 => R15Reg::from_opsize(op_size),
            // There are only control and special registers left, which should be put in a
            // different category
            _ => unreachable!(),
        }
    }
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

    // Convert the value to a register, specified by r/m32
    // A word general-purpose register or memory operand used for instructions whose operand-size
    // attribute is 16 bits. The word general-purpose registers are: EAX, ECX, EDX, EBX, ESP, EBP,
    // ESI, EDI. The contents of memory are found at the address provided by the effective address
    // computation. Word registers R8D - R15D are available using REX.R in 64-bit mode.
    pub fn from_rm64(value: u8) -> Self {
        // We make sure that value can have only the lower 3 bits set
        let value = value & 0b1111;
        match value {
            0 => Self::RAX,
            1 => Self::RCX,
            2 => Self::RDX,
            3 => Self::RBX,
            4 => Self::RSP,
            5 => Self::RBP,
            6 => Self::RSI,
            7 => Self::RDI,
            8 => Self::R8,
            9 => Self::R9,
            10 => Self::R10,
            11 => Self::R11,
            12 => Self::R12,
            13 => Self::R13,
            14 => Self::R14,
            15 => Self::R15,
            _ => unreachable!(),
        }
    }

    pub fn from_byte_with_arch(value: u8, maybe_arch: Option<Arch>) -> Self {
        let arch = match maybe_arch {
            Some(arch) => arch,
            None => Arch::Arch64,
        };

        match arch {
            Arch::Arch16 => Self::from_rm16(value),
            Arch::Arch32 => Self::from_rm32(value),
            Arch::Arch64 => Self::from_rm64(value),
        }
    }
}

