//! Binary reading utilities for parsing W3G replay files.
//!
//! This module provides functions for reading little-endian integers,
//! byte slices, and null-terminated strings from byte buffers. All
//! functions perform bounds checking and return appropriate errors
//! for truncated or malformed data.
//!
//! # Endianness
//!
//! All W3G replay formats use little-endian byte order for multi-byte
//! integers. The functions in this module handle the conversion
//! automatically.
//!
//! # Example
//!
//! ```
//! use w3g_parser::binary::{read_u16_le, read_u32_le, read_bytes, read_string};
//!
//! let data = [0x26, 0x89, 0x01, 0x00, b'H', b'i', 0x00];
//!
//! // Read a little-endian u16 at offset 0
//! let value = read_u16_le(&data, 0).unwrap();
//! assert_eq!(value, 0x8926);
//!
//! // Read a little-endian u32 at offset 0
//! let value = read_u32_le(&data, 0).unwrap();
//! assert_eq!(value, 0x00018926);
//!
//! // Read a null-terminated string at offset 4
//! let s = read_string(&data, 4, 10).unwrap();
//! assert_eq!(s, "Hi");
//! ```

use crate::error::{ParserError, Result};

/// Reads a little-endian u16 value from the byte buffer at the given offset.
///
/// # Arguments
///
/// * `bytes` - The byte buffer to read from
/// * `offset` - The byte offset where the u16 starts
///
/// # Errors
///
/// Returns `ParserError::UnexpectedEof` if the buffer doesn't contain
/// at least 2 bytes starting from the given offset.
///
/// # Example
///
/// ```
/// use w3g_parser::binary::read_u16_le;
///
/// let data = [0x34, 0x12, 0xFF, 0xFF];
/// assert_eq!(read_u16_le(&data, 0).unwrap(), 0x1234);
/// assert_eq!(read_u16_le(&data, 2).unwrap(), 0xFFFF);
/// ```
pub fn read_u16_le(bytes: &[u8], offset: usize) -> Result<u16> {
    const SIZE: usize = 2;

    if offset + SIZE > bytes.len() {
        return Err(ParserError::unexpected_eof(
            offset + SIZE,
            bytes.len(),
        ));
    }

    let slice = &bytes[offset..offset + SIZE];
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

/// Reads a little-endian u32 value from the byte buffer at the given offset.
///
/// # Arguments
///
/// * `bytes` - The byte buffer to read from
/// * `offset` - The byte offset where the u32 starts
///
/// # Errors
///
/// Returns `ParserError::UnexpectedEof` if the buffer doesn't contain
/// at least 4 bytes starting from the given offset.
///
/// # Example
///
/// ```
/// use w3g_parser::binary::read_u32_le;
///
/// let data = [0x78, 0x56, 0x34, 0x12];
/// assert_eq!(read_u32_le(&data, 0).unwrap(), 0x12345678);
/// ```
pub fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32> {
    const SIZE: usize = 4;

    if offset + SIZE > bytes.len() {
        return Err(ParserError::unexpected_eof(
            offset + SIZE,
            bytes.len(),
        ));
    }

    let slice = &bytes[offset..offset + SIZE];
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

/// Reads a slice of bytes from the buffer at the given offset.
///
/// # Arguments
///
/// * `bytes` - The byte buffer to read from
/// * `offset` - The byte offset where the slice starts
/// * `len` - The number of bytes to read
///
/// # Errors
///
/// Returns `ParserError::UnexpectedEof` if the buffer doesn't contain
/// at least `len` bytes starting from the given offset.
///
/// # Example
///
/// ```
/// use w3g_parser::binary::read_bytes;
///
/// let data = b"GRBN\x02\x00\x00\x00";
/// let magic = read_bytes(data, 0, 4).unwrap();
/// assert_eq!(magic, b"GRBN");
/// ```
pub fn read_bytes(bytes: &[u8], offset: usize, len: usize) -> Result<&[u8]> {
    if offset + len > bytes.len() {
        return Err(ParserError::unexpected_eof(
            offset + len,
            bytes.len(),
        ));
    }

    Ok(&bytes[offset..offset + len])
}

/// Reads a null-terminated string from the buffer at the given offset.
///
/// The string is read until a null byte (0x00) is encountered or
/// `max_len` bytes have been read, whichever comes first. The null
/// byte is not included in the returned string.
///
/// # Arguments
///
/// * `bytes` - The byte buffer to read from
/// * `offset` - The byte offset where the string starts
/// * `max_len` - The maximum number of bytes to scan for the null terminator
///
/// # Errors
///
/// - Returns `ParserError::UnexpectedEof` if offset is beyond the buffer
/// - Returns `ParserError::InvalidHeader` if the bytes are not valid UTF-8
///
/// # Example
///
/// ```
/// use w3g_parser::binary::read_string;
///
/// let data = b"Hello\x00World";
/// let s = read_string(data, 0, 10).unwrap();
/// assert_eq!(s, "Hello");
/// ```
pub fn read_string(bytes: &[u8], offset: usize, max_len: usize) -> Result<String> {
    // Ensure we have at least some bytes to read
    if offset >= bytes.len() {
        return Err(ParserError::unexpected_eof(offset + 1, bytes.len()));
    }

    // Determine the actual range we can scan
    let end = std::cmp::min(offset + max_len, bytes.len());
    let search_slice = &bytes[offset..end];

    // Find the null terminator within the search range
    let string_len = search_slice
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(search_slice.len());

    let string_bytes = &search_slice[..string_len];

    // Convert to UTF-8 string
    String::from_utf8(string_bytes.to_vec()).map_err(|e| ParserError::InvalidHeader {
        reason: format!("Invalid UTF-8 string at offset {offset}: {e}"),
    })
}

/// Reads a fixed-length string from the buffer, stripping null padding.
///
/// This is useful for reading fixed-size string fields that may be
/// null-padded. All trailing null bytes are removed from the result.
///
/// # Arguments
///
/// * `bytes` - The byte buffer to read from
/// * `offset` - The byte offset where the string starts
/// * `len` - The exact number of bytes in the fixed field
///
/// # Errors
///
/// - Returns `ParserError::UnexpectedEof` if offset + len is beyond the buffer
/// - Returns `ParserError::InvalidHeader` if the bytes are not valid UTF-8
///
/// # Example
///
/// ```
/// use w3g_parser::binary::read_fixed_string;
///
/// let data = b"Hi\x00\x00\x00\x00\x00\x00";
/// let s = read_fixed_string(data, 0, 8).unwrap();
/// assert_eq!(s, "Hi");
/// ```
pub fn read_fixed_string(bytes: &[u8], offset: usize, len: usize) -> Result<String> {
    let slice = read_bytes(bytes, offset, len)?;

    // Find the first null byte or use the entire length
    let string_len = slice.iter().position(|&b| b == 0).unwrap_or(len);

    String::from_utf8(slice[..string_len].to_vec()).map_err(|e| ParserError::InvalidHeader {
        reason: format!("Invalid UTF-8 string at offset {offset}: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================
    // read_u16_le tests
    // ========================

    #[test]
    fn test_read_u16_le_basic() {
        let data = [0x34, 0x12];
        assert_eq!(read_u16_le(&data, 0).unwrap(), 0x1234);
    }

    #[test]
    fn test_read_u16_le_with_offset() {
        let data = [0x00, 0x00, 0x34, 0x12, 0xFF, 0xFF];
        assert_eq!(read_u16_le(&data, 2).unwrap(), 0x1234);
        assert_eq!(read_u16_le(&data, 4).unwrap(), 0xFFFF);
    }

    #[test]
    fn test_read_u16_le_at_end() {
        let data = [0x00, 0x00, 0x34, 0x12];
        assert_eq!(read_u16_le(&data, 2).unwrap(), 0x1234);
    }

    #[test]
    fn test_read_u16_le_overflow() {
        let data = [0x34, 0x12];
        let result = read_u16_le(&data, 1);
        assert!(matches!(
            result,
            Err(ParserError::UnexpectedEof {
                expected: 3,
                available: 2
            })
        ));
    }

    #[test]
    fn test_read_u16_le_empty() {
        let data: [u8; 0] = [];
        let result = read_u16_le(&data, 0);
        assert!(matches!(result, Err(ParserError::UnexpectedEof { .. })));
    }

    #[test]
    fn test_read_u16_le_offset_beyond_buffer() {
        let data = [0x34, 0x12];
        let result = read_u16_le(&data, 10);
        assert!(matches!(result, Err(ParserError::UnexpectedEof { .. })));
    }

    // ========================
    // read_u32_le tests
    // ========================

    #[test]
    fn test_read_u32_le_basic() {
        let data = [0x78, 0x56, 0x34, 0x12];
        assert_eq!(read_u32_le(&data, 0).unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_u32_le_with_offset() {
        let data = [0x00, 0x00, 0x78, 0x56, 0x34, 0x12];
        assert_eq!(read_u32_le(&data, 2).unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_u32_le_real_file_size() {
        // From FORMAT.md: File size 100,646 stored as: 26 89 01 00
        let data = [0x26, 0x89, 0x01, 0x00];
        assert_eq!(read_u32_le(&data, 0).unwrap(), 100_646);
    }

    #[test]
    fn test_read_u32_le_overflow() {
        let data = [0x78, 0x56, 0x34, 0x12];
        let result = read_u32_le(&data, 1);
        assert!(matches!(result, Err(ParserError::UnexpectedEof { .. })));
    }

    #[test]
    fn test_read_u32_le_too_short() {
        let data = [0x78, 0x56, 0x34];
        let result = read_u32_le(&data, 0);
        assert!(matches!(
            result,
            Err(ParserError::UnexpectedEof {
                expected: 4,
                available: 3
            })
        ));
    }

    // ========================
    // read_bytes tests
    // ========================

    #[test]
    fn test_read_bytes_basic() {
        let data = b"GRBN\x02\x00\x00\x00";
        let magic = read_bytes(data, 0, 4).unwrap();
        assert_eq!(magic, b"GRBN");
    }

    #[test]
    fn test_read_bytes_with_offset() {
        let data = b"\x00\x00GRBN";
        let magic = read_bytes(data, 2, 4).unwrap();
        assert_eq!(magic, b"GRBN");
    }

    #[test]
    fn test_read_bytes_entire_buffer() {
        let data = b"GRBN";
        let result = read_bytes(data, 0, 4).unwrap();
        assert_eq!(result, data.as_slice());
    }

    #[test]
    fn test_read_bytes_overflow() {
        let data = b"GRBN";
        let result = read_bytes(data, 2, 4);
        assert!(matches!(
            result,
            Err(ParserError::UnexpectedEof {
                expected: 6,
                available: 4
            })
        ));
    }

    #[test]
    fn test_read_bytes_zero_length() {
        let data = b"GRBN";
        let result = read_bytes(data, 2, 0).unwrap();
        assert_eq!(result, &[] as &[u8]);
    }

    // ========================
    // read_string tests
    // ========================

    #[test]
    fn test_read_string_basic() {
        let data = b"Hello\x00World";
        let s = read_string(data, 0, 20).unwrap();
        assert_eq!(s, "Hello");
    }

    #[test]
    fn test_read_string_with_offset() {
        let data = b"Hello\x00World\x00End";
        let s = read_string(data, 6, 20).unwrap();
        assert_eq!(s, "World");
    }

    #[test]
    fn test_read_string_no_null() {
        // If no null terminator within max_len, returns all bytes
        let data = b"HelloWorld";
        let s = read_string(data, 0, 5).unwrap();
        assert_eq!(s, "Hello");
    }

    #[test]
    fn test_read_string_empty() {
        let data = b"\x00Hello";
        let s = read_string(data, 0, 10).unwrap();
        assert_eq!(s, "");
    }

    #[test]
    fn test_read_string_classic_magic() {
        // Test reading the Classic format magic string
        let data = b"Warcraft III recorded game\x1A\x00";
        let s = read_string(data, 0, 26).unwrap();
        assert_eq!(s, "Warcraft III recorded game");
    }

    #[test]
    fn test_read_string_offset_at_end() {
        let data = b"Hello";
        let result = read_string(data, 5, 10);
        assert!(matches!(result, Err(ParserError::UnexpectedEof { .. })));
    }

    #[test]
    fn test_read_string_invalid_utf8() {
        let data = [0xFF, 0xFE, 0x00]; // Invalid UTF-8 sequence
        let result = read_string(&data, 0, 10);
        assert!(matches!(result, Err(ParserError::InvalidHeader { .. })));
    }

    // ========================
    // read_fixed_string tests
    // ========================

    #[test]
    fn test_read_fixed_string_basic() {
        let data = b"Hi\x00\x00\x00\x00\x00\x00";
        let s = read_fixed_string(data, 0, 8).unwrap();
        assert_eq!(s, "Hi");
    }

    #[test]
    fn test_read_fixed_string_full_length() {
        let data = b"HelloWor";
        let s = read_fixed_string(data, 0, 8).unwrap();
        assert_eq!(s, "HelloWor");
    }

    #[test]
    fn test_read_fixed_string_with_offset() {
        let data = b"\x00\x00Hi\x00\x00";
        let s = read_fixed_string(data, 2, 4).unwrap();
        assert_eq!(s, "Hi");
    }

    #[test]
    fn test_read_fixed_string_overflow() {
        let data = b"Hi";
        let result = read_fixed_string(data, 0, 8);
        assert!(matches!(result, Err(ParserError::UnexpectedEof { .. })));
    }

    // ========================
    // Integration tests
    // ========================

    #[test]
    fn test_parse_grbn_header_fields() {
        // Simulated GRBN header start (based on FORMAT.md)
        let header: &[u8] = &[
            0x47, 0x52, 0x42, 0x4E, // 0x00: Magic "GRBN"
            0x02, 0x00, 0x00, 0x00, // 0x04: Version = 2
            0x0B, 0x00, 0x00, 0x00, // 0x08: Unknown_1 = 11
            0x00, 0xC8, 0x00, 0x00, // 0x0C: Unknown_2 = 51200
        ];

        let magic = read_bytes(header, 0, 4).unwrap();
        assert_eq!(magic, b"GRBN");

        let version = read_u32_le(header, 4).unwrap();
        assert_eq!(version, 2);

        let unknown1 = read_u32_le(header, 8).unwrap();
        assert_eq!(unknown1, 11);

        let unknown2 = read_u32_le(header, 12).unwrap();
        assert_eq!(unknown2, 51200);
    }

    #[test]
    fn test_parse_classic_header_fields() {
        // Simulated Classic header fields (based on FORMAT.md)
        #[rustfmt::skip]
        let header: &[u8] = &[
            // 0x1C: Header size = 68
            0x44, 0x00, 0x00, 0x00,
            // 0x20: File size = 100,646
            0x26, 0x89, 0x01, 0x00,
            // 0x24: Header version = 1
            0x01, 0x00, 0x00, 0x00,
        ];

        let header_size = read_u32_le(header, 0).unwrap();
        assert_eq!(header_size, 68);

        let file_size = read_u32_le(header, 4).unwrap();
        assert_eq!(file_size, 100_646);

        let header_version = read_u32_le(header, 8).unwrap();
        assert_eq!(header_version, 1);
    }

    #[test]
    fn test_block_header_type_a() {
        // Type A block header (8 bytes) from FORMAT.md
        let block_header: &[u8] = &[
            0x79, 0x0C, // Compressed size = 0x0C79 = 3193
            0x00, 0x20, // Decompressed size = 0x2000 = 8192
            0xFE, 0xC0, 0x86, 0x26, // Checksum
        ];

        let compressed_size = read_u16_le(block_header, 0).unwrap();
        assert_eq!(compressed_size, 3193);

        let decompressed_size = read_u16_le(block_header, 2).unwrap();
        assert_eq!(decompressed_size, 8192);
    }

    #[test]
    fn test_block_header_type_b() {
        // Type B block header (12 bytes) from FORMAT.md
        let block_header: &[u8] = &[
            0x26, 0x0E, // Compressed size = 0x0E26 = 3622
            0x00, 0x00, // Padding
            0x00, 0x20, // Decompressed size = 0x2000 = 8192
            0x00, 0x00, // Padding
            0xFF, 0xBB, 0xA2, 0x82, // Checksum
        ];

        let compressed_size = read_u16_le(block_header, 0).unwrap();
        assert_eq!(compressed_size, 3622);

        let decompressed_size = read_u16_le(block_header, 4).unwrap();
        assert_eq!(decompressed_size, 8192);
    }
}
