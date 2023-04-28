//! Module that implements a safe reader for a byte slice/sequence
pub struct Reader {
    // Current position in the buffer that is backing this `Reader`
    pos: usize,
    // Buffer used to read data from
    bytes: Vec<u8>,
}

impl Reader {
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self {
            pos: 0,
            bytes,
        }
    }

    /// Reads `size` bytes from the buffer that back this `Reader`
    pub fn read_bytes(&mut self, size: usize) -> Option<&[u8]> {
        // Try and read the desired bytes
        let bytes_read = self.bytes.get(self.pos..self.pos + size)?;

        // If we successfully read the bytes, we move the pointer by `size`
        self.pos += size;

        // Return the read bytes
        Some(bytes_read)
    }

    pub fn read<T: FromLeBytes>() -> Option<T> {
        let bytes 
    }
}

/// Trait that needs to be implemented by types that want to be constructed from the `Reader` data
pub trait FromLeBytes {
    fn from_le_bytes() -> Self;
}

impl FromLeBytes for u64 {
    fn from_le_bytes() -> Self {

    }
}


