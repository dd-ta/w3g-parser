//! GRBN decompression for Warcraft III: Reforged replay files.
//!
//! The GRBN format (Reforged) has a more complex structure than originally
//! documented. It contains:
//!
//! 1. A 128-byte GRBN header
//! 2. A small zlib-compressed metadata blob at offset 0x80 (contains player info, etc.)
//! 3. Zero padding
//! 4. An embedded Classic format replay (identified by the Classic magic string)
//!
//! This module decompresses both the metadata and the embedded Classic replay,
//! returning the combined decompressed data.
//!
//! # Example
//!
//! ```no_run
//! use w3g_parser::header::grbn::GrbnHeader;
//! use w3g_parser::decompress::grbn::decompress_grbn;
//!
//! let data = std::fs::read("replay.w3g").unwrap();
//! let header = GrbnHeader::parse(&data).unwrap();
//! let decompressed = decompress_grbn(&data, &header).unwrap();
//! println!("Decompressed {} bytes", decompressed.len());
//! ```

use std::io::Read;

use flate2::read::ZlibDecoder;

use crate::binary::read_u16_le;
use crate::error::{ParserError, Result};
use crate::format::{ClassicVersion, CLASSIC_MAGIC, CLASSIC_TYPE_B_THRESHOLD};
use crate::header::grbn::{GrbnHeader, GRBN_DATA_OFFSET};

/// Size of the Classic header in bytes.
const CLASSIC_HEADER_SIZE: usize = 68;

/// Offset to the build version field within a Classic header.
const CLASSIC_BUILD_VERSION_OFFSET: usize = 0x34;

/// Offset to the block count field within a Classic header.
const CLASSIC_BLOCK_COUNT_OFFSET: usize = 0x2C;

/// Block header size for Type A format (build version < 10000).
const BLOCK_HEADER_SIZE_A: usize = 8;

/// Block header size for Type B format (build version >= 10000).
const BLOCK_HEADER_SIZE_B: usize = 12;

/// Decompresses the replay data from a GRBN format file.
///
/// GRBN files have a complex structure:
/// 1. Metadata zlib at offset 0x80 (small, contains player info)
/// 2. An embedded Classic format replay at a variable offset
///
/// This function decompresses both sections and returns the combined data:
/// - First: the metadata from the initial zlib stream
/// - Second: the decompressed data from the embedded Classic replay
///
/// # Arguments
///
/// * `data` - The raw bytes of the entire replay file
/// * `_header` - The parsed GRBN header (currently unused but kept for API consistency)
///
/// # Returns
///
/// A `Vec<u8>` containing the combined decompressed replay data.
///
/// # Errors
///
/// - `ParserError::UnexpectedEof` if the data is shorter than expected
/// - `ParserError::DecompressionError` if decompression fails
///
/// # Example
///
/// ```no_run
/// use w3g_parser::header::grbn::GrbnHeader;
/// use w3g_parser::decompress::grbn::decompress_grbn;
///
/// let data = std::fs::read("replay.w3g").unwrap();
/// let header = GrbnHeader::parse(&data).unwrap();
/// let decompressed = decompress_grbn(&data, &header)?;
/// # Ok::<(), w3g_parser::error::ParserError>(())
/// ```
pub fn decompress_grbn(data: &[u8], _header: &GrbnHeader) -> Result<Vec<u8>> {
    // Ensure we have data beyond the header
    if data.len() <= GRBN_DATA_OFFSET {
        return Err(ParserError::unexpected_eof(
            GRBN_DATA_OFFSET + 1,
            data.len(),
        ));
    }

    let mut result = Vec::new();

    // Step 1: Decompress the metadata zlib at offset 0x80
    let metadata = decompress_metadata_zlib(data)?;
    result.extend(metadata);

    // Step 2: Find and decompress the embedded Classic replay
    let embedded_data = decompress_embedded_classic(data)?;
    result.extend(embedded_data);

    Ok(result)
}

/// Decompresses the metadata zlib stream at offset 0x80.
///
/// This small stream contains game metadata like player names, game settings, etc.
fn decompress_metadata_zlib(data: &[u8]) -> Result<Vec<u8>> {
    let compressed = &data[GRBN_DATA_OFFSET..];

    let mut decoder = ZlibDecoder::new(compressed);
    let mut decompressed = Vec::new();

    decoder.read_to_end(&mut decompressed).map_err(|e| {
        ParserError::DecompressionError {
            reason: format!("GRBN metadata zlib decompression failed: {e}"),
        }
    })?;

    Ok(decompressed)
}

/// Finds and decompresses the embedded Classic replay.
///
/// Searches for the Classic magic string in the data and decompresses
/// the block-based replay data found there.
fn decompress_embedded_classic(data: &[u8]) -> Result<Vec<u8>> {
    // Find the Classic header by searching for its magic string
    let classic_offset = find_classic_header(data).ok_or_else(|| ParserError::DecompressionError {
        reason: "No embedded Classic replay found in GRBN file".to_string(),
    })?;

    // Ensure we have enough data for the Classic header
    if classic_offset + CLASSIC_HEADER_SIZE > data.len() {
        return Err(ParserError::unexpected_eof(
            classic_offset + CLASSIC_HEADER_SIZE,
            data.len(),
        ));
    }

    // Read Classic header fields
    let build_version = u32::from_le_bytes([
        data[classic_offset + CLASSIC_BUILD_VERSION_OFFSET],
        data[classic_offset + CLASSIC_BUILD_VERSION_OFFSET + 1],
        data[classic_offset + CLASSIC_BUILD_VERSION_OFFSET + 2],
        data[classic_offset + CLASSIC_BUILD_VERSION_OFFSET + 3],
    ]);

    let block_count = u32::from_le_bytes([
        data[classic_offset + CLASSIC_BLOCK_COUNT_OFFSET],
        data[classic_offset + CLASSIC_BLOCK_COUNT_OFFSET + 1],
        data[classic_offset + CLASSIC_BLOCK_COUNT_OFFSET + 2],
        data[classic_offset + CLASSIC_BLOCK_COUNT_OFFSET + 3],
    ]);

    // Determine block header size based on version
    let version = if build_version < CLASSIC_TYPE_B_THRESHOLD {
        ClassicVersion::TypeA
    } else {
        ClassicVersion::TypeB
    };

    let block_header_size = match version {
        ClassicVersion::TypeA => BLOCK_HEADER_SIZE_A,
        ClassicVersion::TypeB => BLOCK_HEADER_SIZE_B,
    };

    // Decompress all blocks
    let mut result = Vec::new();
    let mut offset = classic_offset + CLASSIC_HEADER_SIZE; // 0x44 bytes after Classic header

    for block_index in 0..block_count {
        // Read compressed size from block header
        let compressed_size = read_u16_le(data, offset).map_err(|_| {
            ParserError::DecompressionError {
                reason: format!(
                    "Failed to read block header {block_index} at offset 0x{offset:X}"
                ),
            }
        })? as usize;

        let compressed_start = offset + block_header_size;
        let compressed_end = compressed_start + compressed_size;

        // Ensure we have enough data
        if compressed_end > data.len() {
            return Err(ParserError::DecompressionError {
                reason: format!(
                    "Embedded Classic block {block_index} extends beyond file (offset 0x{offset:X}, needs {compressed_end} bytes)"
                ),
            });
        }

        // Decompress this block
        let compressed_data = &data[compressed_start..compressed_end];
        let mut decoder = ZlibDecoder::new(compressed_data);

        decoder.read_to_end(&mut result).map_err(|e| {
            ParserError::DecompressionError {
                reason: format!(
                    "Embedded Classic block {block_index} decompression failed at offset 0x{offset:X}: {e}"
                ),
            }
        })?;

        offset = compressed_end;
    }

    Ok(result)
}

/// Searches for the Classic header magic string in the data.
///
/// Returns the offset of the Classic header if found, or None otherwise.
fn find_classic_header(data: &[u8]) -> Option<usize> {
    // Start searching after the GRBN header and metadata
    let search_start = GRBN_DATA_OFFSET + 100; // Skip past metadata area

    (search_start..data.len().saturating_sub(CLASSIC_MAGIC.len())).find(|&i| &data[i..i + CLASSIC_MAGIC.len()] == CLASSIC_MAGIC)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_classic_header_not_present() {
        // Create data without Classic header
        let mut data = vec![0u8; 1000];
        data[0x00..0x04].copy_from_slice(b"GRBN");

        let result = find_classic_header(&data);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_classic_header_present() {
        let mut data = vec![0u8; 1000];
        data[0x00..0x04].copy_from_slice(b"GRBN");

        // Insert Classic header at offset 500
        data[500..528].copy_from_slice(CLASSIC_MAGIC);

        let result = find_classic_header(&data);
        assert_eq!(result, Some(500));
    }

    #[test]
    fn test_decompress_grbn_too_short() {
        let data = vec![0u8; 64];
        let header = GrbnHeader {
            magic: *b"GRBN",
            version: 2,
            unknown_1: 11,
            unknown_2: 51200,
            unknown_3: 0,
            unknown_4: 0,
            decompressed_size: 1000,
        };

        let result = decompress_grbn(&data, &header);
        assert!(matches!(result, Err(ParserError::UnexpectedEof { .. })));
    }

    #[test]
    fn test_constants() {
        assert_eq!(GRBN_DATA_OFFSET, 0x80);
        assert_eq!(GRBN_DATA_OFFSET, 128);
        assert_eq!(CLASSIC_HEADER_SIZE, 68);
        assert_eq!(BLOCK_HEADER_SIZE_A, 8);
        assert_eq!(BLOCK_HEADER_SIZE_B, 12);
    }
}
