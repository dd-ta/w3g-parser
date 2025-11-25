//! Classic header parser for original Warcraft III replay files.
//!
//! The Classic format was used by Warcraft III: Reign of Chaos and
//! The Frozen Throne (pre-patch 1.32). It uses a 68-byte header
//! followed by block-based zlib compression.
//!
//! # Header Layout (68 bytes)
//!
//! | Offset | Size | Field | Description |
//! |--------|------|-------|-------------|
//! | 0x00 | 28 | `magic` | "Warcraft III recorded game\x1A\x00" |
//! | 0x1C | 4 | `header_size` | Always 68 (0x44) |
//! | 0x20 | 4 | `file_size` | Total file size in bytes |
//! | 0x24 | 4 | `header_version` | Always 1 |
//! | 0x28 | 4 | `decompressed_size` | Size after decompression |
//! | 0x2C | 4 | `block_count` | Number of data blocks |
//! | 0x30 | 4 | `sub_header_magic` | "PX3W" (W3XP reversed, TFT marker) |
//! | 0x34 | 4 | `build_version` | 26 (Type A) or 10000+ (Type B) |
//! | 0x38 | 4 | `flags` | Unknown flags (high bit set) |
//! | 0x3C | 4 | `duration_ms` | Game duration in milliseconds |
//! | 0x40 | 4 | `checksum` | Unknown checksum |
//!
//! Data blocks start at offset 0x44 (68 bytes).
//!
//! # Block Format Variants
//!
//! The build version field determines the block header format:
//! - **Type A** (build < 10000): 8-byte block headers
//! - **Type B** (build >= 10000): 12-byte block headers

use crate::binary::{read_bytes, read_u32_le};
use crate::error::{ParserError, Result};
use crate::format::{ClassicVersion, CLASSIC_MAGIC, CLASSIC_TYPE_B_THRESHOLD};

/// The size of a Classic header in bytes.
pub const CLASSIC_HEADER_SIZE: usize = 68;

/// The byte offset where data blocks begin in Classic format.
pub const CLASSIC_DATA_OFFSET: usize = 0x44;

/// The expected header size value in Classic headers.
pub const CLASSIC_EXPECTED_HEADER_SIZE: u32 = 68;

/// The expected header version value in Classic headers.
pub const CLASSIC_EXPECTED_HEADER_VERSION: u32 = 1;

/// The sub-header magic for The Frozen Throne expansion.
/// "PX3W" is "W3XP" reversed, indicating TFT/expansion content.
pub const TFT_SUB_HEADER_MAGIC: &[u8; 4] = b"PX3W";

/// Parsed header for Classic format replay files.
///
/// This struct contains all fields from the 68-byte Classic header.
/// The build version field is particularly important as it determines
/// the block format variant (Type A or Type B).
///
/// # Example
///
/// ```no_run
/// use w3g_parser::header::classic::ClassicHeader;
///
/// let data = std::fs::read("replay.w3g").unwrap();
/// let header = ClassicHeader::parse(&data).unwrap();
/// println!("Game duration: {} ms", header.duration_ms);
/// println!("Block format: {:?}", header.version_type());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassicHeader {
    /// Magic string at offset 0x00 (28 bytes).
    /// "Warcraft III recorded game\x1A\x00"
    pub magic: [u8; 28],

    /// Header size at offset 0x1C (always 68).
    pub header_size: u32,

    /// Total file size in bytes at offset 0x20.
    /// This should match the actual file size on disk.
    pub file_size: u32,

    /// Header version at offset 0x24 (always 1).
    pub header_version: u32,

    /// Total size of decompressed replay data at offset 0x28.
    pub decompressed_size: u32,

    /// Number of data blocks at offset 0x2C.
    /// Each block contains compressed replay data.
    pub block_count: u32,

    /// Sub-header magic at offset 0x30.
    /// "PX3W" indicates The Frozen Throne expansion.
    pub sub_header_magic: [u8; 4],

    /// Build version at offset 0x34.
    /// Determines block format:
    /// - 26: Type A (8-byte block headers)
    /// - 10032, 10036, etc.: Type B (12-byte block headers)
    pub build_version: u32,

    /// Flags/build info at offset 0x38.
    /// High bit is always set (0x80xxxxxx pattern).
    /// Exact meaning unknown.
    pub flags: u32,

    /// Game duration in milliseconds at offset 0x3C.
    pub duration_ms: u32,

    /// Checksum at offset 0x40.
    /// Algorithm unknown.
    pub checksum: [u8; 4],
}

impl ClassicHeader {
    /// Parses a Classic header from raw bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes of the replay file (at least 68 bytes required)
    ///
    /// # Returns
    ///
    /// A `ClassicHeader` struct containing all header fields.
    ///
    /// # Errors
    ///
    /// - `ParserError::UnexpectedEof` if data is shorter than 68 bytes
    /// - `ParserError::InvalidMagic` if the magic string doesn't match
    ///
    /// # Example
    ///
    /// ```no_run
    /// use w3g_parser::header::classic::ClassicHeader;
    ///
    /// let data = std::fs::read("replay.w3g").unwrap();
    /// let header = ClassicHeader::parse(&data)?;
    /// # Ok::<(), w3g_parser::error::ParserError>(())
    /// ```
    pub fn parse(data: &[u8]) -> Result<Self> {
        // Check minimum size
        if data.len() < CLASSIC_HEADER_SIZE {
            return Err(ParserError::unexpected_eof(
                CLASSIC_HEADER_SIZE,
                data.len(),
            ));
        }

        // Read and validate magic
        let magic_slice = read_bytes(data, 0x00, 28)?;
        if magic_slice != CLASSIC_MAGIC {
            return Err(ParserError::invalid_magic(CLASSIC_MAGIC, magic_slice));
        }

        let mut magic = [0u8; 28];
        magic.copy_from_slice(magic_slice);

        // Read all fields
        let header_size = read_u32_le(data, 0x1C)?;
        let file_size = read_u32_le(data, 0x20)?;
        let header_version = read_u32_le(data, 0x24)?;
        let decompressed_size = read_u32_le(data, 0x28)?;
        let block_count = read_u32_le(data, 0x2C)?;

        let sub_header_magic_slice = read_bytes(data, 0x30, 4)?;
        let mut sub_header_magic = [0u8; 4];
        sub_header_magic.copy_from_slice(sub_header_magic_slice);

        let build_version = read_u32_le(data, 0x34)?;
        let flags = read_u32_le(data, 0x38)?;
        let duration_ms = read_u32_le(data, 0x3C)?;

        let checksum_slice = read_bytes(data, 0x40, 4)?;
        let mut checksum = [0u8; 4];
        checksum.copy_from_slice(checksum_slice);

        Ok(ClassicHeader {
            magic,
            header_size,
            file_size,
            header_version,
            decompressed_size,
            block_count,
            sub_header_magic,
            build_version,
            flags,
            duration_ms,
            checksum,
        })
    }

    /// Determines the Classic version type based on build version.
    ///
    /// - **Type A**: Build version < 10000 (uses 8-byte block headers)
    /// - **Type B**: Build version >= 10000 (uses 12-byte block headers)
    ///
    /// # Returns
    ///
    /// The `ClassicVersion` variant for this header.
    #[must_use]
    pub fn version_type(&self) -> ClassicVersion {
        ClassicVersion::from_build_version(self.build_version)
    }

    /// Returns the byte offset where data blocks begin.
    ///
    /// For Classic format, this is always 0x44 (68 bytes).
    #[must_use]
    pub const fn data_offset(&self) -> usize {
        CLASSIC_DATA_OFFSET
    }

    /// Returns whether this is a Type A (8-byte block header) replay.
    #[must_use]
    pub fn is_type_a(&self) -> bool {
        self.build_version < CLASSIC_TYPE_B_THRESHOLD
    }

    /// Returns whether this is a Type B (12-byte block header) replay.
    #[must_use]
    pub fn is_type_b(&self) -> bool {
        self.build_version >= CLASSIC_TYPE_B_THRESHOLD
    }

    /// Returns the block header size in bytes based on the build version.
    #[must_use]
    pub fn block_header_size(&self) -> usize {
        self.version_type().block_header_size()
    }

    /// Converts the game duration from milliseconds to a human-readable format.
    ///
    /// # Returns
    ///
    /// A tuple of (hours, minutes, seconds, milliseconds).
    #[must_use]
    pub fn duration_parts(&self) -> (u32, u32, u32, u32) {
        let total_ms = self.duration_ms;
        let ms = total_ms % 1000;
        let total_seconds = total_ms / 1000;
        let seconds = total_seconds % 60;
        let total_minutes = total_seconds / 60;
        let minutes = total_minutes % 60;
        let hours = total_minutes / 60;

        (hours, minutes, seconds, ms)
    }

    /// Returns the game duration formatted as "HH:MM:SS".
    #[must_use]
    pub fn duration_string(&self) -> String {
        let (hours, minutes, seconds, _) = self.duration_parts();
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    }

    /// Returns the display version string for this replay.
    ///
    /// For Reforged builds (>= 10000):
    /// - Build 10000-10099 -> "2.00"
    /// - Build 10100-10199 -> "2.01"
    /// - etc.
    ///
    /// For Classic builds (< 10000):
    /// - Build 26 -> "1.26"
    /// - Build 6059 -> "1.29" (TFT)
    /// - etc.
    #[must_use]
    pub fn version_string(&self) -> String {
        if self.build_version >= 10000 {
            // Reforged: version 2.x
            let minor = (self.build_version - 10000) / 100;
            format!("2.{minor:02}")
        } else if self.build_version >= 6000 {
            // TFT 1.29+
            let minor = 29 + (self.build_version - 6059) / 100;
            format!("1.{minor}")
        } else {
            // Classic/TFT: version 1.x where x = build
            format!("1.{}", self.build_version)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a minimal valid Classic header for testing.
    fn create_test_header() -> Vec<u8> {
        let mut header = vec![0u8; CLASSIC_HEADER_SIZE];

        // Magic string
        header[0x00..0x1C].copy_from_slice(CLASSIC_MAGIC);

        // Header size = 68
        header[0x1C..0x20].copy_from_slice(&68u32.to_le_bytes());

        // File size = 100_646
        header[0x20..0x24].copy_from_slice(&100_646u32.to_le_bytes());

        // Header version = 1
        header[0x24..0x28].copy_from_slice(&1u32.to_le_bytes());

        // Decompressed size = 500_000
        header[0x28..0x2C].copy_from_slice(&500_000u32.to_le_bytes());

        // Block count = 34
        header[0x2C..0x30].copy_from_slice(&34u32.to_le_bytes());

        // Sub-header magic "PX3W"
        header[0x30..0x34].copy_from_slice(b"PX3W");

        // Build version = 26 (Type A)
        header[0x34..0x38].copy_from_slice(&26u32.to_le_bytes());

        // Flags = 0x80000000
        header[0x38..0x3C].copy_from_slice(&0x8000_0000u32.to_le_bytes());

        // Duration = 650_600 ms (about 10.8 minutes)
        header[0x3C..0x40].copy_from_slice(&650_600u32.to_le_bytes());

        // Checksum (arbitrary)
        header[0x40..0x44].copy_from_slice(&[0xAB, 0xCD, 0xEF, 0x12]);

        header
    }

    #[test]
    fn test_parse_valid_header() {
        let data = create_test_header();
        let header = ClassicHeader::parse(&data).unwrap();

        assert_eq!(&header.magic, CLASSIC_MAGIC);
        assert_eq!(header.header_size, 68);
        assert_eq!(header.file_size, 100_646);
        assert_eq!(header.header_version, 1);
        assert_eq!(header.decompressed_size, 500_000);
        assert_eq!(header.block_count, 34);
        assert_eq!(&header.sub_header_magic, b"PX3W");
        assert_eq!(header.build_version, 26);
        assert_eq!(header.flags, 0x8000_0000);
        assert_eq!(header.duration_ms, 650_600);
    }

    #[test]
    fn test_parse_too_short() {
        let data = vec![0u8; 32]; // Less than 68 bytes
        let result = ClassicHeader::parse(&data);

        assert!(matches!(
            result,
            Err(ParserError::UnexpectedEof {
                expected: 68,
                available: 32
            })
        ));
    }

    #[test]
    fn test_parse_invalid_magic() {
        let mut data = create_test_header();
        data[0..10].copy_from_slice(b"Invalid!!!");

        let result = ClassicHeader::parse(&data);
        assert!(matches!(result, Err(ParserError::InvalidMagic { .. })));
    }

    #[test]
    fn test_version_type_a() {
        let data = create_test_header(); // Uses build version 26
        let header = ClassicHeader::parse(&data).unwrap();

        assert_eq!(header.version_type(), ClassicVersion::TypeA);
        assert!(header.is_type_a());
        assert!(!header.is_type_b());
        assert_eq!(header.block_header_size(), 8);
    }

    #[test]
    fn test_version_type_b() {
        let mut data = create_test_header();
        // Change build version to 10036 (Type B)
        data[0x34..0x38].copy_from_slice(&10036u32.to_le_bytes());

        let header = ClassicHeader::parse(&data).unwrap();

        assert_eq!(header.version_type(), ClassicVersion::TypeB);
        assert!(!header.is_type_a());
        assert!(header.is_type_b());
        assert_eq!(header.block_header_size(), 12);
    }

    #[test]
    fn test_version_type_boundary() {
        // Test exactly at threshold (10000)
        let mut data = create_test_header();
        data[0x34..0x38].copy_from_slice(&10000u32.to_le_bytes());

        let header = ClassicHeader::parse(&data).unwrap();
        assert_eq!(header.version_type(), ClassicVersion::TypeB);

        // Test just below threshold (9999)
        data[0x34..0x38].copy_from_slice(&9999u32.to_le_bytes());

        let header = ClassicHeader::parse(&data).unwrap();
        assert_eq!(header.version_type(), ClassicVersion::TypeA);
    }

    #[test]
    fn test_data_offset() {
        let data = create_test_header();
        let header = ClassicHeader::parse(&data).unwrap();

        assert_eq!(header.data_offset(), 0x44);
        assert_eq!(header.data_offset(), 68);
    }

    #[test]
    fn test_duration_parts() {
        let data = create_test_header();
        let header = ClassicHeader::parse(&data).unwrap();

        // 650_600 ms = 10 minutes, 50 seconds, 600 ms
        let (hours, minutes, seconds, ms) = header.duration_parts();
        assert_eq!(hours, 0);
        assert_eq!(minutes, 10);
        assert_eq!(seconds, 50);
        assert_eq!(ms, 600);
    }

    #[test]
    fn test_duration_string() {
        let data = create_test_header();
        let header = ClassicHeader::parse(&data).unwrap();

        assert_eq!(header.duration_string(), "00:10:50");
    }

    #[test]
    fn test_duration_parts_long_game() {
        let mut data = create_test_header();
        // Set duration to 1 hour, 23 minutes, 45 seconds (5025000 ms)
        data[0x3C..0x40].copy_from_slice(&5_025_000u32.to_le_bytes());

        let header = ClassicHeader::parse(&data).unwrap();
        let (hours, minutes, seconds, _) = header.duration_parts();

        assert_eq!(hours, 1);
        assert_eq!(minutes, 23);
        assert_eq!(seconds, 45);
        assert_eq!(header.duration_string(), "01:23:45");
    }

    #[test]
    fn test_constants() {
        assert_eq!(CLASSIC_HEADER_SIZE, 68);
        assert_eq!(CLASSIC_DATA_OFFSET, 0x44);
        assert_eq!(CLASSIC_EXPECTED_HEADER_SIZE, 68);
        assert_eq!(CLASSIC_EXPECTED_HEADER_VERSION, 1);
        assert_eq!(TFT_SUB_HEADER_MAGIC, b"PX3W");
    }

    #[test]
    fn test_header_with_extra_data() {
        // Header parsing should work even with extra data appended
        let mut data = create_test_header();
        data.extend_from_slice(&[0x79, 0x0C, 0x00, 0x20]); // Fake block header

        let header = ClassicHeader::parse(&data).unwrap();
        assert_eq!(header.file_size, 100_646);
    }
}
