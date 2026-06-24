use bincode_next::{Decode, Encode};

use crate::RsvpError;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct ChunkLog {
    /// The length of the chunk  
    pub len: u32,
    /// The offset in bytes from the start of the file to the beginning of this chunk.
    pub offset: u32,
    /// The index of the first token in this chunk, relative to the entire token stream.
    pub first_token_index: u32,
}
impl ChunkLog {
    pub const SIZE: usize = size_of::<ChunkLog>();

    pub fn as_bytes(&self) -> [u8; ChunkLog::SIZE] {
        self.into()
    }
}
impl From<&ChunkLog> for [u8; ChunkLog::SIZE] {
    fn from(value: &ChunkLog) -> Self {
        let mut bytes = [0_u8; ChunkLog::SIZE];
        bytes[0..4].copy_from_slice(&value.len.to_le_bytes());
        bytes[4..8].copy_from_slice(&value.offset.to_le_bytes());
        bytes[8..12].copy_from_slice(&value.first_token_index.to_le_bytes());
        bytes
    }
}
impl TryFrom<&[u8]> for ChunkLog {
    type Error = RsvpError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < ChunkLog::SIZE {
            return Err(RsvpError::InvalidHeader);
        }
        Ok(Self {
            len: u32::from_le_bytes(value[0..4].try_into()?),
            offset: u32::from_le_bytes(value[4..8].try_into()?),
            first_token_index: u32::from_le_bytes(value[8..12].try_into()?),
        })
    }
}

pub struct ChunkedBlob<'a> {
    /// A reference to the data, either a file buffer or on flash
    file_data: &'a [u8],
    /// Raw bytes of the chunk logs, `chunk_count * size_of::<ChunkLog>`
    log_slice: &'a [u8],
    /// The index of the log
    chunk_count: u16,
    /// Hint to decompressors allocator
    max_raw_chunk_len: u32,
}
