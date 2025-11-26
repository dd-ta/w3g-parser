//! Game record header parsing for decompressed W3G replay data.
//!
//! This module parses the initial game record header that appears at the start
//! of decompressed Classic replay data. The header contains host player information
//! and encoded game settings.
//!
//! # Format
//!
//! The game record header has the following structure:
//!
//! | Offset | Size | Type | Field |
//! |--------|------|------|-------|
//! | 0x00 | 4 | u32 LE | Record type (always 0x00000110) |
//! | 0x04 | 1 | u8 | Unknown (usually 0x00) |
//! | 0x05 | 1 | u8 | Host player slot |
//! | 0x06 | var | string | Host player name (null-terminated) |
//! | var | 1 | u8 | Host flags (0x01 or 0x02) |
//! | var | var | string | Additional data (null-terminated) |
//! | var | var | bytes | Encoded game settings |
//!
//! # Example
//!
//! ```ignore
//! use w3g_parser::records::GameRecordHeader;
//!
//! let decompressed_data = /* ... */;
//! let header = GameRecordHeader::parse(&decompressed_data)?;
//!
//! println!("Host: {} (slot {})", header.host_name, header.host_slot);
//! ```

use crate::binary::{read_string, read_u32_le};
use crate::error::{ParserError, Result};

/// Magic value for the game record header (0x10 0x01 0x00 0x00 as little-endian u32).
pub const GAME_RECORD_MAGIC: u32 = 0x0000_0110;

/// The initial game record that appears at the start of decompressed data.
///
/// This contains the host player information and encoded game settings.
/// The header is followed by player slot records (0x16 marker) and then
/// the action stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRecordHeader {
    /// Record type marker (always 0x00000110).
    pub record_type: u32,

    /// Unknown byte (usually 0x00).
    pub unknown_1: u8,

    /// Slot number of the host player (typically 1-12).
    pub host_slot: u8,

    /// Name of the host player (null-terminated in binary).
    pub host_name: String,

    /// Additional flag byte after host name (usually 0x01 or 0x02).
    pub host_flags: u8,

    /// Additional data after host name (clan tag, custom data, etc.).
    pub additional_data: String,

    /// Raw encoded game settings (encoding not yet reverse engineered).
    pub encoded_settings: Vec<u8>,

    /// Total bytes consumed by this record.
    pub byte_length: usize,
}

impl GameRecordHeader {
    /// Parses a game record header from decompressed replay data.
    ///
    /// # Arguments
    ///
    /// * `data` - The decompressed replay data (starting from offset 0)
    ///
    /// # Returns
    ///
    /// A `GameRecordHeader` containing the parsed fields.
    ///
    /// # Errors
    ///
    /// - `ParserError::InvalidHeader` if the magic bytes don't match
    /// - `ParserError::UnexpectedEof` if the data is truncated
    pub fn parse(data: &[u8]) -> Result<Self> {
        // Read and validate record type (4 bytes)
        let record_type = read_u32_le(data, 0)?;
        if record_type != GAME_RECORD_MAGIC {
            return Err(ParserError::InvalidHeader {
                reason: format!(
                    "Invalid game record magic: expected 0x{GAME_RECORD_MAGIC:08X}, found 0x{record_type:08X}"
                ),
            });
        }

        // Read unknown byte at offset 4
        if data.len() < 6 {
            return Err(ParserError::unexpected_eof(6, data.len()));
        }
        let unknown_1 = data[4];

        // Read host slot at offset 5
        let host_slot = data[5];

        // Read host name (null-terminated string starting at offset 6)
        let host_name = read_string(data, 6, 256)?;
        let host_name_end = 6 + host_name.len() + 1; // +1 for null terminator

        // Read flags byte after null terminator
        if data.len() <= host_name_end {
            return Err(ParserError::unexpected_eof(host_name_end + 1, data.len()));
        }
        let host_flags = data[host_name_end];

        // Read additional data string (null-terminated)
        let additional_data = read_string(data, host_name_end + 1, 256)?;
        let additional_data_end = host_name_end + 1 + additional_data.len() + 1;

        // Find the end of encoded settings by scanning for the first player slot marker (0x16)
        // or the slot record marker (0x19)
        let encoded_settings_start = additional_data_end;
        let encoded_settings_end = find_settings_boundary(data, encoded_settings_start);

        let encoded_settings = if encoded_settings_end > encoded_settings_start {
            data[encoded_settings_start..encoded_settings_end].to_vec()
        } else {
            Vec::new()
        };

        Ok(GameRecordHeader {
            record_type,
            unknown_1,
            host_slot,
            host_name,
            host_flags,
            additional_data,
            encoded_settings,
            byte_length: encoded_settings_end,
        })
    }

    /// Returns whether this is a valid game record header.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.record_type == GAME_RECORD_MAGIC && !self.host_name.is_empty()
    }

    /// Returns the offset where player slot records begin.
    #[must_use]
    pub fn player_records_offset(&self) -> usize {
        self.byte_length
    }

    /// Extracts the game name (lobby name) from encoded settings.
    ///
    /// The game name is stored as a null-terminated string starting at offset 1
    /// when the first byte is 0x00 (Variant A). Returns empty string if no game
    /// name is present or the format is not recognized.
    #[must_use]
    pub fn game_name(&self) -> String {
        if self.encoded_settings.is_empty() {
            return String::new();
        }

        // Variant A: First byte is 0x00, game name follows at offset 1
        if self.encoded_settings[0] == 0x00 && self.encoded_settings.len() > 1 {
            // Find null terminator
            for i in 1..self.encoded_settings.len() {
                if self.encoded_settings[i] == 0x00 {
                    if i > 1 {
                        return String::from_utf8_lossy(&self.encoded_settings[1..i]).to_string();
                    }
                    break;
                }
            }
        }

        String::new()
    }

    /// Extracts the map path from encoded settings.
    ///
    /// The map path is obfuscated by interleaving with non-ASCII bytes.
    /// This method extracts all printable ASCII characters to recover the path.
    /// The result may contain fragments of the actual path due to the encoding.
    ///
    /// Returns None if no map path can be extracted.
    #[must_use]
    pub fn map_path_raw(&self) -> Option<String> {
        if self.encoded_settings.len() < 20 {
            return None;
        }

        // Skip past game name and initial settings bytes
        // Map path typically starts around offset 0x10-0x20
        let start = self.find_map_path_start();

        // Extract all printable ASCII bytes until we hit a long sequence of non-printable
        let mut path_chars = Vec::new();
        let mut non_printable_streak = 0;

        for &byte in &self.encoded_settings[start..] {
            if byte >= 0x20 && byte <= 0x7E {
                path_chars.push(byte);
                non_printable_streak = 0;
            } else if byte == 0x00 {
                // Null terminator - end of path
                break;
            } else {
                non_printable_streak += 1;
                // If we've seen many non-printable bytes, we've probably passed the path
                if non_printable_streak > 10 && path_chars.len() > 10 {
                    break;
                }
            }
        }

        if path_chars.len() >= 5 {
            Some(String::from_utf8_lossy(&path_chars).to_string())
        } else {
            None
        }
    }

    /// Finds the likely start offset of the map path within encoded settings.
    fn find_map_path_start(&self) -> usize {
        // Look for common map path patterns:
        // - "Maps" or "maps"
        // - ".w3x" or ".w3m" file extensions
        // - Backslash or forward slash path separators

        for i in 0..self.encoded_settings.len().saturating_sub(4) {
            let b = &self.encoded_settings[i..];

            // Check for "Maps" (case insensitive)
            if b.len() >= 4 {
                let chunk = [b[0], b[1], b[2], b[3]];
                if matches!(
                    &chunk,
                    b"Maps" | b"maps" | b"MAPS" | b"Maqs" | b"maqs"
                ) {
                    return i;
                }
            }

            // Check for path separators with nearby ASCII
            if (b[0] == b'/' || b[0] == b'\\')
                && b.len() > 1
                && b[1].is_ascii_alphanumeric()
            {
                // Go back a bit to include directory name
                return i.saturating_sub(8);
            }
        }

        // Default: skip past game name section (roughly offset 0x0C-0x15)
        if self.encoded_settings[0] == 0x00 {
            // Variant A: skip game name + settings bytes
            for i in 1..self.encoded_settings.len().min(50) {
                if self.encoded_settings[i] == 0x00 {
                    return (i + 13).min(self.encoded_settings.len()); // Skip 13 settings bytes
                }
            }
        }

        // Variant B or fallback
        12.min(self.encoded_settings.len())
    }
}

/// Finds the boundary where encoded settings end and player records begin.
///
/// Scans for valid player slot record patterns. We ONLY look for 0x16/0x19
/// markers followed by valid slot IDs and player name characters, because
/// bytes like 0x1F, 0x20, 0x22 can appear within encoded settings data.
fn find_settings_boundary(data: &[u8], start: usize) -> usize {
    // Player slot records start with 0x16, followed by a slot ID (1-24 typically)
    // We need to be careful not to match 0x16 bytes that are part of encoded settings
    // Look for pattern: 0x16 followed by a reasonable slot number (0x01-0x18)
    // followed by what looks like a player name (ASCII printable characters)
    //
    // IMPORTANT: We do NOT early-return on 0x1F/0x20/0x22 because these bytes
    // can appear within encoded settings data (especially in build 10000+).
    // We only stop when we find a valid player slot pattern.

    for i in start..data.len().saturating_sub(2) {
        match data[i] {
            0x16 => {
                // Check if this looks like a valid player slot record
                // Slot ID should be 1-24, and the next byte should be printable ASCII
                // or the start of a player name
                let slot_id = data[i + 1];
                if (1..=24).contains(&slot_id) && is_valid_name_start(data, i + 2) {
                    return i;
                }
            }
            0x19 => {
                // Alternative slot record marker
                // Similar validation as 0x16 - must have valid slot ID AND name start
                let slot_id = data[i + 1];
                if (1..=24).contains(&slot_id) && is_valid_name_start(data, i + 2) {
                    return i;
                }
            }
            _ => {}
        }
    }

    // If no marker found, return end of data
    data.len()
}

/// Checks if a position in the data looks like the start of a valid player name.
///
/// A valid name start must be:
/// 1. A printable ASCII character (0x20-0x7E), OR
/// 2. A valid UTF-8 multi-byte sequence start (0xC0-0xF4) followed by valid continuation bytes
///
/// Additionally, there must be a null terminator within a reasonable distance (256 bytes).
fn is_valid_name_start(data: &[u8], pos: usize) -> bool {
    if pos >= data.len() {
        return false;
    }

    let first_byte = data[pos];

    // ASCII printable character (most common case)
    if (0x20..=0x7E).contains(&first_byte) {
        // Verify there's a null terminator within 256 bytes
        return has_null_terminator(data, pos, 256);
    }

    // UTF-8 multi-byte sequence start
    // 2-byte sequences: 0xC2-0xDF (0xC0-0xC1 are overlong encodings)
    // 3-byte sequences: 0xE0-0xEF
    // 4-byte sequences: 0xF0-0xF4
    if first_byte >= 0xC2 && first_byte <= 0xF4 {
        // Check that following bytes are valid UTF-8 continuation bytes (0x80-0xBF)
        let expected_continuation = match first_byte {
            0xC2..=0xDF => 1,
            0xE0..=0xEF => 2,
            0xF0..=0xF4 => 3,
            _ => return false,
        };

        // Verify we have enough bytes and they're valid continuations
        if pos + expected_continuation >= data.len() {
            return false;
        }

        for offset in 1..=expected_continuation {
            let cont_byte = data[pos + offset];
            if !(0x80..=0xBF).contains(&cont_byte) {
                return false;
            }
        }

        // Verify there's a null terminator within 256 bytes
        return has_null_terminator(data, pos, 256);
    }

    false
}

/// Checks if there's a null terminator within max_len bytes from pos.
fn has_null_terminator(data: &[u8], pos: usize, max_len: usize) -> bool {
    let end = (pos + max_len).min(data.len());
    data[pos..end].iter().any(|&b| b == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_record_header_parse_basic() {
        // Construct a minimal game record header
        let mut data = Vec::new();

        // Record type: 0x00000110
        data.extend_from_slice(&[0x10, 0x01, 0x00, 0x00]);

        // Unknown byte
        data.push(0x00);

        // Host slot: 3
        data.push(0x03);

        // Host name: "kaiseris" + null
        data.extend_from_slice(b"kaiseris\x00");

        // Flags: 0x01
        data.push(0x01);

        // Additional data: "rich" + null
        data.extend_from_slice(b"rich\x00");

        // Some encoded settings bytes (arbitrary)
        data.extend_from_slice(&[0x81, 0x03, 0x79, 0x07]);

        // Player slot marker to end settings (with full null-terminated name)
        data.push(0x16);
        data.push(0x04); // Slot 4
        data.extend_from_slice(b"TestPlayer\x00"); // Full name with null terminator

        let header = GameRecordHeader::parse(&data).unwrap();

        assert_eq!(header.record_type, GAME_RECORD_MAGIC);
        assert_eq!(header.unknown_1, 0x00);
        assert_eq!(header.host_slot, 3);
        assert_eq!(header.host_name, "kaiseris");
        assert_eq!(header.host_flags, 0x01);
        assert_eq!(header.additional_data, "rich");
        assert_eq!(header.encoded_settings, vec![0x81, 0x03, 0x79, 0x07]);
        assert!(header.is_valid());
    }

    #[test]
    fn test_game_record_header_invalid_magic() {
        let data = [0x00, 0x00, 0x00, 0x00, 0x00, 0x03, b'T', b'e', b's', b't', 0x00];
        let result = GameRecordHeader::parse(&data);
        assert!(matches!(result, Err(ParserError::InvalidHeader { .. })));
    }

    #[test]
    fn test_game_record_header_truncated() {
        let data = [0x10, 0x01, 0x00, 0x00];
        let result = GameRecordHeader::parse(&data);
        assert!(matches!(result, Err(ParserError::UnexpectedEof { .. })));
    }

    #[test]
    fn test_game_record_header_empty_additional_data() {
        let mut data = Vec::new();

        // Record type
        data.extend_from_slice(&[0x10, 0x01, 0x00, 0x00]);

        // Unknown byte
        data.push(0x00);

        // Host slot
        data.push(0x01);

        // Host name: "Player" + null
        data.extend_from_slice(b"Player\x00");

        // Flags
        data.push(0x02);

        // Empty additional data (just null terminator)
        data.push(0x00);

        // Player slot marker (with full null-terminated name)
        data.push(0x16);
        data.push(0x02);
        data.extend_from_slice(b"Other\x00"); // Full name with null terminator

        let header = GameRecordHeader::parse(&data).unwrap();

        assert_eq!(header.host_name, "Player");
        assert_eq!(header.additional_data, "");
        assert!(header.encoded_settings.is_empty());
    }

    #[test]
    fn test_game_record_header_battlenet_name() {
        let mut data = Vec::new();

        // Record type
        data.extend_from_slice(&[0x10, 0x01, 0x00, 0x00]);

        // Unknown and slot
        data.extend_from_slice(&[0x00, 0x01]);

        // Battle.net style name
        data.extend_from_slice(b"MisterWinner#21670\x00");

        // Flags
        data.push(0x02);

        // Additional data with FLO-STREAM
        data.extend_from_slice(b"\x00"); // Empty additional data

        // Some settings bytes
        data.extend_from_slice(&[0x46, 0x4C, 0x4F]); // "FLO"

        // Player slot marker
        data.push(0x16);
        data.push(0x02);
        data.push(b'L');

        let header = GameRecordHeader::parse(&data).unwrap();

        assert_eq!(header.host_name, "MisterWinner#21670");
        assert_eq!(header.host_slot, 1);
    }

    #[test]
    fn test_find_settings_boundary() {
        // Test data with a clear player slot marker (include null terminator after name)
        let data = [0x81, 0x03, 0x79, 0x07, 0x16, 0x04, b'G', b'r', b'e', b'e', b'n', 0x00];

        let boundary = find_settings_boundary(&data, 0);
        assert_eq!(boundary, 4);
    }

    #[test]
    fn test_find_settings_boundary_no_marker() {
        // Test data with no valid marker
        let data = [0x81, 0x03, 0x79, 0x07, 0x05, 0x06, 0x07, 0x08];

        let boundary = find_settings_boundary(&data, 0);
        assert_eq!(boundary, data.len());
    }
}
