//! Error types for the W3G replay parser.
//!
//! This module defines a comprehensive error hierarchy for handling all
//! failure cases during replay parsing, including I/O errors, format
//! validation failures, and decompression issues.

use thiserror::Error;

/// The main error type for W3G replay parsing operations.
///
/// This enum covers all error cases that can occur during parsing:
/// - File I/O failures
/// - Invalid or unrecognized format markers
/// - Malformed header structures
/// - Decompression failures
/// - Truncated or incomplete data
///
/// # Example
///
/// ```
/// use w3g_parser::error::{ParserError, Result};
///
/// fn example_operation() -> Result<()> {
///     // Operations that may fail return Result<T>
///     Err(ParserError::InvalidHeader {
///         reason: "Missing required field".to_string(),
///     })
/// }
/// ```
#[derive(Error, Debug)]
pub enum ParserError {
    /// An I/O error occurred while reading the replay file.
    ///
    /// This wraps standard library I/O errors for seamless error propagation
    /// using the `?` operator.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// The file's magic bytes do not match any known W3G format.
    ///
    /// W3G replays must start with either:
    /// - `GRBN` (0x4752424E) for Reforged format
    /// - `Warcraft III recorded game\x1A\x00` (28 bytes) for Classic format
    #[error("Invalid magic bytes: expected {expected}, found {found}")]
    InvalidMagic {
        /// The expected magic bytes (as hex string for display).
        expected: String,
        /// The actual bytes found at the start of the file (as hex string).
        found: String,
    },

    /// The replay header is malformed or contains invalid data.
    ///
    /// This error is returned when header fields fail validation checks,
    /// such as unexpected values in required fields.
    #[error("Invalid header: {reason}")]
    InvalidHeader {
        /// A description of what makes the header invalid.
        reason: String,
    },

    /// Decompression of replay data failed.
    ///
    /// W3G replays use zlib compression. This error occurs when the
    /// compressed data is corrupted or uses an unsupported compression
    /// method.
    #[error("Decompression failed: {reason}")]
    DecompressionError {
        /// A description of the decompression failure.
        reason: String,
    },

    /// The data ended unexpectedly before the required bytes could be read.
    ///
    /// This typically indicates a truncated replay file.
    #[error("Unexpected end of data: expected {expected} bytes, but only {available} available")]
    UnexpectedEof {
        /// The number of bytes that were expected to be available.
        expected: usize,
        /// The actual number of bytes available.
        available: usize,
    },
}

impl ParserError {
    /// Creates an `InvalidMagic` error with the given byte slices.
    ///
    /// The bytes are converted to hex strings for human-readable display.
    ///
    /// # Arguments
    ///
    /// * `expected` - The expected magic bytes
    /// * `found` - The actual bytes found
    ///
    /// # Example
    ///
    /// ```
    /// use w3g_parser::error::ParserError;
    ///
    /// let err = ParserError::invalid_magic(b"GRBN", b"\x00\x00\x00\x00");
    /// assert!(err.to_string().contains("Invalid magic bytes"));
    /// ```
    #[must_use]
    pub fn invalid_magic(expected: &[u8], found: &[u8]) -> Self {
        ParserError::InvalidMagic {
            expected: bytes_to_hex(expected),
            found: bytes_to_hex(found),
        }
    }

    /// Creates an `UnexpectedEof` error with the given sizes.
    ///
    /// # Arguments
    ///
    /// * `expected` - The number of bytes that were needed
    /// * `available` - The number of bytes actually available
    #[must_use]
    pub fn unexpected_eof(expected: usize, available: usize) -> Self {
        ParserError::UnexpectedEof { expected, available }
    }
}

/// Converts a byte slice to a hexadecimal string representation.
///
/// If the slice is 8 bytes or less, formats as space-separated hex values.
/// If longer, shows the first 8 bytes followed by "...".
fn bytes_to_hex(bytes: &[u8]) -> String {
    if bytes.len() <= 8 {
        bytes
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        let prefix: String = bytes[..8]
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(" ");
        format!("{prefix}... ({} bytes total)", bytes.len())
    }
}

/// A specialized Result type for W3G parsing operations.
///
/// This is a convenience alias that uses `ParserError` as the error type.
pub type Result<T> = std::result::Result<T, ParserError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_error_display() {
        let err = ParserError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert!(err.to_string().contains("I/O error"));

        let err = ParserError::invalid_magic(b"GRBN", b"\x00\x01\x02\x03");
        assert!(err.to_string().contains("Invalid magic bytes"));

        let err = ParserError::InvalidHeader {
            reason: "missing field".to_string(),
        };
        assert!(err.to_string().contains("Invalid header"));
        assert!(err.to_string().contains("missing field"));

        let err = ParserError::DecompressionError {
            reason: "invalid zlib stream".to_string(),
        };
        assert!(err.to_string().contains("Decompression failed"));

        let err = ParserError::unexpected_eof(128, 64);
        assert!(err.to_string().contains("expected 128 bytes"));
        assert!(err.to_string().contains("64 available"));
    }

    #[test]
    fn test_bytes_to_hex_short() {
        let result = bytes_to_hex(b"GRBN");
        assert_eq!(result, "47 52 42 4E");
    }

    #[test]
    fn test_bytes_to_hex_long() {
        let bytes = b"Warcraft III recorded game";
        let result = bytes_to_hex(bytes);
        assert!(result.contains("..."));
        assert!(result.contains("26 bytes total"));
    }

    #[test]
    fn test_invalid_magic_helper() {
        let err = ParserError::invalid_magic(b"GRBN", b"BAD!");
        match err {
            ParserError::InvalidMagic { expected, found } => {
                assert_eq!(expected, "47 52 42 4E");
                assert_eq!(found, "42 41 44 21");
            }
            _ => panic!("Expected InvalidMagic variant"),
        }
    }

    #[test]
    fn test_error_is_send_sync() {
        // Ensure our error type can be used across threads
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ParserError>();
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "test error");
        let parser_err: ParserError = io_err.into();
        match parser_err {
            ParserError::IoError(_) => {}
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<u32> {
            Ok(42)
        }
        assert_eq!(returns_result().unwrap(), 42);

        fn returns_error() -> Result<u32> {
            Err(ParserError::InvalidHeader {
                reason: "test".to_string(),
            })
        }
        assert!(returns_error().is_err());
    }
}
