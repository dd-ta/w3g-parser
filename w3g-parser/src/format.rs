//! Format detection and routing for W3G replay files.
//!
//! This module handles automatic detection of replay format type based on
//! magic bytes at the start of the file, and provides enums to represent
//! the different format variants.
//!
//! # Format Detection
//!
//! W3G replays come in two format families:
//!
//! - **GRBN (Reforged)**: Identified by magic bytes `GRBN` (0x4752424E)
//! - **Classic**: Identified by magic string `Warcraft III recorded game\x1A\x00` (28 bytes)
//!
//! The Classic format has two sub-variants based on the build version field:
//! - **Type A** (build < 10000): Uses 8-byte block headers
//! - **Type B** (build >= 10000): Uses 12-byte block headers
//!
//! # Example
//!
//! ```
//! use w3g_parser::format::{detect_format, ReplayFormat};
//!
//! // GRBN format detection
//! let grbn_data = b"GRBN\x02\x00\x00\x00";
//! assert!(matches!(detect_format(grbn_data), Ok(ReplayFormat::Grbn)));
//!
//! // Classic format detection (partial magic shown)
//! let classic_magic = b"Warcraft III recorded game\x1A\x00";
//! assert!(matches!(detect_format(classic_magic), Ok(ReplayFormat::Classic)));
//! ```

use crate::error::{ParserError, Result};

/// The magic bytes for GRBN (Reforged) format.
pub const GRBN_MAGIC: &[u8; 4] = b"GRBN";

/// The magic string for Classic format (26 characters + 0x1A + 0x00).
pub const CLASSIC_MAGIC: &[u8; 28] = b"Warcraft III recorded game\x1A\x00";

/// The threshold build version that distinguishes Type A from Type B Classic format.
/// Builds below this value use 8-byte block headers (Type A).
/// Builds at or above this value use 12-byte block headers (Type B).
pub const CLASSIC_TYPE_B_THRESHOLD: u32 = 10000;

/// Represents the top-level format family of a W3G replay file.
///
/// Format detection is based on magic bytes at the start of the file:
/// - `Grbn`: Magic bytes `GRBN` at offset 0x00
/// - `Classic`: Magic string `Warcraft III recorded game\x1A\x00` at offset 0x00
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayFormat {
    /// GRBN format used by Warcraft III: Reforged (patch 1.32+).
    ///
    /// Characteristics:
    /// - 128-byte header
    /// - Single continuous zlib stream
    /// - Data starts at offset 0x80
    Grbn,

    /// Classic format used by original Warcraft III (RoC/TFT pre-1.32).
    ///
    /// Characteristics:
    /// - 68-byte header
    /// - Block-based zlib compression
    /// - Data starts at offset 0x44
    /// - Sub-variant (Type A or B) determined by build version
    Classic,
}

impl ReplayFormat {
    /// Returns the size of the main header for this format.
    #[must_use]
    pub const fn header_size(&self) -> usize {
        match self {
            ReplayFormat::Grbn => 128,
            ReplayFormat::Classic => 68,
        }
    }

    /// Returns the byte offset where compressed data begins.
    #[must_use]
    pub const fn data_offset(&self) -> usize {
        match self {
            ReplayFormat::Grbn => 0x80,    // 128
            ReplayFormat::Classic => 0x44, // 68
        }
    }
}

/// Represents the sub-variant of the Classic format.
///
/// The Classic format has two block header sizes depending on the build version:
/// - **Type A**: Build version < 10000 (e.g., version 26)
/// - **Type B**: Build version >= 10000 (e.g., versions 10032, 10036)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassicVersion {
    /// Type A format with 8-byte block headers.
    ///
    /// Used in older replays (build version 26 and similar).
    /// Block header layout:
    /// - 2 bytes: Compressed data size
    /// - 2 bytes: Decompressed size (always 0x2000)
    /// - 4 bytes: Checksum
    TypeA,

    /// Type B format with 12-byte block headers.
    ///
    /// Used in newer replays (build version 10000+).
    /// Block header layout:
    /// - 2 bytes: Compressed data size
    /// - 2 bytes: Padding (zeros)
    /// - 2 bytes: Decompressed size (always 0x2000)
    /// - 2 bytes: Padding (zeros)
    /// - 4 bytes: Checksum
    TypeB,
}

impl ClassicVersion {
    /// Returns the size of a block header in bytes for this version.
    #[must_use]
    pub const fn block_header_size(&self) -> usize {
        match self {
            ClassicVersion::TypeA => 8,
            ClassicVersion::TypeB => 12,
        }
    }

    /// Determines the Classic version type from a build version number.
    ///
    /// # Arguments
    ///
    /// * `build_version` - The build version from the Classic header (offset 0x34)
    ///
    /// # Returns
    ///
    /// - `TypeA` if `build_version` < 10000
    /// - `TypeB` if `build_version` >= 10000
    #[must_use]
    pub const fn from_build_version(build_version: u32) -> Self {
        if build_version < CLASSIC_TYPE_B_THRESHOLD {
            ClassicVersion::TypeA
        } else {
            ClassicVersion::TypeB
        }
    }
}

/// Detects the format of a W3G replay file from its raw bytes.
///
/// This function examines the magic bytes at the start of the file to determine
/// which format parser should be used.
///
/// # Arguments
///
/// * `data` - The raw bytes of the replay file (at least 28 bytes required)
///
/// # Returns
///
/// - `Ok(ReplayFormat::Grbn)` if the file starts with `GRBN`
/// - `Ok(ReplayFormat::Classic)` if the file starts with `Warcraft III recorded game\x1A\x00`
///
/// # Errors
///
/// - `ParserError::InvalidMagic` if neither magic sequence is found
/// - `ParserError::UnexpectedEof` if the file is too short to contain magic bytes
///
/// # Example
///
/// ```
/// use w3g_parser::format::{detect_format, ReplayFormat};
///
/// let grbn_data = b"GRBN\x02\x00\x00\x00";
/// assert!(matches!(detect_format(grbn_data), Ok(ReplayFormat::Grbn)));
/// ```
pub fn detect_format(data: &[u8]) -> Result<ReplayFormat> {
    // Need at least 4 bytes to check for GRBN magic
    if data.len() < 4 {
        return Err(ParserError::unexpected_eof(4, data.len()));
    }

    // Check for GRBN magic (4 bytes)
    if &data[..4] == GRBN_MAGIC {
        return Ok(ReplayFormat::Grbn);
    }

    // Need at least 28 bytes to check for Classic magic
    if data.len() < 28 {
        return Err(ParserError::unexpected_eof(28, data.len()));
    }

    // Check for Classic magic (28 bytes)
    if &data[..28] == CLASSIC_MAGIC {
        return Ok(ReplayFormat::Classic);
    }

    // Neither magic matched
    let found = if data.len() >= 28 {
        &data[..28]
    } else {
        data
    };

    Err(ParserError::invalid_magic(CLASSIC_MAGIC, found))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================
    // ReplayFormat tests
    // ========================

    #[test]
    fn test_replay_format_header_size() {
        assert_eq!(ReplayFormat::Grbn.header_size(), 128);
        assert_eq!(ReplayFormat::Classic.header_size(), 68);
    }

    #[test]
    fn test_replay_format_data_offset() {
        assert_eq!(ReplayFormat::Grbn.data_offset(), 0x80);
        assert_eq!(ReplayFormat::Classic.data_offset(), 0x44);
    }

    // ========================
    // ClassicVersion tests
    // ========================

    #[test]
    fn test_classic_version_block_header_size() {
        assert_eq!(ClassicVersion::TypeA.block_header_size(), 8);
        assert_eq!(ClassicVersion::TypeB.block_header_size(), 12);
    }

    #[test]
    fn test_classic_version_from_build_version() {
        // Type A: build < 10000
        assert_eq!(ClassicVersion::from_build_version(26), ClassicVersion::TypeA);
        assert_eq!(ClassicVersion::from_build_version(0), ClassicVersion::TypeA);
        assert_eq!(ClassicVersion::from_build_version(9999), ClassicVersion::TypeA);

        // Type B: build >= 10000
        assert_eq!(ClassicVersion::from_build_version(10000), ClassicVersion::TypeB);
        assert_eq!(ClassicVersion::from_build_version(10032), ClassicVersion::TypeB);
        assert_eq!(ClassicVersion::from_build_version(10036), ClassicVersion::TypeB);
        assert_eq!(ClassicVersion::from_build_version(99999), ClassicVersion::TypeB);
    }

    // ========================
    // detect_format tests
    // ========================

    #[test]
    fn test_detect_format_grbn() {
        let data = b"GRBN\x02\x00\x00\x00\x0B\x00\x00\x00";
        assert!(matches!(detect_format(data), Ok(ReplayFormat::Grbn)));
    }

    #[test]
    fn test_detect_format_classic() {
        let data = b"Warcraft III recorded game\x1A\x00\x44\x00\x00\x00";
        assert!(matches!(detect_format(data), Ok(ReplayFormat::Classic)));
    }

    #[test]
    fn test_detect_format_too_short_for_grbn() {
        let data = b"GRB";
        let result = detect_format(data);
        assert!(matches!(
            result,
            Err(ParserError::UnexpectedEof {
                expected: 4,
                available: 3
            })
        ));
    }

    #[test]
    fn test_detect_format_too_short_for_classic() {
        // Not GRBN, but also not enough bytes for Classic check
        let data = b"Warcraft III recorded ga";
        let result = detect_format(data);
        assert!(matches!(
            result,
            Err(ParserError::UnexpectedEof {
                expected: 28,
                available: 24
            })
        ));
    }

    #[test]
    fn test_detect_format_invalid_magic() {
        let data = b"Invalid magic bytes here!!!!\x00\x00\x00\x00";
        let result = detect_format(data);
        assert!(matches!(result, Err(ParserError::InvalidMagic { .. })));
    }

    #[test]
    fn test_detect_format_empty() {
        let data: &[u8] = &[];
        let result = detect_format(data);
        assert!(matches!(result, Err(ParserError::UnexpectedEof { .. })));
    }

    #[test]
    fn test_grbn_magic_constant() {
        assert_eq!(GRBN_MAGIC, b"GRBN");
        assert_eq!(GRBN_MAGIC[0], 0x47);
        assert_eq!(GRBN_MAGIC[1], 0x52);
        assert_eq!(GRBN_MAGIC[2], 0x42);
        assert_eq!(GRBN_MAGIC[3], 0x4E);
    }

    #[test]
    fn test_classic_magic_constant() {
        assert_eq!(CLASSIC_MAGIC.len(), 28);
        assert_eq!(&CLASSIC_MAGIC[..26], b"Warcraft III recorded game");
        assert_eq!(CLASSIC_MAGIC[26], 0x1A);
        assert_eq!(CLASSIC_MAGIC[27], 0x00);
    }
}
