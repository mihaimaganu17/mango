//! Module that handles x86_64 Instruction Prefixes parsing

/// Represents instruction prefixes of 1 byte each. They are divided into four groups, each
/// with a set of allowable prefix codes.
pub enum Prefix {
    // Lock, repeat and BND prefixes
    Group1(Group1), 
    // Segment Override prefixes
    Group2(Group2),
    // Operand-size override, allows a program to switch between 16-bit and 32-bit operand sizes.
    OpSize,
    // Address-size override, allows a program to switch between 16-bit and 32-bit addressing
    AddrSize,
}

#[derive(Debug)]
pub enum PrefixError {
    InvalidPrefix,
}

#[derive(Debug)]
pub enum Group1 {
    // Forces an operation that ensures exclusive use of shared memory in a multiprocessor
    // environment.
    Lock,
    // Represents REPNE(Repeat Not Equal)/REPNZ(Repeat Not Zero). Repeat Not Zero prefix applies
    // only to string and I/O instructions. This can also be the BND prefix if certain conditions
    // are met.
    RepNE,
    // The Repeat prefix applies only to string an I/O isntructions.
    Rep,
}

impl TryFrom<u8> for Group1 {
    type Error = PrefixError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            prefix_code::LOCK => Ok(Self::Lock), 
            prefix_code::REPNE => Ok(Self::RepNE), 
            prefix_code::REP => Ok(Self::Rep), 
            _ => Err(PrefixError::InvalidPrefix),
        }
    }
}

#[derive(Debug)]
pub enum Group2 {
    // CS Segment override(used with any branch instruction) or
    // Branch not taken(on older microarchitectures, used only with Jcc instructions)
    CsSegOverride,
    // SS Segment override(used with any branch instruction)
    SsSegOverride,
    // DS Segment override(used with any branch instruction) or
    // Branch not taken(on older microarchitectures, used only with Jcc instructions)
    DsSegOverride,
    // ES Segment override(used with any branch instruction)
    EsSegOverride,
    // FS Segment override(used with any branch instruction)
    FsSegOverride,
    // GS Segment override(used with any branch instruction)
    GsSegOverride,
}

impl TryFrom<u8> for Group2 {
    type Error = PrefixError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            prefix_code::CS_SEG_OVERRIDE => Ok(Self::CsSegOverride),
            prefix_code::SS_SEG_OVERRIDE => Ok(Self::SsSegOverride),
            prefix_code::DS_SEG_OVERRIDE => Ok(Self::DsSegOverride),
            prefix_code::ES_SEG_OVERRIDE => Ok(Self::EsSegOverride),
            prefix_code::FS_SEG_OVERRIDE => Ok(Self::FsSegOverride),
            prefix_code::GS_SEG_OVERRIDE => Ok(Self::GsSegOverride),
            _ => Err(PrefixError::InvalidPrefix),
        }
    }
}

mod prefix_code {
    pub const LOCK: u8 = 0xF0;
    pub const REPNE: u8 = 0xF2;
    pub const REP: u8 = 0xF3;
    pub const CS_SEG_OVERRIDE: u8 = 0x2E;
    pub const SS_SEG_OVERRIDE: u8 = 0x36;
    pub const DS_SEG_OVERRIDE: u8 = 0x3E;
    pub const ES_SEG_OVERRIDE: u8 = 0x26;
    pub const FS_SEG_OVERRIDE: u8 = 0x64;
    pub const GS_SEG_OVERRIDE: u8 = 0x65;
    pub const OP_SIZE_OVERRIDE: u8 = 0x66;
    pub const ADDR_SIZE_OVERRIDE: u8 = 0x67;
}
