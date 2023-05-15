//! Module that acts as the core disassembler of the program
use crate::reader::{Reader, ReaderError};
use crate::opcode::OpcodeError;
use crate::inst::{Instruction, InstructionError};
use crate::modrm::Arch;

#[derive(Debug)]
pub struct Disassembler;

#[derive(Debug)]
pub enum DisassemblerError {
    ReaderError(ReaderError),
    OpcodeError(OpcodeError),
    InstructionError(InstructionError),
}

impl From<ReaderError> for DisassemblerError {
    fn from(value: ReaderError) -> Self {
        Self::ReaderError(value)
    }
}

impl From<OpcodeError> for DisassemblerError {
    fn from(value: OpcodeError) -> Self {
        Self::OpcodeError(value)
    }
}

impl From<InstructionError> for DisassemblerError {
    fn from(value: InstructionError) -> Self {
        Self::InstructionError(value)
    }
}

impl Disassembler {
    pub fn parse(&self, reader: &mut Reader) -> Result<(), DisassemblerError> {
        while reader.pos() < 20 {
            let arch = Some(Arch::Arch64);
            let instruction = Instruction::from_reader(reader, arch)?;

            println!("Instruction: {:?}", instruction);
        }

        // First we try and read the prefix
        Ok(())
    }
}
