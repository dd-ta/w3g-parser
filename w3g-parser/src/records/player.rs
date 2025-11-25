//! Player slot record parsing for decompressed W3G replay data.
//!
//! This module parses player slot records that appear after the game record header.
//! Each player slot record (0x16 marker) contains a player's slot ID and name.
//!
//! # Format
//!
//! Player slot records have the following structure:
//!
//! | Offset | Size | Type | Field |
//! |--------|------|------|-------|
//! | 0 | 1 | u8 | Record type (0x16) |
//! | 1 | 1 | u8 | Slot ID |
//! | 2 | var | string | Player name (null-terminated) |
//! | var | 6 | bytes | Trailing data (flags, race info, etc.) |
//!
//! # Example
//!
//! ```ignore
//! use w3g_parser::records::{GameRecordHeader, PlayerRoster};
//!
//! let decompressed_data = /* ... */;
//! let header = GameRecordHeader::parse(&decompressed_data)?;
//! let roster = PlayerRoster::parse(&decompressed_data[header.byte_length..])?;
//!
//! for player in roster.players() {
//!     println!("Slot {}: {}", player.slot_id, player.player_name);
//! }
//! ```

use crate::binary::read_string;
use crate::error::{ParserError, Result};

/// Record type marker for player slot records.
pub const PLAYER_SLOT_MARKER: u8 = 0x16;

/// Record type marker for alternative slot records.
pub const SLOT_RECORD_MARKER: u8 = 0x19;

/// A player slot record (0x16 marker).
///
/// These records appear after the game header and define all players
/// participating in the game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerSlot {
    /// Slot number (typically 1-24 for larger games).
    pub slot_id: u8,

    /// Player name (null-terminated in binary).
    pub player_name: String,

    /// Trailing data (usually includes flags, may contain race info).
    /// Typically 6 bytes: 0x01 0x00 0x00 0x00 0x00 0x00 or similar.
    pub trailing_data: Vec<u8>,

    /// Total bytes consumed by this record.
    pub byte_length: usize,
}

impl PlayerSlot {
    /// Parses a player slot record from decompressed replay data.
    ///
    /// # Arguments
    ///
    /// * `data` - The decompressed replay data starting at a 0x16 marker
    ///
    /// # Returns
    ///
    /// A `PlayerSlot` containing the parsed fields.
    ///
    /// # Errors
    ///
    /// - `ParserError::InvalidHeader` if the marker byte is not 0x16
    /// - `ParserError::UnexpectedEof` if the data is truncated
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.is_empty() {
            return Err(ParserError::unexpected_eof(1, 0));
        }

        // Validate marker byte
        if data[0] != PLAYER_SLOT_MARKER {
            return Err(ParserError::InvalidHeader {
                reason: format!(
                    "Invalid player slot marker: expected 0x{PLAYER_SLOT_MARKER:02X}, found 0x{:02X}",
                    data[0]
                ),
            });
        }

        if data.len() < 3 {
            return Err(ParserError::unexpected_eof(3, data.len()));
        }

        // Read slot ID at offset 1
        let slot_id = data[1];

        // Read player name starting at offset 2
        let player_name = read_string(data, 2, 256)?;
        let name_end = 2 + player_name.len() + 1; // +1 for null terminator

        // Read trailing data (variable length, scan for next record or end)
        let trailing_end = find_trailing_data_end(data, name_end);
        let trailing_data = if trailing_end > name_end {
            data[name_end..trailing_end].to_vec()
        } else {
            Vec::new()
        };

        Ok(PlayerSlot {
            slot_id,
            player_name,
            trailing_data,
            byte_length: trailing_end,
        })
    }

    /// Returns whether this appears to be a valid player slot.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.player_name.is_empty() && self.slot_id <= 24
    }
}

/// Alternative slot record format (0x19 marker).
///
/// Some replays use this format for additional player information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotRecord {
    /// Slot number.
    pub slot_id: u8,

    /// Player name.
    pub player_name: String,

    /// Additional data (variable length).
    pub additional_data: Vec<u8>,

    /// Total bytes consumed.
    pub byte_length: usize,
}

impl SlotRecord {
    /// Parses a slot record from decompressed replay data.
    ///
    /// # Arguments
    ///
    /// * `data` - The decompressed replay data starting at a 0x19 marker
    ///
    /// # Returns
    ///
    /// A `SlotRecord` containing the parsed fields.
    ///
    /// # Errors
    ///
    /// - `ParserError::InvalidHeader` if the marker byte is not 0x19
    /// - `ParserError::UnexpectedEof` if the data is truncated
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.is_empty() {
            return Err(ParserError::unexpected_eof(1, 0));
        }

        // Validate marker byte
        if data[0] != SLOT_RECORD_MARKER {
            return Err(ParserError::InvalidHeader {
                reason: format!(
                    "Invalid slot record marker: expected 0x{SLOT_RECORD_MARKER:02X}, found 0x{:02X}",
                    data[0]
                ),
            });
        }

        if data.len() < 3 {
            return Err(ParserError::unexpected_eof(3, data.len()));
        }

        // Read slot ID at offset 1
        let slot_id = data[1];

        // Read player name starting at offset 2
        let player_name = read_string(data, 2, 256)?;
        let name_end = 2 + player_name.len() + 1;

        // Find the end of additional data
        let additional_end = find_trailing_data_end(data, name_end);
        let additional_data = if additional_end > name_end {
            data[name_end..additional_end].to_vec()
        } else {
            Vec::new()
        };

        Ok(SlotRecord {
            slot_id,
            player_name,
            additional_data,
            byte_length: additional_end,
        })
    }
}

/// Unified player record that can be either format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerRecord {
    /// Standard player slot (0x16).
    PlayerSlot(PlayerSlot),

    /// Alternative slot record (0x19).
    SlotRecord(SlotRecord),
}

impl PlayerRecord {
    /// Returns the slot ID for this player record.
    #[must_use]
    pub fn slot_id(&self) -> u8 {
        match self {
            PlayerRecord::PlayerSlot(p) => p.slot_id,
            PlayerRecord::SlotRecord(s) => s.slot_id,
        }
    }

    /// Returns the player name for this record.
    #[must_use]
    pub fn player_name(&self) -> &str {
        match self {
            PlayerRecord::PlayerSlot(p) => &p.player_name,
            PlayerRecord::SlotRecord(s) => &s.player_name,
        }
    }

    /// Returns the byte length of this record.
    #[must_use]
    pub fn byte_length(&self) -> usize {
        match self {
            PlayerRecord::PlayerSlot(p) => p.byte_length,
            PlayerRecord::SlotRecord(s) => s.byte_length,
        }
    }
}

/// Collection of all players in a game.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerRoster {
    /// All player records found.
    players: Vec<PlayerRecord>,

    /// Total bytes consumed by all player records.
    pub byte_length: usize,
}

impl PlayerRoster {
    /// Parses a player roster from decompressed replay data.
    ///
    /// This function scans for player slot records (0x16) and slot records (0x19)
    /// starting from the provided data, continuing until it encounters a different
    /// record type (such as a `TimeFrame` marker 0x1F or extended metadata 0x38).
    ///
    /// # Arguments
    ///
    /// * `data` - The decompressed replay data starting after the game record header
    ///
    /// # Returns
    ///
    /// A `PlayerRoster` containing all parsed player records.
    ///
    /// # Errors
    ///
    /// - `ParserError::UnexpectedEof` if the data is truncated during parsing
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut players = Vec::new();
        let mut offset = 0;

        while offset < data.len() {
            match data[offset] {
                PLAYER_SLOT_MARKER => {
                    let slot = PlayerSlot::parse(&data[offset..])?;
                    offset += slot.byte_length;
                    players.push(PlayerRecord::PlayerSlot(slot));
                }
                SLOT_RECORD_MARKER => {
                    let record = SlotRecord::parse(&data[offset..])?;
                    offset += record.byte_length;
                    players.push(PlayerRecord::SlotRecord(record));
                }
                0x00 => {
                    // Padding byte, skip it
                    offset += 1;
                }
                _ => {
                    // Not a player record marker, stop parsing
                    break;
                }
            }
        }

        // Skip extended metadata section (build 10100+)
        // Extended metadata starts with 0x38 records and contains player info
        // We need to find where TimeFrame records actually begin
        let extended_metadata_end = find_extended_metadata_end(data, offset);

        Ok(PlayerRoster {
            players,
            byte_length: extended_metadata_end,
        })
    }

    /// Returns the number of players in the roster.
    #[must_use]
    pub fn len(&self) -> usize {
        self.players.len()
    }

    /// Returns whether the roster is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    /// Returns an iterator over all player records.
    pub fn players(&self) -> impl Iterator<Item = &PlayerRecord> {
        self.players.iter()
    }

    /// Returns a slice of all player records.
    #[must_use]
    pub fn as_slice(&self) -> &[PlayerRecord] {
        &self.players
    }

    /// Returns all player names as a vector.
    #[must_use]
    pub fn player_names(&self) -> Vec<&str> {
        self.players.iter().map(PlayerRecord::player_name).collect()
    }

    /// Finds a player by slot ID.
    #[must_use]
    pub fn get_by_slot(&self, slot_id: u8) -> Option<&PlayerRecord> {
        self.players.iter().find(|p| p.slot_id() == slot_id)
    }

    /// Returns the offset where `TimeFrame` records or other records begin.
    #[must_use]
    pub fn end_offset(&self) -> usize {
        self.byte_length
    }
}

/// Finds the end of extended metadata section (build 10100+).
///
/// Extended metadata consists of 0x38 records and additional player info.
/// This function scans for the first real TimeFrame marker (0x1F or 0x1E)
/// with reasonable values to skip over the metadata section.
fn find_extended_metadata_end(data: &[u8], start: usize) -> usize {
    // If we're at a TimeFrame marker immediately, no extended metadata
    if start < data.len() {
        let byte = data[start];
        if byte == 0x1F || byte == 0x1E {
            // Verify it's a real TimeFrame by checking values
            if start + 5 <= data.len() {
                let time_delta = u16::from_le_bytes([data[start + 1], data[start + 2]]);
                let length_hint = u16::from_le_bytes([data[start + 3], data[start + 4]]);
                // Reasonable TimeFrame values: time < 5000ms, length < 8000
                if time_delta < 5000 && length_hint < 8000 {
                    return start;
                }
            }
        }
    }

    // Scan for the first real TimeFrame marker
    for i in start..data.len() {
        let byte = data[i];
        if byte == 0x1F || byte == 0x1E {
            // Verify it looks like a real TimeFrame
            if i + 5 <= data.len() {
                let time_delta = u16::from_le_bytes([data[i + 1], data[i + 2]]);
                let length_hint = u16::from_le_bytes([data[i + 3], data[i + 4]]);
                // Reasonable TimeFrame values
                if time_delta < 5000 && length_hint < 8000 {
                    return i;
                }
            }
        }
    }

    // If no TimeFrame found, return current position
    start
}

/// Finds the end of trailing data for a player record.
///
/// In W3G format, trailing data is typically 7 bytes after the null-terminated name:
/// - 1 byte flags (0x01 or 0x02)
/// - 6 bytes padding (usually 0x00)
///
/// However, in build 10100+, there's extended metadata (0x38 markers) that can
/// appear immediately after player records. We limit the scan to avoid absorbing
/// this metadata into the player record.
fn find_trailing_data_end(data: &[u8], start: usize) -> usize {
    // Typical trailing data is 7 bytes (flags + 6 padding bytes)
    // Limit scan to 20 bytes max to avoid absorbing extended metadata
    let max_scan = 20;
    let scan_end = (start + max_scan).min(data.len());

    for i in start..scan_end {
        let byte = data[i];
        match byte {
            // Standard record markers
            PLAYER_SLOT_MARKER | SLOT_RECORD_MARKER | 0x1E | 0x1F | 0x20 | 0x22 | 0x17 => {
                return i;
            }
            // Extended metadata marker (build 10100+)
            // 0x38 starts extended player metadata records
            0x38 => {
                return i;
            }
            _ => {}
        }
    }

    // If no marker found within scan limit, use typical 7-byte trailing data
    // This handles cases where the next marker is far away
    let typical_end = start + 7;
    if typical_end <= data.len() {
        typical_end
    } else {
        data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_slot_parse_basic() {
        // Construct a player slot record
        let mut data = Vec::new();

        // Marker
        data.push(PLAYER_SLOT_MARKER);

        // Slot ID: 4
        data.push(0x04);

        // Player name: "GreenField" + null
        data.extend_from_slice(b"GreenField\x00");

        // Trailing data (6 bytes)
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);

        // Next record marker to end
        data.push(PLAYER_SLOT_MARKER);
        data.push(0x07);

        let slot = PlayerSlot::parse(&data).unwrap();

        assert_eq!(slot.slot_id, 4);
        assert_eq!(slot.player_name, "GreenField");
        assert_eq!(slot.trailing_data, vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert!(slot.is_valid());
    }

    #[test]
    fn test_player_slot_invalid_marker() {
        let data = [0x17, 0x04, b'T', b'e', b's', b't', 0x00];
        let result = PlayerSlot::parse(&data);
        assert!(matches!(result, Err(ParserError::InvalidHeader { .. })));
    }

    #[test]
    fn test_player_roster_parse_multiple() {
        let mut data = Vec::new();

        // First player: Slot 4, "GreenField"
        data.push(PLAYER_SLOT_MARKER);
        data.push(0x04);
        data.extend_from_slice(b"GreenField\x00");
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);

        // Second player: Slot 7, "B2W.Lee"
        data.push(PLAYER_SLOT_MARKER);
        data.push(0x07);
        data.extend_from_slice(b"B2W.Lee\x00");
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);

        // Third player: Slot 8, "Slash-"
        data.push(PLAYER_SLOT_MARKER);
        data.push(0x08);
        data.extend_from_slice(b"Slash-\x00");
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);

        // TimeFrame marker to end
        data.push(0x1F);
        data.push(0x00);

        let roster = PlayerRoster::parse(&data).unwrap();

        assert_eq!(roster.len(), 3);
        assert!(!roster.is_empty());

        let names = roster.player_names();
        assert!(names.contains(&"GreenField"));
        assert!(names.contains(&"B2W.Lee"));
        assert!(names.contains(&"Slash-"));

        // Test get_by_slot
        let player = roster.get_by_slot(7).unwrap();
        assert_eq!(player.player_name(), "B2W.Lee");
    }

    #[test]
    fn test_player_roster_empty() {
        // Start with a TimeFrame marker (no players)
        let data = [0x1F, 0x00, 0x00];

        let roster = PlayerRoster::parse(&data).unwrap();

        assert!(roster.is_empty());
        assert_eq!(roster.len(), 0);
        assert_eq!(roster.byte_length, 0);
    }

    #[test]
    fn test_player_roster_with_padding() {
        let mut data = Vec::new();

        // Player
        data.push(PLAYER_SLOT_MARKER);
        data.push(0x01);
        data.extend_from_slice(b"Test\x00");
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);

        // Some padding bytes
        data.push(0x00);
        data.push(0x00);

        // TimeFrame marker
        data.push(0x1F);

        let roster = PlayerRoster::parse(&data).unwrap();

        assert_eq!(roster.len(), 1);
        // Should include padding bytes in byte_length
        assert!(roster.byte_length > 0);
    }

    #[test]
    fn test_slot_record_parse() {
        let mut data = Vec::new();

        // Marker
        data.push(SLOT_RECORD_MARKER);

        // Slot ID
        data.push(0x05);

        // Player name
        data.extend_from_slice(b"AltPlayer\x00");

        // Additional data
        data.extend_from_slice(&[0x01, 0x02, 0x03]);

        // Next marker
        data.push(0x1F);

        let record = SlotRecord::parse(&data).unwrap();

        assert_eq!(record.slot_id, 5);
        assert_eq!(record.player_name, "AltPlayer");
        assert_eq!(record.additional_data, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_player_record_unified_interface() {
        let slot = PlayerSlot {
            slot_id: 3,
            player_name: "TestPlayer".to_string(),
            trailing_data: vec![0x01, 0x00],
            byte_length: 15,
        };

        let record = PlayerRecord::PlayerSlot(slot);

        assert_eq!(record.slot_id(), 3);
        assert_eq!(record.player_name(), "TestPlayer");
        assert_eq!(record.byte_length(), 15);
    }

    #[test]
    fn test_player_slot_battlenet_name() {
        let mut data = Vec::new();

        data.push(PLAYER_SLOT_MARKER);
        data.push(0x02);
        data.extend_from_slice(b"Liqs#21977\x00");
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);
        data.push(0x1F);

        let slot = PlayerSlot::parse(&data).unwrap();

        assert_eq!(slot.player_name, "Liqs#21977");
        assert!(slot.is_valid());
    }
}
