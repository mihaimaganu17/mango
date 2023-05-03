//! Module that acts as the core disassembler of the program
use crate::reader::{Reader, ReaderError};
use crate::prefix::Prefix;
use crate::opcode::{OpcodeIdent, OpcodeError};

#[derive(Debug)]
pub struct Disassembler;

#[derive(Debug)]
pub enum DisassemblerError {
    ReaderError(ReaderError),
    OpcodeError(OpcodeError),
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

impl Disassembler {
    pub fn parse(&self, reader: &mut Reader) -> Result<(), DisassemblerError> {
        while reader.pos() < 10 {
            let byte = reader.read::<u8>()?;

            let prefix = Prefix::from_byte(byte);

            println!("Prefix: {:?}", prefix);

            let opcode = OpcodeIdent::from_prefix_reader(prefix, reader)?;

            println!("Opcode: {:?}", opcode);
        }

        // First we try and read the prefix
        Ok(())
    }
}
