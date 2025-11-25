//! `TimeFrame` record parsing and iteration for decompressed W3G replay data.
//!
//! This module provides parsing for `TimeFrame` records (0x1F marker) and an iterator
//! for sequential access to game action frames with accumulated timestamps.
//!
//! # Format
//!
//! `TimeFrame` records have the following structure:
//!
//! | Offset | Size | Type | Field |
//! |--------|------|------|-------|
//! | 0 | 1 | u8 | Record type (0x1F or 0x1E) |
//! | 1 | 2 | u16 LE | Time increment (milliseconds) |
//! | 3 | 2 | u16 LE | Action data length hint |
//! | 5 | var | bytes | Action data |
//!
//! # Example
//!
//! ```ignore
//! use w3g_parser::records::TimeFrameIterator;
//!
//! let decompressed_data = /* ... */;
//! let start_offset = /* offset after player records */;
//!
//! let mut total_time = 0u32;
//! for result in TimeFrameIterator::new(&decompressed_data, start_offset) {
//!     let frame = result?;
//!     total_time = frame.accumulated_time_ms;
//!     println!("Time: {}ms, Actions: {} bytes", frame.time_delta_ms, frame.action_data.len());
//! }
//! println!("Total game time: {}ms", total_time);
//! ```

use crate::actions::{ActionContext, ActionIterator};
use crate::binary::read_u16_le;
use crate::error::{ParserError, Result};

/// `TimeFrame` record marker (primary).
pub const TIMEFRAME_MARKER_1F: u8 = 0x1F;

/// `TimeFrame` record marker (alternate, possibly for different game modes).
pub const TIMEFRAME_MARKER_1E: u8 = 0x1E;

/// Checksum record marker.
pub const CHECKSUM_MARKER: u8 = 0x22;

/// Chat message record marker.
pub const CHAT_MARKER: u8 = 0x20;

/// Leave game marker.
pub const LEAVE_MARKER: u8 = 0x17;

/// A single `TimeFrame` record containing game actions.
///
/// `TimeFrame` records represent a slice of game time and contain all player actions
/// that occurred during that period. The time delta represents milliseconds
/// since the previous `TimeFrame`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeFrame {
    /// Time increment in milliseconds since the previous `TimeFrame`.
    pub time_delta_ms: u16,

    /// Raw action data (parsing deferred to Phase 5).
    pub action_data: Vec<u8>,

    /// Accumulated game time in milliseconds from game start.
    pub accumulated_time_ms: u32,
}

impl TimeFrame {
    /// Returns whether this `TimeFrame` contains any action data.
    #[must_use]
    pub fn has_actions(&self) -> bool {
        !self.action_data.is_empty()
    }

    /// Returns whether this is an empty `TimeFrame` (no actions).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.action_data.is_empty()
    }

    /// Returns the number of bytes of action data.
    #[must_use]
    pub fn action_len(&self) -> usize {
        self.action_data.len()
    }

    /// Returns an iterator over parsed actions in this `TimeFrame`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// for action_result in frame.actions() {
    ///     match action_result {
    ///         Ok(action) => println!("Player {}: {:?}", action.player_id, action.action_type),
    ///         Err(e) => eprintln!("Parse error: {}", e),
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn actions(&self) -> ActionIterator<'_> {
        let ctx = ActionContext::new(self.accumulated_time_ms, 0);
        ActionIterator::new(&self.action_data, ctx)
    }
}

/// A checksum record that follows `TimeFrame` records.
///
/// Checksum records (0x22 0x04) contain game state verification data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChecksumRecord {
    /// Checksum type (usually 0x04).
    pub checksum_type: u8,

    /// Checksum value (4 bytes).
    pub checksum: u32,
}

impl ChecksumRecord {
    /// Size of a checksum record in bytes.
    pub const SIZE: usize = 6;

    /// Parses a checksum record from decompressed replay data.
    ///
    /// # Arguments
    ///
    /// * `data` - The decompressed replay data starting at a 0x22 marker
    ///
    /// # Returns
    ///
    /// A `ChecksumRecord` containing the parsed fields.
    ///
    /// # Errors
    ///
    /// - `ParserError::InvalidHeader` if the marker byte is not 0x22
    /// - `ParserError::UnexpectedEof` if the data is truncated
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < Self::SIZE {
            return Err(ParserError::unexpected_eof(Self::SIZE, data.len()));
        }

        if data[0] != CHECKSUM_MARKER {
            return Err(ParserError::InvalidHeader {
                reason: format!(
                    "Invalid checksum marker: expected 0x{CHECKSUM_MARKER:02X}, found 0x{:02X}",
                    data[0]
                ),
            });
        }

        let checksum_type = data[1];
        let checksum = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);

        Ok(ChecksumRecord {
            checksum_type,
            checksum,
        })
    }
}

/// A chat message record.
///
/// Chat messages (0x20 marker) contain in-game messages from players.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    /// Flags byte.
    pub flags: u8,

    /// Message ID or type.
    pub message_id: u16,

    /// The message content.
    pub message: String,

    /// Total bytes consumed by this record.
    pub byte_length: usize,
}

impl ChatMessage {
    /// Parses a chat message record from decompressed replay data.
    ///
    /// # Arguments
    ///
    /// * `data` - The decompressed replay data starting at a 0x20 marker
    ///
    /// # Returns
    ///
    /// A `ChatMessage` containing the parsed fields.
    ///
    /// # Errors
    ///
    /// - `ParserError::InvalidHeader` if the marker byte is not 0x20
    /// - `ParserError::UnexpectedEof` if the data is truncated
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 5 {
            return Err(ParserError::unexpected_eof(5, data.len()));
        }

        if data[0] != CHAT_MARKER {
            return Err(ParserError::InvalidHeader {
                reason: format!(
                    "Invalid chat marker: expected 0x{CHAT_MARKER:02X}, found 0x{:02X}",
                    data[0]
                ),
            });
        }

        let flags = data[1];
        let message_id = read_u16_le(data, 2)?;

        // Chat message format (reverse engineered from replay data):
        // - Offset 0: 0x20 (marker)
        // - Offset 1: flags (0x03 for system, 0x07 for player)
        // - Offset 2-3: message_id (u16 little-endian)
        // - Offset 4-8: padding (0x20 XX 0x00 0x00 0x00)
        // - Offset 9+: null-terminated message
        //
        // The message always starts at offset 9 (after the 5-byte padding block)
        let msg_start = 9;

        // Find the message end (null terminator)
        let mut msg_end = msg_start;
        while msg_end < data.len() && data[msg_end] != 0 {
            msg_end += 1;
        }

        let message = if msg_end > msg_start {
            String::from_utf8_lossy(&data[msg_start..msg_end]).to_string()
        } else {
            String::new()
        };

        // Include the null terminator in byte_length
        let byte_length = if msg_end < data.len() {
            msg_end + 1
        } else {
            msg_end
        };

        Ok(ChatMessage {
            flags,
            message_id,
            message,
            byte_length,
        })
    }
}

/// A leave game record.
///
/// Leave game records (0x17 marker) indicate a player leaving the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaveRecord {
    /// Reason code for leaving.
    pub reason: u32,

    /// Player ID who left.
    pub player_id: u8,

    /// Result code.
    pub result: u32,

    /// Unknown field.
    pub unknown: u32,
}

impl LeaveRecord {
    /// Typical size of a leave record.
    pub const SIZE: usize = 14;

    /// Parses a leave record from decompressed replay data.
    ///
    /// # Errors
    ///
    /// - `ParserError::InvalidHeader` if the marker byte is not 0x17
    /// - `ParserError::UnexpectedEof` if the data is truncated
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < Self::SIZE {
            return Err(ParserError::unexpected_eof(Self::SIZE, data.len()));
        }

        if data[0] != LEAVE_MARKER {
            return Err(ParserError::InvalidHeader {
                reason: format!(
                    "Invalid leave marker: expected 0x{LEAVE_MARKER:02X}, found 0x{:02X}",
                    data[0]
                ),
            });
        }

        let reason = u32::from_le_bytes([data[1], data[2], data[3], data[4]]);
        let player_id = data[5];
        let result = u32::from_le_bytes([data[6], data[7], data[8], data[9]]);
        let unknown = u32::from_le_bytes([data[10], data[11], data[12], data[13]]);

        Ok(LeaveRecord {
            reason,
            player_id,
            result,
            unknown,
        })
    }
}

/// Iterator over `TimeFrame` records in decompressed replay data.
///
/// This iterator yields `TimeFrame` records sequentially, handling the interleaved
/// checksum records, chat messages, and leave records automatically.
///
/// # Example
///
/// ```ignore
/// use w3g_parser::records::TimeFrameIterator;
///
/// let iter = TimeFrameIterator::new(&decompressed_data, start_offset);
/// for result in iter {
///     let frame = result?;
///     println!("Time: {}ms", frame.accumulated_time_ms);
/// }
/// ```
pub struct TimeFrameIterator<'a> {
    /// Reference to the decompressed data.
    data: &'a [u8],

    /// Current position in the data.
    offset: usize,

    /// Accumulated game time in milliseconds.
    accumulated_time: u32,

    /// Number of `TimeFrame` records yielded so far.
    frame_count: usize,

    /// Whether iteration has completed.
    finished: bool,
}

impl<'a> TimeFrameIterator<'a> {
    /// Creates a new `TimeFrame` iterator.
    ///
    /// # Arguments
    ///
    /// * `data` - The decompressed replay data
    /// * `start_offset` - The byte offset where `TimeFrame` records begin
    ///
    /// # Returns
    ///
    /// A new `TimeFrameIterator` starting at the given offset.
    #[must_use]
    pub fn new(data: &'a [u8], start_offset: usize) -> Self {
        TimeFrameIterator {
            data,
            offset: start_offset,
            accumulated_time: 0,
            frame_count: 0,
            finished: false,
        }
    }

    /// Returns the current accumulated game time in milliseconds.
    #[must_use]
    pub fn accumulated_time_ms(&self) -> u32 {
        self.accumulated_time
    }

    /// Returns the number of `TimeFrame` records yielded so far.
    #[must_use]
    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    /// Returns the current byte offset in the data.
    #[must_use]
    pub fn current_offset(&self) -> usize {
        self.offset
    }

    /// Returns whether iteration is complete.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Skips over non-TimeFrame records (checksums, chat messages, leave records).
    ///
    /// Returns `true` if we should continue iteration, `false` if we're done.
    fn skip_non_timeframe_records(&mut self) -> bool {
        // Maximum bytes to scan before giving up (avoid infinite loops)
        let scan_limit = 10000;
        let mut bytes_scanned = 0;

        loop {
            if self.offset >= self.data.len() {
                self.finished = true;
                return false;
            }

            if bytes_scanned > scan_limit {
                // Give up after scanning too many bytes without finding a TimeFrame
                self.finished = true;
                return false;
            }

            match self.data[self.offset] {
                CHECKSUM_MARKER => {
                    // Skip checksum record (6 bytes)
                    self.offset += ChecksumRecord::SIZE;
                    bytes_scanned += ChecksumRecord::SIZE;
                }
                CHAT_MARKER => {
                    // Parse and skip chat message
                    if let Ok(msg) = ChatMessage::parse(&self.data[self.offset..]) {
                        self.offset += msg.byte_length;
                        bytes_scanned += msg.byte_length;
                    } else {
                        // If parsing fails, skip 1 byte and continue
                        self.offset += 1;
                        bytes_scanned += 1;
                    }
                }
                LEAVE_MARKER => {
                    // Skip leave record (14 bytes)
                    self.offset += LeaveRecord::SIZE;
                    bytes_scanned += LeaveRecord::SIZE;
                }
                TIMEFRAME_MARKER_1E | TIMEFRAME_MARKER_1F => {
                    // Verify this looks like a real TimeFrame (not just a random byte)
                    if self.is_valid_timeframe_at(self.offset) {
                        return true;
                    }
                    // Not a valid TimeFrame, skip this byte
                    self.offset += 1;
                    bytes_scanned += 1;
                }
                0x00 => {
                    // Padding byte, skip it
                    self.offset += 1;
                    bytes_scanned += 1;
                }
                _ => {
                    // Unknown marker - try to find the next TimeFrame marker
                    // Build 10100+ may have additional record types we don't know
                    if let Some(next_tf) = self.find_next_timeframe() {
                        self.offset = next_tf;
                        bytes_scanned += next_tf - self.offset;
                    } else {
                        self.finished = true;
                        return false;
                    }
                }
            }
        }
    }

    /// Checks if the byte at the given offset is a valid TimeFrame start.
    fn is_valid_timeframe_at(&self, offset: usize) -> bool {
        if offset + 5 > self.data.len() {
            return false;
        }

        let marker = self.data[offset];
        if marker != TIMEFRAME_MARKER_1E && marker != TIMEFRAME_MARKER_1F {
            return false;
        }

        // Check for reasonable time delta (< 5000ms) and length hint (< 8000)
        let time_delta = u16::from_le_bytes([self.data[offset + 1], self.data[offset + 2]]);
        let length_hint = u16::from_le_bytes([self.data[offset + 3], self.data[offset + 4]]);

        time_delta < 5000 && length_hint < 8000
    }

    /// Finds the next valid TimeFrame marker from current offset.
    fn find_next_timeframe(&self) -> Option<usize> {
        // Limit search to avoid scanning entire file
        let search_limit = 1000;
        let end = (self.offset + search_limit).min(self.data.len());

        for i in (self.offset + 1)..end {
            if self.is_valid_timeframe_at(i) {
                return Some(i);
            }
        }
        None
    }

    /// Parses a single `TimeFrame` record at the current offset.
    fn parse_timeframe(&mut self) -> Result<TimeFrame> {
        let data = &self.data[self.offset..];

        if data.len() < 5 {
            return Err(ParserError::unexpected_eof(5, data.len()));
        }

        let marker = data[0];
        if marker != TIMEFRAME_MARKER_1E && marker != TIMEFRAME_MARKER_1F {
            return Err(ParserError::InvalidHeader {
                reason: format!(
                    "Invalid TimeFrame marker: expected 0x1E or 0x1F, found 0x{marker:02X}"
                ),
            });
        }

        // Read time delta (2 bytes, little-endian)
        let time_delta_ms = read_u16_le(data, 1)?;

        // Read action data length hint (2 bytes) - but we use marker-based detection
        // The length hint is not reliable, so we scan for the next record marker
        let _length_hint = read_u16_le(data, 3)?;

        // Find the end of action data by scanning for next record marker
        let action_start = 5;
        let action_end = find_action_boundary(data, action_start);

        let action_data = if action_end > action_start {
            data[action_start..action_end].to_vec()
        } else {
            Vec::new()
        };

        // Update accumulated time
        self.accumulated_time = self.accumulated_time.saturating_add(u32::from(time_delta_ms));

        // Update offset to point past the TimeFrame
        self.offset += action_end;

        Ok(TimeFrame {
            time_delta_ms,
            action_data,
            accumulated_time_ms: self.accumulated_time,
        })
    }
}

impl Iterator for TimeFrameIterator<'_> {
    type Item = Result<TimeFrame>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        // Skip any non-TimeFrame records
        if !self.skip_non_timeframe_records() {
            return None;
        }

        // Parse the TimeFrame
        match self.parse_timeframe() {
            Ok(frame) => {
                self.frame_count += 1;
                Some(Ok(frame))
            }
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}

/// Finds the boundary where action data ends within a `TimeFrame`.
///
/// Scans for known record markers: 0x1E, 0x1F (`TimeFrame`), 0x22 (Checksum),
/// 0x20 (Chat), 0x17 (Leave).
fn find_action_boundary(data: &[u8], start: usize) -> usize {
    for (i, &byte) in data.iter().enumerate().skip(start) {
        match byte {
            TIMEFRAME_MARKER_1E
            | TIMEFRAME_MARKER_1F
            | CHECKSUM_MARKER
            | CHAT_MARKER
            | LEAVE_MARKER => return i,
            _ => {}
        }
    }

    data.len()
}

/// Finds the offset where `TimeFrame` records begin in decompressed data.
///
/// This function scans from the start offset looking for the first `TimeFrame`
/// marker (0x1F or 0x1E).
#[must_use]
pub fn find_timeframe_start(data: &[u8], start: usize) -> Option<usize> {
    (start..data.len()).find(|&i| data[i] == TIMEFRAME_MARKER_1F || data[i] == TIMEFRAME_MARKER_1E)
}

/// Summary statistics from `TimeFrame` iteration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TimeFrameStats {
    /// Total number of `TimeFrame` records.
    pub frame_count: usize,

    /// Total game time in milliseconds.
    pub total_time_ms: u32,

    /// Total bytes of action data across all frames.
    pub total_action_bytes: usize,

    /// Number of frames with no action data.
    pub empty_frame_count: usize,
}

impl TimeFrameStats {
    /// Calculates statistics from an iterator of `TimeFrame` records.
    ///
    /// This consumes the iterator and calculates summary statistics.
    ///
    /// # Errors
    ///
    /// Returns an error if any `TimeFrame` in the iterator fails to parse.
    pub fn from_iterator(iter: TimeFrameIterator<'_>) -> Result<Self> {
        let mut stats = TimeFrameStats::default();

        for result in iter {
            let frame = result?;
            stats.frame_count += 1;
            stats.total_time_ms = frame.accumulated_time_ms;
            stats.total_action_bytes += frame.action_data.len();
            if frame.is_empty() {
                stats.empty_frame_count += 1;
            }
        }

        Ok(stats)
    }

    /// Returns the average time delta per frame in milliseconds.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn average_time_delta_ms(&self) -> f64 {
        if self.frame_count == 0 {
            0.0
        } else {
            f64::from(self.total_time_ms) / self.frame_count as f64
        }
    }

    /// Returns the total game time formatted as MM:SS.
    #[must_use]
    pub fn duration_string(&self) -> String {
        let total_seconds = self.total_time_ms / 1000;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{minutes}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeframe_parse_basic() {
        let mut data = Vec::new();

        // TimeFrame marker
        data.push(TIMEFRAME_MARKER_1F);

        // Time delta: 60ms (0x003C)
        data.extend_from_slice(&[0x3C, 0x00]);

        // Length hint (ignored)
        data.extend_from_slice(&[0x64, 0x00]);

        // Some action data
        data.extend_from_slice(&[0x01, 0x1A, 0x00, 0x16, 0x01]);

        // Checksum marker to end
        data.push(CHECKSUM_MARKER);
        data.extend_from_slice(&[0x04, 0x00, 0x00, 0x00, 0x00]);

        let mut iter = TimeFrameIterator::new(&data, 0);
        let frame = iter.next().unwrap().unwrap();

        assert_eq!(frame.time_delta_ms, 60);
        assert_eq!(frame.accumulated_time_ms, 60);
        assert_eq!(frame.action_data, vec![0x01, 0x1A, 0x00, 0x16, 0x01]);
        assert!(frame.has_actions());
    }

    #[test]
    fn test_timeframe_empty_actions() {
        let mut data = Vec::new();

        // TimeFrame marker
        data.push(TIMEFRAME_MARKER_1F);

        // Time delta: 2ms
        data.extend_from_slice(&[0x02, 0x00]);

        // Length hint: 0
        data.extend_from_slice(&[0x00, 0x00]);

        // Immediately followed by checksum
        data.push(CHECKSUM_MARKER);
        data.extend_from_slice(&[0x04, 0x00, 0x00, 0x00, 0x00]);

        let mut iter = TimeFrameIterator::new(&data, 0);
        let frame = iter.next().unwrap().unwrap();

        assert_eq!(frame.time_delta_ms, 2);
        assert!(frame.is_empty());
        assert!(!frame.has_actions());
    }

    #[test]
    fn test_timeframe_iterator_multiple() {
        let mut data = Vec::new();

        // First TimeFrame
        data.push(TIMEFRAME_MARKER_1F);
        data.extend_from_slice(&[0x10, 0x00]); // 16ms
        data.extend_from_slice(&[0x00, 0x00]);
        data.push(CHECKSUM_MARKER);
        data.extend_from_slice(&[0x04, 0x00, 0x00, 0x00, 0x00]);

        // Second TimeFrame
        data.push(TIMEFRAME_MARKER_1F);
        data.extend_from_slice(&[0x20, 0x00]); // 32ms
        data.extend_from_slice(&[0x00, 0x00]);
        data.push(CHECKSUM_MARKER);
        data.extend_from_slice(&[0x04, 0x00, 0x00, 0x00, 0x00]);

        // Third TimeFrame
        data.push(TIMEFRAME_MARKER_1F);
        data.extend_from_slice(&[0x0A, 0x00]); // 10ms
        data.extend_from_slice(&[0x00, 0x00]);

        let iter = TimeFrameIterator::new(&data, 0);
        let frames: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();

        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0].time_delta_ms, 16);
        assert_eq!(frames[0].accumulated_time_ms, 16);
        assert_eq!(frames[1].time_delta_ms, 32);
        assert_eq!(frames[1].accumulated_time_ms, 48);
        assert_eq!(frames[2].time_delta_ms, 10);
        assert_eq!(frames[2].accumulated_time_ms, 58);
    }

    #[test]
    fn test_checksum_record_parse() {
        let data = [0x22, 0x04, 0x12, 0x34, 0x56, 0x78];

        let checksum = ChecksumRecord::parse(&data).unwrap();

        assert_eq!(checksum.checksum_type, 0x04);
        assert_eq!(checksum.checksum, 0x78563412);
    }

    #[test]
    fn test_checksum_record_invalid_marker() {
        let data = [0x21, 0x04, 0x00, 0x00, 0x00, 0x00];
        let result = ChecksumRecord::parse(&data);
        assert!(matches!(result, Err(ParserError::InvalidHeader { .. })));
    }

    #[test]
    fn test_find_timeframe_start() {
        let data = [0x00, 0x16, 0x04, 0x00, 0x1F, 0x02, 0x00];

        let start = find_timeframe_start(&data, 0);
        assert_eq!(start, Some(4));
    }

    #[test]
    fn test_find_timeframe_start_not_found() {
        let data = [0x00, 0x16, 0x04, 0x00, 0x22, 0x04];

        let start = find_timeframe_start(&data, 0);
        assert_eq!(start, None);
    }

    #[test]
    fn test_timeframe_stats() {
        let mut data = Vec::new();

        // Three TimeFrames with varying time deltas
        for time in &[100u16, 200u16, 150u16] {
            data.push(TIMEFRAME_MARKER_1F);
            data.extend_from_slice(&time.to_le_bytes());
            data.extend_from_slice(&[0x00, 0x00]);
            data.push(CHECKSUM_MARKER);
            data.extend_from_slice(&[0x04, 0x00, 0x00, 0x00, 0x00]);
        }

        let iter = TimeFrameIterator::new(&data, 0);
        let stats = TimeFrameStats::from_iterator(iter).unwrap();

        assert_eq!(stats.frame_count, 3);
        assert_eq!(stats.total_time_ms, 450);
        assert_eq!(stats.empty_frame_count, 3); // All empty
        assert_eq!(stats.duration_string(), "0:00");
    }

    #[test]
    fn test_timeframe_iterator_with_start_offset() {
        let mut data = Vec::new();

        // Some garbage bytes before the TimeFrames
        data.extend_from_slice(&[0x00, 0x16, 0x04, 0x00]);

        // TimeFrame at offset 4
        data.push(TIMEFRAME_MARKER_1F);
        data.extend_from_slice(&[0x64, 0x00]); // 100ms
        data.extend_from_slice(&[0x00, 0x00]);

        let iter = TimeFrameIterator::new(&data, 4);
        let frames: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();

        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].time_delta_ms, 100);
    }

    #[test]
    fn test_timeframe_1e_marker() {
        let mut data = Vec::new();

        // TimeFrame with 0x1E marker
        data.push(TIMEFRAME_MARKER_1E);
        data.extend_from_slice(&[0x32, 0x00]); // 50ms
        data.extend_from_slice(&[0x00, 0x00]);

        let mut iter = TimeFrameIterator::new(&data, 0);
        let frame = iter.next().unwrap().unwrap();

        assert_eq!(frame.time_delta_ms, 50);
    }

    #[test]
    fn test_iterator_helper_methods() {
        let data = [
            TIMEFRAME_MARKER_1F,
            0x10,
            0x00,
            0x00,
            0x00,
            CHECKSUM_MARKER,
            0x04,
            0x00,
            0x00,
            0x00,
            0x00,
        ];

        let mut iter = TimeFrameIterator::new(&data, 0);

        assert_eq!(iter.frame_count(), 0);
        assert_eq!(iter.accumulated_time_ms(), 0);
        assert!(!iter.is_finished());

        let _ = iter.next();

        assert_eq!(iter.frame_count(), 1);
        assert_eq!(iter.accumulated_time_ms(), 16);
    }
}
