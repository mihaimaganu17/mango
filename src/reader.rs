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

    pub fn read<T: FromLeBytes>(&mut self) -> Option<T> {
        let nbytes = std::mem::size_of::<T>();
        let bytes = self.read_bytes(nbytes)?;
        T::from_le_bytes(bytes)
    }
}

/// Trait that needs to be implemented by types that want to be constructed from the `Reader` data
pub trait FromLeBytes: Sized {
    fn from_le_bytes(bytes: &[u8]) -> Option<Self>;
}

#[macro_export]
macro_rules! read_type {
    ($ty:ty) => {
        impl FromLeBytes for $ty {
            fn from_le_bytes(bytes: &[u8]) -> Option<Self> {
                Some(Self::from_le_bytes(bytes.try_into().ok()?))
            }
        }
    }
}

read_type!(u8);
read_type!(u16);
read_type!(u32);
read_type!(u64);
read_type!(u128);

read_type!(i8);
read_type!(i16);
read_type!(i32);
read_type!(i64);
read_type!(i128);
