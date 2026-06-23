#![no_std]

use bincode_next::{BorrowDecode, Encode};
use thiserror::Error;

pub type Result<T> = core::result::Result<T, RsvpError>;

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
    InvalidSlice(#[from] core::array::TryFromSliceError)
}


const MAGIC_VERSION: [u8; 8] = *b"%rsvp_rs";
const VERSION_V1: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressionType {
    None,
    Heatshrink,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents the header of an RSVP file.
pub struct RsvpHeader {
    /// The magic number and version of the RSVP file format. Usually, this is the bytes "%rsvp_rs".
    magic: u64,
    /// The version of the RSVP file format
    version: u8,
    /// The compression type used for the token stream
    compression: CompressionType,
    /// Reserved for future use. Should be set to 0.
    _reserved: u16,

    /// The number of tokens in the token stream. Purely for informational purposes, as the token stream is self-delimiting.
    token_count: u32,
    /// The offset in bytes from the start of the file to the beginning of the token stream.
    layout: BlobLayout
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobLayout{
    Empty,
    Single{
        token_offset: u32,
        token_stream_len: u32,
    },
    Chunked{
        chunk_count: u16,
        chunk_offsets: u32,
        max_chunk_length: u32,
    }
}

impl RsvpHeader {
    /// Creates a new RsvpHeader with default values.
    pub fn default() -> Self {
        Self {
            magic: u64::from_le_bytes(MAGIC_VERSION),
            version: VERSION_V1,
            compression: CompressionType::None,
            _reserved: 0,
            token_count: 0,
            layout: BlobLayout::Empty,
        }
    }
    /// Sets the compression type for the header.
    pub fn with_compression(mut self, compression: CompressionType) -> Self {
        self.compression = compression;
        self
    }
    /// Sets the token count for the header.
    pub fn with_token_count(mut self, token_count: u32) -> Self {
        self.token_count = token_count;
        self
    }
    /// Sets the token offset and token stream length for the header.
    pub fn with_token_offset_and_stream_len(
        mut self,
        token_offset: u32,
        token_stream_len: u32,
    ) -> Self {
        self.layout = BlobLayout::Single {
            token_offset,
            token_stream_len,
        };
        self
    }
    pub fn with_chunked_blob(
        mut self,
        chunk_count: u16,
        chunk_offsets: u32,
        max_chunk_length: u32,
    ) -> Self {
        self.layout = BlobLayout::Chunked {
            chunk_count,
            chunk_offsets,
            max_chunk_length,
        };
        self
    }
    /// Converts the RsvpHeader into a byte array representation.
    pub fn to_bytes(&self) -> [u8; core::mem::size_of::<RsvpHeader>()] {
        self.clone().into()
    }
}
impl From<RsvpHeader> for [u8; size_of::<RsvpHeader>()] {
    fn from(value: RsvpHeader) -> Self {
        let mut bytes = [0u8; size_of::<RsvpHeader>()];
        bytes[0..8].copy_from_slice(&value.magic.to_le_bytes());
        bytes[8] = value.version;
        bytes[9] = match value.compression {
            CompressionType::None => 0,
            CompressionType::Heatshrink => 1,
        };
        bytes[10..12].copy_from_slice(&value._reserved.to_le_bytes());
        bytes[12..16].copy_from_slice(&value.token_count.to_le_bytes());
        bytes[16..30].copy_from_slice(&match value.layout {
            BlobLayout::Empty => [0u8; 16],
            BlobLayout::Single {
                token_offset,
                token_stream_len,
            } => {
                let mut b = [0u8; 14];
                b[0] = 1;
                b[2..6].copy_from_slice(&token_offset.to_le_bytes());
                b[6..10].copy_from_slice(&token_stream_len.to_le_bytes());
                b
            }
            BlobLayout::Chunked {
                chunk_count,
                chunk_offsets,
                max_chunk_length,
            } => {
                let mut b = [0u8; 14];
                b[0] = 2;
                b[2..4].copy_from_slice(&chunk_count.to_le_bytes());
                b[4..8].copy_from_slice(&chunk_offsets.to_le_bytes());
                b[8..12].copy_from_slice(&max_chunk_length.to_le_bytes());
                b
            }
        }.to_le_bytes());
        bytes
    }
}
impl TryFrom<&[u8]> for RsvpHeader {
    type Error = RsvpError;
    fn try_from(value: &[u8]) -> Result<Self> {
        if value.len() < size_of::<RsvpHeader>() {
            return Err(RsvpError::InvalidHeader);
        }
        if &value[0..8] != MAGIC_VERSION {
            return Err(RsvpError::InvalidMagic);
        }
        if value[8] != VERSION_V1 {
            return Err(RsvpError::UnsupportedVersion);
        }
        Ok(Self {
            magic: u64::from_le_bytes(value[0..8].try_into()?),
            version: value[8],
            compression: match value[9] {
                0 => CompressionType::None,
                1 => CompressionType::Heatshrink,
                _ => return Err(RsvpError::InvalidHeader),
            },
            _reserved: u16::from_le_bytes(value[10..12].try_into()?),
            token_count: u32::from_le_bytes(value[12..16].try_into()?),
            layout: match value[16] {
                0 => BlobLayout::Empty,
                1 => BlobLayout::Single {
                    token_offset: u32::from_le_bytes(value[18..22].try_into()?),
                    token_stream_len: u32::from_le_bytes(value[22..26].try_into()?),
                },
                2 => BlobLayout::Chunked {
                    chunk_count: u16::from_le_bytes(value[18..20].try_into()?),
                    chunk_offsets: u32::from_le_bytes(value[20..24].try_into()?),
                    max_chunk_length: u32::from_le_bytes(value[24..28].try_into()?),
                },
                _ => return Err(RsvpError::InvalidHeader),
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata<'a> {
    author: &'a str,
    title: &'a str,
    date: &'a str,
    publication: &'a str,
    description: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkLog{
    /// The length of the chunk  
    len: u32,
    /// The offset in bytes from the start of the file to the beginning of this chunk.
    offset: u32,
    /// The index of the first token in this chunk, relative to the entire token stream.
    first_token_index: u32,
}


pub struct ChunkedBlob<'a>{
    /// A reference to the data, either a file buffer or on flash
    data: &'a [u8],
    /// Raw bytes of the chunk logs, `chunk_count * size_of::<ChunkLog>` 
    index: &'a [u8],
    chunk_count: u16,
    /// Hint to decompressors allocator
    max_raw_chunk_len: u32,
    compression: CompressionType
}



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RsvpFile {
    header: RsvpHeader,
    // frames: &'a [Frame<'a>],
}

pub enum DelayKind {
    None,
    Short,
    Medium,
    Long,
}
#[derive(Encode, BorrowDecode, Debug, Clone, PartialEq, Eq)]
pub enum Affix {
    None,
    Comma,
    Period,
    Question,
    Exclamation,
    Colon,
    Semicolon,
    Ellipsis,
}
impl From<Affix> for DelayKind {
    fn from(affix: Affix) -> Self {
        match affix {
            Affix::None => DelayKind::None,
            Affix::Comma | Affix::Colon | Affix::Semicolon => DelayKind::Short,
            Affix::Period | Affix::Question | Affix::Exclamation => DelayKind::Medium,
            Affix::Ellipsis => DelayKind::Long,
        }
    }
}

#[derive(Encode, BorrowDecode, Debug, Clone, PartialEq, Eq)]
pub struct WordToken<'a> {
    pub prefix: Affix,
    pub text: &'a str,
    pub suffix: Affix,
}

#[derive(Encode, BorrowDecode, Debug, Clone, PartialEq, Eq)]
pub enum TokenKind<'a> {
    Word(WordToken<'a>),
    ParagraphBreak,
    LineBreak,
    SectionBreak,
}
