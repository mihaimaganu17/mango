//! Module that acts as the core disassembler of the program
use crate::inst::{Instruction, InstructionError};
use crate::modrm::Arch;
use crate::opcode::OpcodeError;
use crate::reader::{Reader, ReaderError};
use crate::stringify_opcode_type;

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
    pub fn parse(
        &self,
        reader: &mut Reader,
        maybe_arch: Option<Arch>,
    ) -> Result<(), DisassemblerError> {
        // Initialize a counter for how many instructions we have parsed
        let mut parser_insts = 0;
        while parser_insts < 20 && reader.bytes_unread() > 0 {
            let arch = if let Some(_) = maybe_arch {
                maybe_arch
            } else {
                Some(Arch::Arch64)
            };
            reader.start_recording()?;
            let instruction = Instruction::from_reader(reader, arch)?;
            parser_insts += 1;
            let read_bytes = reader.stop_recording()?;

            let hex_bytes = read_bytes.iter().fold(String::new(), |acc, x| format!("{acc}{x:02x} "));

            let ident = instruction.opcode.ident;

            println!(
                "{0: <30} {1: <10} {2: <10}",
                hex_bytes,
                stringify_opcode_type!(ident),
                instruction.operands,
            );
        }

        // First we try and read the prefix
        Ok(())
    }
}
