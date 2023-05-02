//! Module that acts as the core disassembler of the program
use crate::reader::Reader;
use crate::prefix::Prefix;

pub struct Disassembler;

pub enum DisassemblerError {
    ReaderError,
}

impl Disassembler {
    pub fn parse(reader: &mut Reader) -> Result<(), DisassemblerError> {
        // First we try and read the prefix
        Ok(())
    }
}
