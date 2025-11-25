//! GRBN header parser for Warcraft III: Reforged replay files.
//!
//! The GRBN format was introduced in Warcraft III: Reforged (patch 1.32+).
//! It uses a 128-byte header followed by a single continuous zlib stream.
//!
//! # Header Layout (128 bytes)
//!
//! | Offset | Size | Field | Description |
//! |--------|------|-------|-------------|
//! | 0x00 | 4 | `magic` | "GRBN" (0x4752424E) |
//! | 0x04 | 4 | `version` | Format version (always 2) |
//! | 0x08 | 4 | `unknown_1` | Unknown (always 11) |
//! | 0x0C | 4 | `unknown_2` | Unknown (always 51200) |
//! | 0x10 | 8 | `reserved_1` | Reserved (zeros) |
//! | 0x18 | 4 | `unknown_3` | Unknown (0-6 observed) |
//! | 0x1C | 4 | `unknown_4` | Unknown (0 or 1) |
//! | 0x20 | 4 | `reserved_2` | Reserved (zeros) |
//! | 0x24 | 4 | `decompressed_size` | Size after decompression |
//! | 0x28 | 88 | `reserved_3` | Reserved (zeros) |
//!
//! Data starts at offset 0x80 (128 bytes).

use crate::binary::{read_bytes, read_u32_le};
use crate::error::{ParserError, Result};
use crate::format::GRBN_MAGIC;

/// The size of a GRBN header in bytes.
pub const GRBN_HEADER_SIZE: usize = 128;

/// The byte offset where compressed data begins in GRBN format.
pub const GRBN_DATA_OFFSET: usize = 0x80;

/// The expected version number in GRBN headers.
pub const GRBN_EXPECTED_VERSION: u32 = 2;

/// Parsed header for GRBN (Reforged) format replay files.
///
/// This struct contains all fields from the 128-byte GRBN header.
/// Fields marked as "unknown" have consistent patterns but their
/// exact purpose is not yet determined.
///
/// # Example
///
/// ```no_run
/// use w3g_parser::header::grbn::GrbnHeader;
///
/// let data = std::fs::read("replay.w3g").unwrap();
/// let header = GrbnHeader::parse(&data).unwrap();
/// println!("Decompressed size: {} bytes", header.decompressed_size);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrbnHeader {
    /// Magic bytes "GRBN" (0x4752424E) at offset 0x00.
    pub magic: [u8; 4],

    /// Format version at offset 0x04 (always 2 in observed files).
    pub version: u32,

    /// Unknown field at offset 0x08 (always 11 in observed files).
    /// Possibly a sub-version or feature flag.
    pub unknown_1: u32,

    /// Unknown field at offset 0x0C (always 51200 in observed files).
    /// Possibly a buffer size or chunk size hint.
    pub unknown_2: u32,

    /// Unknown field at offset 0x18 (observed values: 0-6).
    /// Possibly player count or game mode indicator.
    pub unknown_3: u32,

    /// Unknown field at offset 0x1C (observed values: 0 or 1).
    /// Possibly a boolean flag.
    pub unknown_4: u32,

    /// Total size of decompressed replay data at offset 0x24.
    /// This is the expected size after decompressing the zlib stream.
    pub decompressed_size: u32,
}

impl GrbnHeader {
    /// Parses a GRBN header from raw bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes of the replay file (at least 128 bytes required)
    ///
    /// # Returns
    ///
    /// A `GrbnHeader` struct containing all header fields.
    ///
    /// # Errors
    ///
    /// - `ParserError::UnexpectedEof` if data is shorter than 128 bytes
    /// - `ParserError::InvalidMagic` if the magic bytes are not "GRBN"
    ///
    /// # Example
    ///
    /// ```no_run
    /// use w3g_parser::header::grbn::GrbnHeader;
    ///
    /// let data = std::fs::read("replay.w3g").unwrap();
    /// let header = GrbnHeader::parse(&data)?;
    /// # Ok::<(), w3g_parser::error::ParserError>(())
    /// ```
    pub fn parse(data: &[u8]) -> Result<Self> {
        // Check minimum size
        if data.len() < GRBN_HEADER_SIZE {
            return Err(ParserError::unexpected_eof(GRBN_HEADER_SIZE, data.len()));
        }

        // Read and validate magic
        let magic_slice = read_bytes(data, 0x00, 4)?;
        if magic_slice != GRBN_MAGIC {
            return Err(ParserError::invalid_magic(GRBN_MAGIC, magic_slice));
        }

        let mut magic = [0u8; 4];
        magic.copy_from_slice(magic_slice);

        // Read all fields
        let version = read_u32_le(data, 0x04)?;
        let unknown_1 = read_u32_le(data, 0x08)?;
        let unknown_2 = read_u32_le(data, 0x0C)?;
        // 0x10-0x17: reserved_1 (8 bytes of zeros)
        let unknown_3 = read_u32_le(data, 0x18)?;
        let unknown_4 = read_u32_le(data, 0x1C)?;
        // 0x20-0x23: reserved_2 (4 bytes of zeros)
        let decompressed_size = read_u32_le(data, 0x24)?;
        // 0x28-0x7F: reserved_3 (88 bytes of zeros)

        Ok(GrbnHeader {
            magic,
            version,
            unknown_1,
            unknown_2,
            unknown_3,
            unknown_4,
            decompressed_size,
        })
    }

    /// Returns the byte offset where compressed data begins.
    ///
    /// For GRBN format, this is always 0x80 (128 bytes).
    #[must_use]
    pub const fn data_offset(&self) -> usize {
        GRBN_DATA_OFFSET
    }

    /// Returns the total header size in bytes.
    ///
    /// For GRBN format, this is always 128 bytes.
    #[must_use]
    pub const fn header_size(&self) -> usize {
        GRBN_HEADER_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a minimal valid GRBN header for testing.
    fn create_test_header() -> Vec<u8> {
        let mut header = vec![0u8; GRBN_HEADER_SIZE];

        // Magic "GRBN"
        header[0x00..0x04].copy_from_slice(b"GRBN");

        // Version = 2
        header[0x04..0x08].copy_from_slice(&2u32.to_le_bytes());

        // Unknown_1 = 11
        header[0x08..0x0C].copy_from_slice(&11u32.to_le_bytes());

        // Unknown_2 = 51200
        header[0x0C..0x10].copy_from_slice(&51200u32.to_le_bytes());

        // Reserved_1 (0x10-0x17) - already zeros

        // Unknown_3 = 2
        header[0x18..0x1C].copy_from_slice(&2u32.to_le_bytes());

        // Unknown_4 = 1
        header[0x1C..0x20].copy_from_slice(&1u32.to_le_bytes());

        // Reserved_2 (0x20-0x23) - already zeros

        // Decompressed size = 1_000_000
        header[0x24..0x28].copy_from_slice(&1_000_000u32.to_le_bytes());

        // Reserved_3 (0x28-0x7F) - already zeros

        header
    }

    #[test]
    fn test_parse_valid_header() {
        let data = create_test_header();
        let header = GrbnHeader::parse(&data).unwrap();

        assert_eq!(&header.magic, b"GRBN");
        assert_eq!(header.version, 2);
        assert_eq!(header.unknown_1, 11);
        assert_eq!(header.unknown_2, 51200);
        assert_eq!(header.unknown_3, 2);
        assert_eq!(header.unknown_4, 1);
        assert_eq!(header.decompressed_size, 1_000_000);
    }

    #[test]
    fn test_parse_too_short() {
        let data = vec![0u8; 64]; // Less than 128 bytes
        let result = GrbnHeader::parse(&data);

        assert!(matches!(
            result,
            Err(ParserError::UnexpectedEof {
                expected: 128,
                available: 64
            })
        ));
    }

    #[test]
    fn test_parse_invalid_magic() {
        let mut data = create_test_header();
        data[0..4].copy_from_slice(b"BAD!");

        let result = GrbnHeader::parse(&data);
        assert!(matches!(result, Err(ParserError::InvalidMagic { .. })));
    }

    #[test]
    fn test_data_offset() {
        let data = create_test_header();
        let header = GrbnHeader::parse(&data).unwrap();

        assert_eq!(header.data_offset(), 0x80);
        assert_eq!(header.data_offset(), 128);
    }

    #[test]
    fn test_header_size() {
        let data = create_test_header();
        let header = GrbnHeader::parse(&data).unwrap();

        assert_eq!(header.header_size(), 128);
    }

    #[test]
    fn test_constants() {
        assert_eq!(GRBN_HEADER_SIZE, 128);
        assert_eq!(GRBN_DATA_OFFSET, 0x80);
        assert_eq!(GRBN_EXPECTED_VERSION, 2);
    }

    #[test]
    fn test_header_with_extra_data() {
        // Header parsing should work even with extra data appended
        let mut data = create_test_header();
        data.extend_from_slice(&[0x78, 0x9C, 0xCD, 0x96]); // Fake zlib data

        let header = GrbnHeader::parse(&data).unwrap();
        assert_eq!(&header.magic, b"GRBN");
        assert_eq!(header.decompressed_size, 1_000_000);
    }
}
