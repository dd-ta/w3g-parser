//! Header parsing for W3G replay files.
//!
//! This module provides parsers for both W3G replay format families:
//! - **GRBN (Reforged)**: 128-byte header, single zlib stream
//! - **Classic**: 68-byte header, block-based zlib compression
//!
//! # Usage
//!
//! The [`Header`] enum provides a unified interface for parsing either format.
//! Format detection is automatic based on magic bytes at the start of the file.
//!
//! ```no_run
//! use w3g_parser::header::Header;
//!
//! let data = std::fs::read("replay.w3g").unwrap();
//! let header = Header::parse(&data).unwrap();
//!
//! println!("Data offset: {}", header.data_offset());
//! println!("Decompressed size: {} bytes", header.decompressed_size());
//! ```
//!
//! # Format-Specific Access
//!
//! For format-specific fields, match on the `Header` enum or use the
//! individual header modules directly:
//!
//! ```no_run
//! use w3g_parser::header::{Header, ClassicHeader, GrbnHeader};
//!
//! let data = std::fs::read("replay.w3g").unwrap();
//! let header = Header::parse(&data).unwrap();
//!
//! match &header {
//!     Header::Grbn(h) => {
//!         println!("GRBN version: {}", h.version);
//!     }
//!     Header::Classic(h) => {
//!         println!("Build version: {}", h.build_version);
//!         println!("Game duration: {}", h.duration_string());
//!     }
//! }
//! ```

pub mod classic;
pub mod grbn;

pub use classic::ClassicHeader;
pub use grbn::GrbnHeader;

use crate::error::Result;
use crate::format::{detect_format, ClassicVersion, ReplayFormat};

/// A unified header type that can represent either GRBN or Classic format.
///
/// This enum provides a common interface for accessing header information
/// regardless of the underlying format. Use [`Header::parse`] to automatically
/// detect the format and parse the appropriate header.
///
/// # Example
///
/// ```no_run
/// use w3g_parser::header::Header;
///
/// let data = std::fs::read("replay.w3g").unwrap();
/// let header = Header::parse(&data)?;
///
/// // Common operations work on both formats
/// println!("Data starts at offset: {}", header.data_offset());
/// println!("Decompressed size: {} bytes", header.decompressed_size());
/// # Ok::<(), w3g_parser::error::ParserError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Header {
    /// A GRBN (Reforged) format header.
    Grbn(GrbnHeader),

    /// A Classic format header.
    Classic(ClassicHeader),
}

impl Header {
    /// Parses a header from raw bytes, automatically detecting the format.
    ///
    /// This function examines the magic bytes at the start of the file to
    /// determine which parser to use:
    /// - `GRBN` magic: Parses as GRBN (Reforged) format
    /// - `Warcraft III recorded game` magic: Parses as Classic format
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes of the replay file
    ///
    /// # Returns
    ///
    /// A `Header` enum variant containing the parsed header.
    ///
    /// # Errors
    ///
    /// - `ParserError::UnexpectedEof` if data is too short for format detection or header
    /// - `ParserError::InvalidMagic` if the magic bytes don't match any known format
    ///
    /// # Example
    ///
    /// ```no_run
    /// use w3g_parser::header::Header;
    ///
    /// let data = std::fs::read("replay.w3g").unwrap();
    /// let header = Header::parse(&data)?;
    /// # Ok::<(), w3g_parser::error::ParserError>(())
    /// ```
    pub fn parse(data: &[u8]) -> Result<Self> {
        let format = detect_format(data)?;

        match format {
            ReplayFormat::Grbn => Ok(Header::Grbn(GrbnHeader::parse(data)?)),
            ReplayFormat::Classic => Ok(Header::Classic(ClassicHeader::parse(data)?)),
        }
    }

    /// Returns the byte offset where compressed data begins.
    ///
    /// - GRBN: 0x80 (128 bytes)
    /// - Classic: 0x44 (68 bytes)
    #[must_use]
    pub fn data_offset(&self) -> usize {
        match self {
            Header::Grbn(h) => h.data_offset(),
            Header::Classic(h) => h.data_offset(),
        }
    }

    /// Returns the expected decompressed size in bytes.
    ///
    /// This value comes from the header and represents the total size
    /// of all replay data after decompression.
    #[must_use]
    pub fn decompressed_size(&self) -> u32 {
        match self {
            Header::Grbn(h) => h.decompressed_size,
            Header::Classic(h) => h.decompressed_size,
        }
    }

    /// Returns the format type of this header.
    #[must_use]
    pub fn format(&self) -> ReplayFormat {
        match self {
            Header::Grbn(_) => ReplayFormat::Grbn,
            Header::Classic(_) => ReplayFormat::Classic,
        }
    }

    /// Returns the header size in bytes.
    ///
    /// - GRBN: 128 bytes
    /// - Classic: 68 bytes
    #[must_use]
    pub fn header_size(&self) -> usize {
        match self {
            Header::Grbn(_) => grbn::GRBN_HEADER_SIZE,
            Header::Classic(_) => classic::CLASSIC_HEADER_SIZE,
        }
    }

    /// Returns whether this is a GRBN (Reforged) format header.
    #[must_use]
    pub fn is_grbn(&self) -> bool {
        matches!(self, Header::Grbn(_))
    }

    /// Returns whether this is a Classic format header.
    #[must_use]
    pub fn is_classic(&self) -> bool {
        matches!(self, Header::Classic(_))
    }

    /// Returns the Classic version type if this is a Classic header.
    ///
    /// Returns `None` for GRBN headers.
    #[must_use]
    pub fn classic_version(&self) -> Option<ClassicVersion> {
        match self {
            Header::Classic(h) => Some(h.version_type()),
            Header::Grbn(_) => None,
        }
    }

    /// Returns a reference to the inner GRBN header, if present.
    #[must_use]
    pub fn as_grbn(&self) -> Option<&GrbnHeader> {
        match self {
            Header::Grbn(h) => Some(h),
            Header::Classic(_) => None,
        }
    }

    /// Returns a reference to the inner Classic header, if present.
    #[must_use]
    pub fn as_classic(&self) -> Option<&ClassicHeader> {
        match self {
            Header::Classic(h) => Some(h),
            Header::Grbn(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a minimal valid GRBN header for testing.
    fn create_grbn_header() -> Vec<u8> {
        let mut header = vec![0u8; 128];
        header[0x00..0x04].copy_from_slice(b"GRBN");
        header[0x04..0x08].copy_from_slice(&2u32.to_le_bytes());
        header[0x08..0x0C].copy_from_slice(&11u32.to_le_bytes());
        header[0x0C..0x10].copy_from_slice(&51200u32.to_le_bytes());
        header[0x24..0x28].copy_from_slice(&1_000_000u32.to_le_bytes());
        header
    }

    /// Creates a minimal valid Classic header for testing.
    fn create_classic_header() -> Vec<u8> {
        let mut header = vec![0u8; 68];
        header[0x00..0x1C].copy_from_slice(b"Warcraft III recorded game\x1A\x00");
        header[0x1C..0x20].copy_from_slice(&68u32.to_le_bytes());
        header[0x20..0x24].copy_from_slice(&100_646u32.to_le_bytes());
        header[0x24..0x28].copy_from_slice(&1u32.to_le_bytes());
        header[0x28..0x2C].copy_from_slice(&500_000u32.to_le_bytes());
        header[0x2C..0x30].copy_from_slice(&34u32.to_le_bytes());
        header[0x30..0x34].copy_from_slice(b"PX3W");
        header[0x34..0x38].copy_from_slice(&26u32.to_le_bytes());
        header[0x38..0x3C].copy_from_slice(&0x8000_0000u32.to_le_bytes());
        header[0x3C..0x40].copy_from_slice(&650_600u32.to_le_bytes());
        header
    }

    #[test]
    fn test_parse_grbn() {
        let data = create_grbn_header();
        let header = Header::parse(&data).unwrap();

        assert!(header.is_grbn());
        assert!(!header.is_classic());
        assert_eq!(header.format(), ReplayFormat::Grbn);
        assert_eq!(header.data_offset(), 0x80);
        assert_eq!(header.decompressed_size(), 1_000_000);
        assert_eq!(header.header_size(), 128);
        assert!(header.as_grbn().is_some());
        assert!(header.as_classic().is_none());
        assert!(header.classic_version().is_none());
    }

    #[test]
    fn test_parse_classic() {
        let data = create_classic_header();
        let header = Header::parse(&data).unwrap();

        assert!(header.is_classic());
        assert!(!header.is_grbn());
        assert_eq!(header.format(), ReplayFormat::Classic);
        assert_eq!(header.data_offset(), 0x44);
        assert_eq!(header.decompressed_size(), 500_000);
        assert_eq!(header.header_size(), 68);
        assert!(header.as_classic().is_some());
        assert!(header.as_grbn().is_none());
        assert_eq!(header.classic_version(), Some(ClassicVersion::TypeA));
    }

    #[test]
    fn test_parse_classic_type_b() {
        let mut data = create_classic_header();
        // Change build version to 10036 (Type B)
        data[0x34..0x38].copy_from_slice(&10036u32.to_le_bytes());

        let header = Header::parse(&data).unwrap();

        assert!(header.is_classic());
        assert_eq!(header.classic_version(), Some(ClassicVersion::TypeB));

        if let Header::Classic(h) = &header {
            assert!(h.is_type_b());
            assert_eq!(h.block_header_size(), 12);
        }
    }

    #[test]
    fn test_grbn_inner_access() {
        let data = create_grbn_header();
        let header = Header::parse(&data).unwrap();

        let grbn = header.as_grbn().unwrap();
        assert_eq!(grbn.version, 2);
        assert_eq!(grbn.unknown_1, 11);
        assert_eq!(grbn.unknown_2, 51200);
    }

    #[test]
    fn test_classic_inner_access() {
        let data = create_classic_header();
        let header = Header::parse(&data).unwrap();

        let classic = header.as_classic().unwrap();
        assert_eq!(classic.build_version, 26);
        assert_eq!(classic.duration_ms, 650_600);
        assert_eq!(classic.block_count, 34);
    }

    #[test]
    fn test_header_enum_matching() {
        let grbn_data = create_grbn_header();
        let classic_data = create_classic_header();

        let grbn_header = Header::parse(&grbn_data).unwrap();
        let classic_header = Header::parse(&classic_data).unwrap();

        match grbn_header {
            Header::Grbn(h) => {
                assert_eq!(h.decompressed_size, 1_000_000);
            }
            Header::Classic(_) => panic!("Expected GRBN header"),
        }

        match classic_header {
            Header::Classic(h) => {
                assert_eq!(h.file_size, 100_646);
            }
            Header::Grbn(_) => panic!("Expected Classic header"),
        }
    }
}
