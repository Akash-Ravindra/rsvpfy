#![no_std]

use bincode_next::config::{self, Configuration};
use thiserror::Error;

pub type RsvpResult<T> = core::result::Result<T, RsvpError>;
pub const BINCODE_CONFIG: Configuration = config::standard();

#[derive(Error, Debug)]
/// Errors that can occur when reading or writing RSVP files.
pub enum RsvpError {
    #[error("The file does not start with the expected magic bytes.")]
    InvalidMagic,

    #[error("The file has a version that is not supported by this library.")]
    UnsupportedVersion,

    #[error("The file is too short to contain a valid header.")]
    UnexpectedEof,

    #[error("The file contains invalid UTF-8 data.")]
    InvalidUtf8,

    #[error("The file contains an invalid header.")]
    InvalidHeader,

    #[error("Unable to convert slice to array.")]
    InvalidSlice(#[from] core::array::TryFromSliceError),

    #[error("Bincode Deser failure")]
    DecodeError(#[from] bincode_next::error::DecodeError),
}

pub mod chunk;
pub mod header;
pub mod token;

#[cfg(test)]
pub(crate) mod test {
    use bincode_next::encode_into_slice;

    use crate::{
        BINCODE_CONFIG,
        chunk::ChunkLog,
        header::{
            RsvpHeader,
            test::{generate_header, generate_metadata},
        },
    };

    extern crate std;
    use std::vec::Vec;

    #[test]
    fn test_create_fake_file() {
        let mut header = generate_header();
        let metadata = generate_metadata();

        let chunks = [[0xde; 1024]; 10];

        let mut buf = Vec::new();
        buf.extend_from_slice(&[0u8; RsvpHeader::HEADER_LEN]);
        buf.resize(1024, 0);
        encode_into_slice(&metadata, &mut buf, BINCODE_CONFIG).expect("Failed to encode metadata");

        let chunk_offset = RsvpHeader::HEADER_LEN + buf.len();
        let mut chunk_data_offset = chunk_offset + (ChunkLog::SIZE * chunks.len());

        header.with_chunked_blob(
            chunks.len() as u16,
            chunk_offset as u32,
            chunks[0].len() as u32,
        );
        buf[..RsvpHeader::HEADER_LEN].copy_from_slice(&header.to_bytes());

        for (index, chunk) in chunks.iter().enumerate() {
            buf.extend_from_slice(
                &ChunkLog {
                    len: chunk.len() as u32,
                    offset: chunk_data_offset as u32,
                    first_token_index: index as u32,
                }
                .as_bytes(),
            );
            chunk_data_offset += chunk.len();
        }
        for chunk in &chunks {
            buf.extend_from_slice(chunk);
        }
    }
}
