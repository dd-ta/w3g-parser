//! Decompression utilities for W3G replay files.
//!
//! This module provides decompression functionality for both W3G format families:
//! - **GRBN (Reforged)**: Metadata zlib + embedded Classic replay
//! - **Classic**: Block-based zlib compression with Type A/B variants
//!
//! # Usage
//!
//! Use the [`decompress`] function to automatically decompress either format:
//!
//! ```no_run
//! use w3g_parser::header::Header;
//! use w3g_parser::decompress::decompress;
//!
//! let data = std::fs::read("replay.w3g").unwrap();
//! let header = Header::parse(&data).unwrap();
//! let decompressed = decompress(&data, &header).unwrap();
//! println!("Decompressed {} bytes", decompressed.len());
//! ```
//!
//! # Format-Specific Functions
//!
//! For direct access to format-specific decompression:
//!
//! - [`grbn::decompress_grbn`] - GRBN decompression (metadata + embedded Classic)
//! - [`classic::decompress_classic`] - Classic block-based decompression
//!
//! # Compression Details
//!
//! ## GRBN Format
//!
//! GRBN (Reforged) files have a complex structure:
//! - Metadata zlib at offset 0x80 (contains player info, game settings)
//! - An embedded Classic format replay at a variable offset
//!
//! ## Classic Format
//!
//! - Multiple independently compressed blocks
//! - Each block typically decompresses to 8192 bytes (0x2000)
//! - Block header format varies by build version:
//!   - Type A (build < 10000): 8-byte headers
//!   - Type B (build >= 10000): 12-byte headers

pub mod classic;
pub mod grbn;

pub use classic::{
    decompress_classic, BlockHeader, BLOCK_DECOMPRESSED_SIZE, BLOCK_HEADER_SIZE_A,
    BLOCK_HEADER_SIZE_B,
};
pub use grbn::decompress_grbn;

use crate::error::Result;
use crate::header::Header;

/// Decompresses replay data from either GRBN or Classic format.
///
/// This function automatically detects the format from the header and
/// calls the appropriate decompression function.
///
/// # Arguments
///
/// * `data` - The raw bytes of the entire replay file
/// * `header` - The parsed header (GRBN or Classic)
///
/// # Returns
///
/// A `Vec<u8>` containing the decompressed replay data.
///
/// # Errors
///
/// - `ParserError::UnexpectedEof` if the data is truncated
/// - `ParserError::DecompressionError` if the zlib data is invalid
///
/// # Example
///
/// ```no_run
/// use w3g_parser::header::Header;
/// use w3g_parser::decompress::decompress;
///
/// let data = std::fs::read("replay.w3g").unwrap();
/// let header = Header::parse(&data)?;
/// let decompressed = decompress(&data, &header)?;
///
/// println!("Decompressed {} bytes", decompressed.len());
/// # Ok::<(), w3g_parser::error::ParserError>(())
/// ```
pub fn decompress(data: &[u8], header: &Header) -> Result<Vec<u8>> {
    match header {
        Header::Grbn(h) => decompress_grbn(data, h),
        Header::Classic(h) => decompress_classic(data, h),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::CLASSIC_MAGIC;

    /// Creates a minimal Classic file for testing.
    fn create_classic_file() -> Vec<u8> {
        let mut file = vec![0u8; 68];

        file[0x00..0x1C].copy_from_slice(CLASSIC_MAGIC);
        file[0x1C..0x20].copy_from_slice(&68u32.to_le_bytes());
        file[0x20..0x24].copy_from_slice(&1000u32.to_le_bytes());
        file[0x24..0x28].copy_from_slice(&1u32.to_le_bytes());
        file[0x28..0x2C].copy_from_slice(&4u32.to_le_bytes());
        file[0x2C..0x30].copy_from_slice(&1u32.to_le_bytes()); // 1 block
        file[0x30..0x34].copy_from_slice(b"PX3W");
        file[0x34..0x38].copy_from_slice(&26u32.to_le_bytes()); // Type A
        file[0x38..0x3C].copy_from_slice(&0x8000_0000u32.to_le_bytes());
        file[0x3C..0x40].copy_from_slice(&0u32.to_le_bytes());

        // "Test" compressed with zlib (12 bytes)
        let zlib_data: &[u8] = &[
            0x78, 0x9C, 0x0B, 0x49, 0x2D, 0x2E, 0x01, 0x00, 0x03, 0xDD, 0x01, 0xA1,
        ];

        // Type A block header
        let compressed_size = zlib_data.len() as u16;
        file.extend_from_slice(&compressed_size.to_le_bytes());
        file.extend_from_slice(&4u16.to_le_bytes());
        file.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        file.extend_from_slice(zlib_data);

        file
    }

    #[test]
    fn test_decompress_classic() {
        let data = create_classic_file();
        let header = Header::parse(&data).unwrap();
        let result = decompress(&data, &header).unwrap();

        assert_eq!(result, b"Test");
    }

    // Note: GRBN decompression tests require real replay files with embedded
    // Classic replays, so they are tested in integration tests rather than
    // unit tests.
}
