//! Decompressed data record parsing for W3G replay files.
//!
//! This module provides parsers for the various record types found in decompressed
//! W3G replay data:
//!
//! - **Game Record Header**: Initial record with host player info and game settings
//! - **Player Slot Records**: Player names and slot assignments
//! - **`TimeFrame` Records**: Game actions with timestamps
//! - **Checksum Records**: Game state verification
//! - **Chat Messages**: In-game chat
//!
//! # Record Structure Overview
//!
//! After decompression, W3G replay data follows this structure:
//!
//! 1. **Game Record Header** (starts at offset 0)
//!    - Record type: 0x10 0x01 0x00 0x00
//!    - Host player information
//!    - Encoded game settings
//!
//! 2. **Player Slot Records** (0x16 marker)
//!    - One record per player in the game
//!    - Contains slot ID and player name
//!
//! 3. **Action Stream**
//!    - `TimeFrame` records (0x1F or 0x1E markers) with time deltas and action data
//!    - Checksum records (0x22 0x04) for state verification
//!    - Chat messages (0x20) and leave records (0x17)
//!
//! # Example
//!
//! ```ignore
//! use w3g_parser::header::Header;
//! use w3g_parser::decompress::decompress;
//! use w3g_parser::records::{GameRecord, TimeFrameIterator};
//!
//! // Parse header and decompress
//! let data = std::fs::read("replay.w3g")?;
//! let header = Header::parse(&data)?;
//! let decompressed = decompress(&data, &header)?;
//!
//! // Parse game record (header + players)
//! let game_record = GameRecord::parse(&decompressed)?;
//! println!("Host: {}", game_record.header.host_name);
//!
//! for name in game_record.players.player_names() {
//!     println!("Player: {}", name);
//! }
//!
//! // Iterate over TimeFrames
//! let iter = game_record.timeframes(&decompressed);
//! for result in iter {
//!     let frame = result?;
//!     println!("Time: {}ms, Actions: {} bytes",
//!              frame.accumulated_time_ms,
//!              frame.action_data.len());
//! }
//! ```

pub mod game_header;
pub mod player;
pub mod timeframe;

pub use game_header::{GameRecordHeader, GAME_RECORD_MAGIC};
pub use player::{
    PlayerRecord, PlayerRoster, PlayerSlot, SlotRecord, PLAYER_SLOT_MARKER, SLOT_RECORD_MARKER,
};
pub use timeframe::{
    find_timeframe_start, ChatMessage, ChecksumRecord, LeaveRecord, TimeFrame, TimeFrameIterator,
    TimeFrameStats, CHAT_MARKER, CHECKSUM_MARKER, LEAVE_MARKER, TIMEFRAME_MARKER_1E,
    TIMEFRAME_MARKER_1F,
};

use crate::error::{ParserError, Result};

/// Magic bytes for the game record header (0x10 0x01 0x00 0x00).
pub const GAME_RECORD_MAGIC_BYTES: &[u8; 4] = &[0x10, 0x01, 0x00, 0x00];

/// Finds the start of the game record in decompressed data.
///
/// For Classic replays, the game record starts at offset 0.
/// For GRBN replays, there's protobuf metadata first, followed by the game record.
///
/// This function searches for the game record magic bytes (0x10 0x01 0x00 0x00).
///
/// # Errors
///
/// Returns an error if no game record header is found.
pub fn find_game_record_start(data: &[u8]) -> Result<usize> {
    if data.len() < 4 {
        return Err(ParserError::unexpected_eof(4, data.len()));
    }

    // Check if data starts with the game record magic
    if &data[..4] == GAME_RECORD_MAGIC_BYTES {
        return Ok(0);
    }

    // Search for the magic bytes in the data
    for i in 0..data.len().saturating_sub(4) {
        if &data[i..i + 4] == GAME_RECORD_MAGIC_BYTES {
            // Verify this looks like a valid header location
            // The byte after the magic should be reasonable (usually 0x00)
            if i + 5 < data.len() {
                return Ok(i);
            }
        }
    }

    Err(ParserError::InvalidHeader {
        reason: "Game record magic bytes (0x10 0x01 0x00 0x00) not found in data".to_string(),
    })
}

/// Parsed game record containing all extracted metadata from decompressed replay data.
///
/// This struct provides a unified view of the game record header, player roster,
/// and access to the `TimeFrame` action stream.
#[derive(Debug, Clone)]
pub struct GameRecord {
    /// Game record header with host player info.
    pub header: GameRecordHeader,

    /// All players in the game.
    pub players: PlayerRoster,

    /// Offset where `TimeFrame` records begin.
    pub timeframe_offset: usize,
}

impl GameRecord {
    /// Parses a game record from decompressed replay data.
    ///
    /// This function chains the parsing of:
    /// 1. Game record header
    /// 2. Player roster
    ///
    /// And calculates the offset where `TimeFrame` records begin.
    ///
    /// For GRBN replays, the decompressed data starts with protobuf metadata
    /// followed by the Classic game record. This function searches for the
    /// game record magic bytes (0x10 0x01 0x00 0x00) to handle both cases.
    ///
    /// # Arguments
    ///
    /// * `data` - The decompressed replay data
    ///
    /// # Returns
    ///
    /// A `GameRecord` containing the parsed metadata.
    ///
    /// # Errors
    ///
    /// - `ParserError::InvalidHeader` if the game record header is malformed
    /// - `ParserError::UnexpectedEof` if the data is truncated
    pub fn parse(data: &[u8]) -> Result<Self> {
        // Find the game record start offset
        // For Classic replays, this is at offset 0
        // For GRBN replays, we need to search for the magic bytes
        let start_offset = find_game_record_start(data)?;
        let game_data = &data[start_offset..];

        // Parse game record header
        let header = GameRecordHeader::parse(game_data)?;
        let header_end = header.byte_length;

        // Parse player roster (starts after game header)
        let players = PlayerRoster::parse(&game_data[header_end..])?;
        let players_end = header_end + players.byte_length;

        // Find where TimeFrames actually begin
        // The TimeFrame offset is after players, but there may be additional
        // padding or game start records
        let timeframe_offset = find_timeframe_start(game_data, players_end).unwrap_or(game_data.len());

        // Adjust offset to be relative to original data
        let timeframe_offset = start_offset + timeframe_offset;

        Ok(GameRecord {
            header,
            players,
            timeframe_offset,
        })
    }

    /// Creates a `TimeFrame` iterator over the action stream.
    ///
    /// # Arguments
    ///
    /// * `data` - The same decompressed data used to parse this `GameRecord`
    ///
    /// # Returns
    ///
    /// A `TimeFrameIterator` starting at the `TimeFrame` offset.
    #[must_use]
    pub fn timeframes<'a>(&self, data: &'a [u8]) -> TimeFrameIterator<'a> {
        TimeFrameIterator::new(data, self.timeframe_offset)
    }

    /// Returns the host player's name.
    #[must_use]
    pub fn host_name(&self) -> &str {
        &self.header.host_name
    }

    /// Returns the host player's slot ID.
    #[must_use]
    pub fn host_slot(&self) -> u8 {
        self.header.host_slot
    }

    /// Returns all player names (including host from player roster).
    #[must_use]
    pub fn player_names(&self) -> Vec<&str> {
        self.players.player_names()
    }

    /// Returns the number of players in the game.
    #[must_use]
    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    /// Returns whether the game record appears valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.header.is_valid() && !self.players.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a complete test data blob with header, players, and timeframes.
    fn create_test_data() -> Vec<u8> {
        let mut data = Vec::new();

        // Game record header
        data.extend_from_slice(&[0x10, 0x01, 0x00, 0x00]); // Record type
        data.push(0x00); // Unknown
        data.push(0x03); // Host slot 3
        data.extend_from_slice(b"HostPlayer\x00"); // Host name
        data.push(0x01); // Flags
        data.extend_from_slice(b"\x00"); // Empty additional data
        // No encoded settings before player records

        // Player slot records
        data.push(PLAYER_SLOT_MARKER);
        data.push(0x04);
        data.extend_from_slice(b"Player1\x00");
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);

        data.push(PLAYER_SLOT_MARKER);
        data.push(0x05);
        data.extend_from_slice(b"Player2\x00");
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);

        // TimeFrame
        data.push(TIMEFRAME_MARKER_1F);
        data.extend_from_slice(&[0x64, 0x00]); // 100ms
        data.extend_from_slice(&[0x00, 0x00]); // Length hint
        data.push(CHECKSUM_MARKER);
        data.extend_from_slice(&[0x04, 0x00, 0x00, 0x00, 0x00]);

        // Another TimeFrame
        data.push(TIMEFRAME_MARKER_1F);
        data.extend_from_slice(&[0x32, 0x00]); // 50ms
        data.extend_from_slice(&[0x00, 0x00]);

        data
    }

    #[test]
    fn test_game_record_parse() {
        let data = create_test_data();
        let record = GameRecord::parse(&data).unwrap();

        assert_eq!(record.host_name(), "HostPlayer");
        assert_eq!(record.host_slot(), 3);
        assert!(record.is_valid());
    }

    #[test]
    fn test_game_record_players() {
        let data = create_test_data();
        let record = GameRecord::parse(&data).unwrap();

        assert_eq!(record.player_count(), 2);

        let names = record.player_names();
        assert!(names.contains(&"Player1"));
        assert!(names.contains(&"Player2"));
    }

    #[test]
    fn test_game_record_timeframes() {
        let data = create_test_data();
        let record = GameRecord::parse(&data).unwrap();

        let iter = record.timeframes(&data);
        let frames: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();

        assert!(!frames.is_empty());
        // First frame should have 100ms time delta
        if !frames.is_empty() {
            assert_eq!(frames[0].time_delta_ms, 100);
        }
    }

    #[test]
    fn test_game_record_empty_players() {
        // Create data with header but no player slots (immediate TimeFrame)
        let mut data = Vec::new();

        // Game record header
        data.extend_from_slice(&[0x10, 0x01, 0x00, 0x00]);
        data.push(0x00);
        data.push(0x01);
        data.extend_from_slice(b"Host\x00");
        data.push(0x01);
        data.extend_from_slice(b"\x00");

        // Immediately TimeFrame (no player slots)
        data.push(TIMEFRAME_MARKER_1F);
        data.extend_from_slice(&[0x10, 0x00, 0x00, 0x00]);

        let record = GameRecord::parse(&data).unwrap();

        assert_eq!(record.host_name(), "Host");
        assert_eq!(record.player_count(), 0);
        // Still valid because we have a host
        assert!(record.header.is_valid());
    }
}
