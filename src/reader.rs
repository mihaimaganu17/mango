//! Module that implements a safe reader for a byte slice/sequence
use core::array::TryFromSliceError;

pub struct Reader {
    // Current position in the buffer that is backing this `Reader`
    pos: usize,
    // Buffer used to read data from
    bytes: Vec<u8>,
}

/// General error raised when one of the `Reader` methods fails
#[derive(Debug)]
pub enum ReaderError {
    NotEnoughBytes,
    TryFromSliceError(TryFromSliceError),
}

impl From<TryFromSliceError> for ReaderError {
    fn from(value: TryFromSliceError) -> Self {
        Self::TryFromSliceError(value)
    }
}

impl Reader {
    /// Create a new `Reader` from a vector of bytes
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self { pos: 0, bytes }
    }

    pub fn bytes_unread(&self) -> usize {
        self.bytes.len() - self.pos
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Reads `size` bytes from the buffer that back this `Reader` and moves the buffer pointer
    /// forward by `size` bytes
    pub fn read_bytes(&mut self, size: usize) -> Result<&[u8], ReaderError> {
        // Try and read the desired bytes
        let bytes_read = self
            .bytes
            .get(self.pos..self.pos + size)
            .ok_or(ReaderError::NotEnoughBytes)?;

        // If we successfully read the bytes, we move the pointer by `size`
        self.pos += size;

        // Return the read bytes
        Ok(bytes_read)
    }

    /// Reads `size` bytes from the buffer that back this `Reader`, without moving the buffer
    /// pointer forward
    // Maybe we can have a buffered read, that reads chunks and whenever we need to peak at a byte,
    // we just return the element and the cursor, without re-reading it
    pub fn peek_bytes(&self, size: usize) -> Result<&[u8], ReaderError> {
        // Try and read the desired bytes
        let bytes_read = self
            .bytes
            .get(self.pos..self.pos + size)
            .ok_or(ReaderError::NotEnoughBytes)?;

        // Return the read bytes
        Ok(bytes_read)
    }

    /// Reads a `T` type from the underlying bytes
    ///
    /// # Errors
    ///
    /// Fails if there are not enough bytes in the buffer
    pub fn read<T: FromLeBytes>(&mut self) -> Result<T, ReaderError> {
        let nbytes = std::mem::size_of::<T>();
        let bytes = self.read_bytes(nbytes)?;
        T::from_bytes(bytes)
    }

    /// Peek a `T` type from the underlying bytes
    ///
    /// # Errors
    ///
    /// Fails if there are not enough bytes in the buffer
    pub fn peek<T: FromLeBytes>(&self) -> Result<T, ReaderError> {
        let nbytes = std::mem::size_of::<T>();
        let bytes = self.peek_bytes(nbytes)?;
        T::from_bytes(bytes)
    }
}

/// Trait that needs to be implemented by types that want to be constructed from the `Reader` data
pub trait FromLeBytes: Sized {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ReaderError>;
}

// Macro that implements the trait `FromLeBytes` for every primitive numerical type. Namely:
// u8, u16, u32, u64, u128, i8, i16, i32, i64, i128
#[macro_export]
macro_rules! read_type {
    ($ty:ty) => {
        impl FromLeBytes for $ty {
            fn from_bytes(bytes: &[u8]) -> Result<Self, ReaderError> {
                Ok(Self::from_le_bytes(bytes.try_into()?))
            }
        }
    };
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
