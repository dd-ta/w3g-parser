//! Classic block-based decompression for Warcraft III replay files.
//!
//! The Classic format uses block-based zlib compression. Each block is
//! independently compressed and must be decompressed separately, then
//! concatenated to form the complete replay data.
//!
//! # Block Format Variants
//!
//! Two block header formats exist based on the build version:
//!
//! ## Type A (Build version < 10000)
//!
//! 8-byte block header:
//! - 2 bytes: Compressed data size
//! - 2 bytes: Decompressed size (always 0x2000 = 8192)
//! - 4 bytes: Checksum
//!
//! ## Type B (Build version >= 10000)
//!
//! 12-byte block header:
//! - 2 bytes: Compressed data size
//! - 2 bytes: Padding (zeros)
//! - 2 bytes: Decompressed size (always 0x2000 = 8192)
//! - 2 bytes: Padding (zeros)
//! - 4 bytes: Checksum
//!
//! # Example
//!
//! ```no_run
//! use w3g_parser::header::classic::ClassicHeader;
//! use w3g_parser::decompress::classic::decompress_classic;
//!
//! let data = std::fs::read("replay.w3g").unwrap();
//! let header = ClassicHeader::parse(&data).unwrap();
//! let decompressed = decompress_classic(&data, &header).unwrap();
//! println!("Decompressed {} bytes from {} blocks",
//!          decompressed.len(), header.block_count);
//! ```

use std::io::Read;

use flate2::read::ZlibDecoder;

use crate::binary::read_u16_le;
use crate::error::{ParserError, Result};
use crate::format::ClassicVersion;
use crate::header::classic::{ClassicHeader, CLASSIC_DATA_OFFSET};

/// Block header size for Type A format (build version < 10000).
pub const BLOCK_HEADER_SIZE_A: usize = 8;

/// Block header size for Type B format (build version >= 10000).
pub const BLOCK_HEADER_SIZE_B: usize = 12;

/// Standard decompressed block size (8192 bytes = 0x2000).
pub const BLOCK_DECOMPRESSED_SIZE: usize = 8192;

/// Information parsed from a block header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockHeader {
    /// The size of the compressed data following this header.
    pub compressed_size: u16,

    /// The expected size after decompression (typically 8192).
    pub decompressed_size: u16,

    /// The total size of this block header in bytes.
    pub header_size: usize,
}

impl BlockHeader {
    /// Parses a Type A block header (8 bytes).
    ///
    /// Layout:
    /// - Offset 0: Compressed size (u16)
    /// - Offset 2: Decompressed size (u16)
    /// - Offset 4: Checksum (4 bytes, ignored)
    fn parse_type_a(data: &[u8], offset: usize) -> Result<Self> {
        if offset + BLOCK_HEADER_SIZE_A > data.len() {
            return Err(ParserError::unexpected_eof(
                offset + BLOCK_HEADER_SIZE_A,
                data.len(),
            ));
        }

        let compressed_size = read_u16_le(data, offset)?;
        let decompressed_size = read_u16_le(data, offset + 2)?;

        Ok(BlockHeader {
            compressed_size,
            decompressed_size,
            header_size: BLOCK_HEADER_SIZE_A,
        })
    }

    /// Parses a Type B block header (12 bytes).
    ///
    /// Layout:
    /// - Offset 0: Compressed size (u16)
    /// - Offset 2: Padding (2 bytes, zeros)
    /// - Offset 4: Decompressed size (u16)
    /// - Offset 6: Padding (2 bytes, zeros)
    /// - Offset 8: Checksum (4 bytes, ignored)
    fn parse_type_b(data: &[u8], offset: usize) -> Result<Self> {
        if offset + BLOCK_HEADER_SIZE_B > data.len() {
            return Err(ParserError::unexpected_eof(
                offset + BLOCK_HEADER_SIZE_B,
                data.len(),
            ));
        }

        let compressed_size = read_u16_le(data, offset)?;
        let decompressed_size = read_u16_le(data, offset + 4)?;

        Ok(BlockHeader {
            compressed_size,
            decompressed_size,
            header_size: BLOCK_HEADER_SIZE_B,
        })
    }

    /// Parses a block header based on the Classic version type.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes of the replay file
    /// * `offset` - The byte offset where the block header starts
    /// * `version` - The Classic version type (determines header format)
    ///
    /// # Returns
    ///
    /// A `BlockHeader` containing the parsed information.
    ///
    /// # Errors
    ///
    /// - `ParserError::UnexpectedEof` if data is too short
    pub fn parse(data: &[u8], offset: usize, version: ClassicVersion) -> Result<Self> {
        match version {
            ClassicVersion::TypeA => Self::parse_type_a(data, offset),
            ClassicVersion::TypeB => Self::parse_type_b(data, offset),
        }
    }
}

/// Decompresses the replay data from a Classic format file.
///
/// Classic files contain multiple zlib-compressed blocks starting at
/// offset 0x44 (68 bytes). Each block is independently compressed and
/// must be decompressed separately.
///
/// # Arguments
///
/// * `data` - The raw bytes of the entire replay file
/// * `header` - The parsed Classic header
///
/// # Returns
///
/// A `Vec<u8>` containing the concatenated decompressed data from all blocks.
///
/// # Errors
///
/// - `ParserError::UnexpectedEof` if a block extends beyond the file
/// - `ParserError::DecompressionError` if a block's zlib data is invalid
///
/// # Example
///
/// ```no_run
/// use w3g_parser::header::classic::ClassicHeader;
/// use w3g_parser::decompress::classic::decompress_classic;
///
/// let data = std::fs::read("replay.w3g").unwrap();
/// let header = ClassicHeader::parse(&data)?;
/// let decompressed = decompress_classic(&data, &header)?;
/// # Ok::<(), w3g_parser::error::ParserError>(())
/// ```
pub fn decompress_classic(data: &[u8], header: &ClassicHeader) -> Result<Vec<u8>> {
    let version = header.version_type();
    let block_count = header.block_count;

    // Pre-allocate based on expected total decompressed size
    let capacity = header.decompressed_size as usize;
    let mut result = Vec::with_capacity(capacity);

    // Start at the data offset (after 68-byte header)
    let mut offset = CLASSIC_DATA_OFFSET;

    for block_index in 0..block_count {
        // Parse block header
        let block_header = BlockHeader::parse(data, offset, version).map_err(|e| {
            ParserError::DecompressionError {
                reason: format!(
                    "Failed to parse block header {block_index} at offset 0x{offset:X}: {e}"
                ),
            }
        })?;

        // Move past the block header to the compressed data
        let compressed_start = offset + block_header.header_size;
        let compressed_end = compressed_start + block_header.compressed_size as usize;

        // Ensure we have enough data
        if compressed_end > data.len() {
            return Err(ParserError::DecompressionError {
                reason: format!(
                    "Block {} at offset 0x{:X} extends beyond file (needs {} bytes, file has {})",
                    block_index, offset, compressed_end, data.len()
                ),
            });
        }

        // Get the compressed data slice
        let compressed_data = &data[compressed_start..compressed_end];

        // Decompress this block
        let mut decoder = ZlibDecoder::new(compressed_data);
        decoder.read_to_end(&mut result).map_err(|e| {
            ParserError::DecompressionError {
                reason: format!(
                    "Block {block_index} decompression failed at offset 0x{offset:X}: {e}"
                ),
            }
        })?;

        // Move to the next block
        offset = compressed_end;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::CLASSIC_MAGIC;

    /// Creates a minimal Classic header for testing.
    fn create_test_classic_header(build_version: u32, block_count: u32) -> Vec<u8> {
        let mut header = vec![0u8; 68];

        header[0x00..0x1C].copy_from_slice(CLASSIC_MAGIC);
        header[0x1C..0x20].copy_from_slice(&68u32.to_le_bytes());
        header[0x20..0x24].copy_from_slice(&1000u32.to_le_bytes()); // File size placeholder
        header[0x24..0x28].copy_from_slice(&1u32.to_le_bytes());
        header[0x28..0x2C].copy_from_slice(&100u32.to_le_bytes()); // Decompressed size
        header[0x2C..0x30].copy_from_slice(&block_count.to_le_bytes());
        header[0x30..0x34].copy_from_slice(b"PX3W");
        header[0x34..0x38].copy_from_slice(&build_version.to_le_bytes());
        header[0x38..0x3C].copy_from_slice(&0x8000_0000u32.to_le_bytes());
        header[0x3C..0x40].copy_from_slice(&0u32.to_le_bytes()); // Duration

        header
    }

    /// Creates valid zlib compressed data for "Test" string.
    fn create_zlib_test_data() -> Vec<u8> {
        // "Test" compressed with zlib (12 bytes)
        vec![0x78, 0x9C, 0x0B, 0x49, 0x2D, 0x2E, 0x01, 0x00, 0x03, 0xDD, 0x01, 0xA1]
    }

    #[test]
    fn test_block_header_parse_type_a() {
        let data = [
            0x0C, 0x00, // Compressed size = 12
            0x00, 0x20, // Decompressed size = 8192
            0xAB, 0xCD, 0xEF, 0x12, // Checksum
        ];

        let header = BlockHeader::parse_type_a(&data, 0).unwrap();
        assert_eq!(header.compressed_size, 12);
        assert_eq!(header.decompressed_size, 8192);
        assert_eq!(header.header_size, 8);
    }

    #[test]
    fn test_block_header_parse_type_b() {
        let data = [
            0x0C, 0x00, // Compressed size = 12
            0x00, 0x00, // Padding
            0x00, 0x20, // Decompressed size = 8192
            0x00, 0x00, // Padding
            0xAB, 0xCD, 0xEF, 0x12, // Checksum
        ];

        let header = BlockHeader::parse_type_b(&data, 0).unwrap();
        assert_eq!(header.compressed_size, 12);
        assert_eq!(header.decompressed_size, 8192);
        assert_eq!(header.header_size, 12);
    }

    #[test]
    fn test_block_header_parse_type_a_truncated() {
        let data = [0x0C, 0x00, 0x00]; // Only 3 bytes, need 8

        let result = BlockHeader::parse_type_a(&data, 0);
        assert!(matches!(result, Err(ParserError::UnexpectedEof { .. })));
    }

    #[test]
    fn test_block_header_parse_type_b_truncated() {
        let data = [0x0C, 0x00, 0x00, 0x00, 0x00, 0x20]; // Only 6 bytes, need 12

        let result = BlockHeader::parse_type_b(&data, 0);
        assert!(matches!(result, Err(ParserError::UnexpectedEof { .. })));
    }

    #[test]
    fn test_decompress_classic_type_a_single_block() {
        let mut file = create_test_classic_header(26, 1);

        // Add Type A block header
        let zlib_data = create_zlib_test_data();
        let compressed_size = zlib_data.len() as u16;

        file.extend_from_slice(&compressed_size.to_le_bytes()); // Compressed size
        file.extend_from_slice(&4u16.to_le_bytes()); // Decompressed size ("Test" = 4 bytes)
        file.extend_from_slice(&[0xAB, 0xCD, 0xEF, 0x12]); // Checksum

        // Add compressed data
        file.extend_from_slice(&zlib_data);

        let header = ClassicHeader::parse(&file).unwrap();
        let result = decompress_classic(&file, &header).unwrap();

        assert_eq!(result, b"Test");
    }

    #[test]
    fn test_decompress_classic_type_b_single_block() {
        let mut file = create_test_classic_header(10036, 1);

        // Add Type B block header
        let zlib_data = create_zlib_test_data();
        let compressed_size = zlib_data.len() as u16;

        file.extend_from_slice(&compressed_size.to_le_bytes()); // Compressed size
        file.extend_from_slice(&[0x00, 0x00]); // Padding
        file.extend_from_slice(&4u16.to_le_bytes()); // Decompressed size
        file.extend_from_slice(&[0x00, 0x00]); // Padding
        file.extend_from_slice(&[0xAB, 0xCD, 0xEF, 0x12]); // Checksum

        // Add compressed data
        file.extend_from_slice(&zlib_data);

        let header = ClassicHeader::parse(&file).unwrap();
        let result = decompress_classic(&file, &header).unwrap();

        assert_eq!(result, b"Test");
    }

    #[test]
    fn test_decompress_classic_multiple_blocks() {
        let mut file = create_test_classic_header(26, 2);

        // Add two Type A blocks, each containing "Test"
        let zlib_data = create_zlib_test_data();
        let compressed_size = zlib_data.len() as u16;

        // Block 1
        file.extend_from_slice(&compressed_size.to_le_bytes());
        file.extend_from_slice(&4u16.to_le_bytes());
        file.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        file.extend_from_slice(&zlib_data);

        // Block 2
        file.extend_from_slice(&compressed_size.to_le_bytes());
        file.extend_from_slice(&4u16.to_le_bytes());
        file.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        file.extend_from_slice(&zlib_data);

        let header = ClassicHeader::parse(&file).unwrap();
        let result = decompress_classic(&file, &header).unwrap();

        assert_eq!(result, b"TestTest");
    }

    #[test]
    fn test_decompress_classic_block_extends_beyond_file() {
        let mut file = create_test_classic_header(26, 1);

        // Add a block header claiming a large compressed size
        file.extend_from_slice(&1000u16.to_le_bytes()); // Claims 1000 bytes
        file.extend_from_slice(&4u16.to_le_bytes());
        file.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        // But only add 10 bytes of data
        file.extend_from_slice(&[0x78, 0x9C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        let header = ClassicHeader::parse(&file).unwrap();
        let result = decompress_classic(&file, &header);

        assert!(matches!(result, Err(ParserError::DecompressionError { .. })));
    }

    #[test]
    fn test_decompress_classic_invalid_zlib() {
        let mut file = create_test_classic_header(26, 1);

        // Add block header with invalid zlib data
        file.extend_from_slice(&4u16.to_le_bytes()); // Compressed size
        file.extend_from_slice(&4u16.to_le_bytes()); // Decompressed size
        file.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Checksum

        // Invalid zlib data
        file.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);

        let header = ClassicHeader::parse(&file).unwrap();
        let result = decompress_classic(&file, &header);

        assert!(matches!(result, Err(ParserError::DecompressionError { .. })));
    }

    #[test]
    fn test_constants() {
        assert_eq!(BLOCK_HEADER_SIZE_A, 8);
        assert_eq!(BLOCK_HEADER_SIZE_B, 12);
        assert_eq!(BLOCK_DECOMPRESSED_SIZE, 8192);
        assert_eq!(BLOCK_DECOMPRESSED_SIZE, 0x2000);
    }
}
