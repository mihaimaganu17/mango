//! Module that acts as the core disassembler of the program
use crate::reader::{Reader, ReaderError};
use crate::prefix::Prefix;

#[derive(Debug)]
pub struct Disassembler;

#[derive(Debug)]
pub enum DisassemblerError {
    ReaderError(ReaderError),
}

impl From<ReaderError> for DisassemblerError {
    fn from(value: ReaderError) -> Self {
        Self::ReaderError(value)
    }
}

impl Disassembler {
    pub fn parse(&self, reader: &mut Reader) -> Result<(), DisassemblerError> {
        while reader.pos() < 10 {
            let byte = reader.read::<u8>()?;
        }

        // First we try and read the prefix
        Ok(())
    }
}
