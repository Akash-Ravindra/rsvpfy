use bincode_next::{BorrowDecode, Encode, borrow_decode_from_slice};

use crate::{BINCODE_CONFIG, RsvpError, RsvpResult};

const MAGIC_VERSION: [u8; 8] = *b"%rsvp_rs";
const VERSION_V1: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CompressionType {
    #[default]
    None,
    Heatshrink,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
    layout: BlobLayout,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// Enumerates the various layouts of the tokens
pub enum BlobLayout {
    #[default]
    Empty,
    Single {
        token_offset: u32,
        token_stream_len: u32,
    },
    Chunked {
        chunk_count: u16,
        chunk_offsets: u32,
        max_chunk_length: u32,
    },
}

impl RsvpHeader {
    pub const HEADER_LEN: usize = 32;
    pub fn new() -> Self {
        Self {
            magic: u64::from_le_bytes(MAGIC_VERSION),
            version: VERSION_V1,
            ..Default::default()
        }
    }
    /// Sets the compression type for the header.
    pub fn with_compression(&mut self, compression: CompressionType) {
        self.compression = compression;
    }
    /// Sets the token count for the header.
    pub fn with_token_count(&mut self, token_count: u32) {
        self.token_count = token_count;
    }
    /// Sets the token offset and token stream length for the header.
    pub fn with_token_offset_and_stream_len(&mut self, token_offset: u32, token_stream_len: u32) {
        self.layout = BlobLayout::Single {
            token_offset,
            token_stream_len,
        }
    }
    pub fn with_chunked_blob(
        &mut self,
        chunk_count: u16,
        chunk_offsets: u32,
        max_chunk_length: u32,
    ) {
        self.layout = BlobLayout::Chunked {
            chunk_count,
            chunk_offsets,
            max_chunk_length,
        }
    }
    /// Converts the RsvpHeader into a byte array representation.
    pub fn to_bytes(&self) -> [u8; RsvpHeader::HEADER_LEN] {
        self.into()
    }
}
impl From<&RsvpHeader> for [u8; RsvpHeader::HEADER_LEN] {
    fn from(value: &RsvpHeader) -> Self {
        let mut bytes = [0u8; RsvpHeader::HEADER_LEN];
        bytes[0..8].copy_from_slice(&value.magic.to_le_bytes());
        bytes[8] = value.version;
        bytes[9] = match value.compression {
            CompressionType::None => 0,
            CompressionType::Heatshrink => 1,
        };
        bytes[10..12].copy_from_slice(&value._reserved.to_le_bytes());
        bytes[12..16].copy_from_slice(&value.token_count.to_le_bytes());
        bytes[16..32].copy_from_slice(&match value.layout {
            BlobLayout::Empty => [0u8; 16],
            BlobLayout::Single {
                token_offset,
                token_stream_len,
            } => {
                let mut b = [0u8; 16];
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
                let mut b = [0u8; 16];
                b[0] = 2;
                b[2..4].copy_from_slice(&chunk_count.to_le_bytes());
                b[4..8].copy_from_slice(&chunk_offsets.to_le_bytes());
                b[8..12].copy_from_slice(&max_chunk_length.to_le_bytes());
                b
            }
        });
        bytes
    }
}
impl TryFrom<&[u8]> for RsvpHeader {
    type Error = RsvpError;
    fn try_from(value: &[u8]) -> RsvpResult<Self> {
        if value.len() < RsvpHeader::HEADER_LEN {
            return Err(RsvpError::InvalidHeader);
        }
        if value[0..8] != MAGIC_VERSION {
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
            },
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Encode, BorrowDecode)]
pub struct Metadata<'a> {
    author: &'a str,
    title: &'a str,
    date: &'a str,
    publication: &'a str,
    description: &'a str,
}
impl<'a> Metadata<'a> {
    pub fn with_author(&mut self, author: &'a str) {
        self.author = author;
    }
    pub fn with_title(&mut self, title: &'a str) {
        self.title = title
    }
    pub fn with_date(&mut self, date: &'a str) {
        self.date = date
    }
    pub fn with_publication(&mut self, publication: &'a str) {
        self.publication = publication
    }
    pub fn with_description(&mut self, description: &'a str) {
        self.description = description;
    }
    pub fn seek_metadata(slice: &'a [u8]) -> RsvpResult<Metadata<'a>> {
        borrow_decode_from_slice(&slice[RsvpHeader::HEADER_LEN..], BINCODE_CONFIG)
            .map(|res| res.0)
            .map_err(RsvpError::from)
    }
}

#[cfg(test)]
pub(crate) mod test {
    extern crate std;
    use std::vec::Vec;

    use super::*;

    pub fn generate_header() -> RsvpHeader {
        let mut header = RsvpHeader::new();
        header.with_compression(CompressionType::Heatshrink);
        header.with_token_count(0xff);
        header.with_token_offset_and_stream_len(0x50, 0x1000);
        header
    }
    pub fn generate_metadata() -> Metadata<'static> {
        Metadata {
            author: "Andy Weir",
            title: "The Martian",
            date: "September 2011",
            publication: "Crown Publishing Group",
            description: "In the year 2035,[7] the crew of NASA's Ares 3 mission have arrived 
            at Acidalia Planitia for a planned month-long stay on Mars.
            After only six sols, a dust and wind storm threatens to topple their Mars 
            Ascent Vehicle (MAV), which would trap them on the planet. During the hurried
            evacuation, an antenna tears loose and impales astronaut Mark Watney, a botanist
            and engineer, also disabling his spacesuit radio. He is flung out of sight by 
            the wind and presumed dead. As the MAV teeters dangerously, mission commander 
            Melissa Lewis has no choice but to take off without completing the search for Watney.",
        }
    }

    #[test]
    fn test_header_building() {
        let header = generate_header();
        let test_header = RsvpHeader {
            magic: u64::from_le_bytes(MAGIC_VERSION),
            version: 1,
            compression: CompressionType::Heatshrink,
            _reserved: 0,
            token_count: 0xff,
            layout: BlobLayout::Single {
                token_offset: 0x50,
                token_stream_len: 0x1000,
            },
        };
        assert_eq!(header, test_header);
    }
    #[test]
    fn test_serialization() {
        let header_serialized = generate_header().to_bytes();
        let deserialized_header: RsvpHeader = header_serialized
            .as_slice()
            .try_into()
            .expect("Unable to convert slice to header");
        assert_eq!(generate_header(), deserialized_header)
    }

    #[test]
    fn test_header_with_metadata() {
        let header = generate_header();
        let metadata = generate_metadata();

        let mut buffer = Vec::new();
        buffer.extend_from_slice(header.to_bytes().as_ref());
        buffer.resize(RsvpHeader::HEADER_LEN + 1024, 0);
        bincode_next::encode_into_slice(
            &metadata,
            &mut buffer[RsvpHeader::HEADER_LEN..],
            BINCODE_CONFIG,
        )
        .expect("Unable to encode the metadata");

        let deserialized_header: RsvpHeader = buffer
            .as_slice()
            .try_into()
            .expect("Unable to convert slice to header");
        assert_eq!(deserialized_header, generate_header());
        let deserialized_metadata =
            Metadata::seek_metadata(&buffer).expect("Unable to deserialize metadata");
        assert_eq!(deserialized_metadata, metadata);
    }
}
