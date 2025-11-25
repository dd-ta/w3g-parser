//! Selection action parsing (0x16).
//!
//! Selection actions specify which units are currently selected,
//! providing context for subsequent ability and movement commands.

use crate::error::{ParserError, Result};

/// A unit selection action.
///
/// Selection actions specify which units are currently selected,
/// used as context for subsequent ability and movement commands.
///
/// # Format
///
/// ```text
/// 16 [count: 1] [mode: 1] [flags: 1] [unit_ids: 8*count]
/// ```
///
/// Each unit uses 8 bytes (4-byte ID appearing twice or ID + counter).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionAction {
    /// Number of units selected (1-12 typical).
    pub unit_count: u8,

    /// Selection mode (1-10 observed).
    /// - Mode 1: Most common, single selection
    /// - Mode 5: Multi-unit selection
    /// - Other modes: Under investigation
    pub mode: u8,

    /// Flags byte (usually 0x00).
    pub flags: u8,

    /// Object IDs of selected units.
    /// Each ID is 4 bytes, originally stored in 8-byte blocks.
    pub unit_ids: Vec<u32>,
}

impl SelectionAction {
    /// Marker byte for selection actions.
    pub const MARKER: u8 = 0x16;

    /// Parses a selection action from raw action data.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw action data starting at the 0x16 marker
    ///
    /// # Returns
    ///
    /// A tuple of `(SelectionAction, bytes_consumed)`.
    ///
    /// # Errors
    ///
    /// - `ParserError::InvalidHeader` if the marker byte is not 0x16
    /// - `ParserError::UnexpectedEof` if the data is truncated
    pub fn parse(data: &[u8]) -> Result<(Self, usize)> {
        if data.is_empty() {
            return Err(ParserError::unexpected_eof(1, 0));
        }

        if data[0] != Self::MARKER {
            return Err(ParserError::InvalidHeader {
                reason: format!(
                    "Invalid selection marker: expected 0x{:02X}, found 0x{:02X}",
                    Self::MARKER,
                    data[0]
                ),
            });
        }

        // Minimum size: marker + count + mode + flags = 4 bytes
        if data.len() < 4 {
            return Err(ParserError::unexpected_eof(4, data.len()));
        }

        let unit_count = data[1];
        let mode = data[2];
        let flags = data[3];

        // Calculate expected data size: 4 header bytes + 8 bytes per unit
        let expected_size = 4 + (unit_count as usize) * 8;
        if data.len() < expected_size {
            return Err(ParserError::unexpected_eof(expected_size, data.len()));
        }

        // Extract unit IDs from 8-byte blocks
        let unit_ids = extract_unit_ids(&data[4..], unit_count);

        Ok((
            SelectionAction {
                unit_count,
                mode,
                flags,
                unit_ids,
            },
            expected_size,
        ))
    }

    /// Returns the selection mode as an enum.
    #[must_use]
    pub fn selection_mode(&self) -> SelectionMode {
        SelectionMode::from(self.mode)
    }

    /// Returns whether this is a multi-unit selection.
    #[must_use]
    pub fn is_multi_select(&self) -> bool {
        self.unit_count > 1
    }
}

/// Extracts unit IDs from 8-byte blocks.
///
/// Each unit is stored as an 8-byte block. We extract the first 4 bytes
/// as the unit ID (the second 4 bytes may be a duplicate or counter).
fn extract_unit_ids(data: &[u8], count: u8) -> Vec<u32> {
    let mut ids = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let offset = i * 8;
        if offset + 4 <= data.len() {
            let id = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            ids.push(id);
        }
    }
    ids
}

/// Selection mode interpretation.
///
/// Different modes represent different selection behaviors,
/// though the exact semantics are still under investigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// Replace current selection entirely (mode 1).
    Replace,
    /// Add to current selection (hypothesized).
    Add,
    /// Toggle selection (hypothesized).
    Toggle,
    /// Unknown mode.
    Unknown(u8),
}

impl From<u8> for SelectionMode {
    fn from(value: u8) -> Self {
        match value {
            1 => SelectionMode::Replace,
            2 => SelectionMode::Add,
            3 => SelectionMode::Toggle,
            n => SelectionMode::Unknown(n),
        }
    }
}

impl SelectionMode {
    /// Returns the raw mode byte.
    #[must_use]
    pub fn as_byte(&self) -> u8 {
        match self {
            SelectionMode::Replace => 1,
            SelectionMode::Add => 2,
            SelectionMode::Toggle => 3,
            SelectionMode::Unknown(n) => *n,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_single_unit() {
        // From Rex's analysis: single unit selection
        let data: &[u8] = &[
            0x16, 0x01, 0x01, 0x00, // Selection: 1 unit, mode 1, flags 0
            0x3B, 0x3A, 0x00, 0x00, 0x3B, 0x3A, 0x00, 0x00, // Unit ID 0x00003A3B
        ];

        let (sel, consumed) = SelectionAction::parse(data).unwrap();

        assert_eq!(sel.unit_count, 1);
        assert_eq!(sel.mode, 1);
        assert_eq!(sel.flags, 0);
        assert_eq!(sel.unit_ids, vec![0x0000_3A3B]);
        assert_eq!(consumed, 12);
        assert!(!sel.is_multi_select());
        assert_eq!(sel.selection_mode(), SelectionMode::Replace);
    }

    #[test]
    fn test_selection_multiple_units() {
        // From Rex's analysis: multi-unit selection
        let data: &[u8] = &[
            0x16, 0x02, 0x05, 0x00, // Selection: 2 units, mode 5
            0x23, 0x3B, 0x00, 0x00, 0x26, 0x3B, 0x00, 0x00, // Unit 1
            0x39, 0x3B, 0x00, 0x00, 0x3C, 0x3B, 0x00, 0x00, // Unit 2
        ];

        let (sel, consumed) = SelectionAction::parse(data).unwrap();

        assert_eq!(sel.unit_count, 2);
        assert_eq!(sel.mode, 5);
        assert_eq!(sel.unit_ids.len(), 2);
        assert_eq!(sel.unit_ids[0], 0x0000_3B23);
        assert_eq!(sel.unit_ids[1], 0x0000_3B39);
        assert_eq!(consumed, 20);
        assert!(sel.is_multi_select());
    }

    #[test]
    fn test_selection_invalid_marker() {
        let data: &[u8] = &[0x15, 0x01, 0x01, 0x00]; // Wrong marker
        let result = SelectionAction::parse(data);
        assert!(matches!(result, Err(ParserError::InvalidHeader { .. })));
    }

    #[test]
    fn test_selection_truncated() {
        let data: &[u8] = &[0x16, 0x01, 0x01]; // Missing flags byte
        let result = SelectionAction::parse(data);
        assert!(matches!(result, Err(ParserError::UnexpectedEof { .. })));
    }

    #[test]
    fn test_selection_mode_conversion() {
        assert_eq!(SelectionMode::from(1), SelectionMode::Replace);
        assert_eq!(SelectionMode::from(2), SelectionMode::Add);
        assert_eq!(SelectionMode::from(3), SelectionMode::Toggle);
        assert_eq!(SelectionMode::from(5), SelectionMode::Unknown(5));

        assert_eq!(SelectionMode::Replace.as_byte(), 1);
        assert_eq!(SelectionMode::Unknown(10).as_byte(), 10);
    }
}
