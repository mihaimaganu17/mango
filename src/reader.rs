//! Module that implements a safe reader for a byte slice/sequence
pub struct Reader {
    // Current position in the buffer that is backing this `Reader`
    pos: usize,
    // Buffer used to read data from
    bytes: Vec<u8>,
}

