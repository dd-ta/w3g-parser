//! Hotkey action parsing (0x17).
//!
//! Hotkey actions represent control group operations such as
//! assigning units to groups (Ctrl+N) or selecting groups (N).

use crate::error::{ParserError, Result};
use std::fmt;

/// A hotkey (control group) operation.
///
/// Control groups allow players to quickly select groups of units
/// using number keys (0-9).
///
/// # Format
///
/// ```text
/// 17 [group: 1] [operation: 1]
/// ```
///
/// Note: The exact format and additional data is still under investigation.
/// Some hotkey actions may include unit IDs for assign operations.
#[derive(Debug, Clone)]
pub struct HotkeyAction {
    /// Control group number (0-9).
    pub group: u8,

    /// Operation type.
    pub operation: HotkeyOperation,
}

impl HotkeyAction {
    /// Action type marker.
    pub const MARKER: u8 = 0x17;

    /// Minimum size of a hotkey action.
    pub const MIN_SIZE: usize = 3;

    /// Parses a hotkey action from raw data.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw action data starting at 0x17 marker
    ///
    /// # Returns
    ///
    /// A tuple of `(HotkeyAction, bytes_consumed)`.
    ///
    /// # Errors
    ///
    /// - `ParserError::InvalidHeader` if marker doesn't match
    /// - `ParserError::UnexpectedEof` if data is truncated
    pub fn parse(data: &[u8]) -> Result<(Self, usize)> {
        if data.len() < Self::MIN_SIZE {
            return Err(ParserError::unexpected_eof(Self::MIN_SIZE, data.len()));
        }

        if data[0] != Self::MARKER {
            return Err(ParserError::InvalidHeader {
                reason: format!(
                    "Invalid hotkey marker: expected 0x{:02X}, found 0x{:02X}",
                    Self::MARKER,
                    data[0]
                ),
            });
        }

        let group = data[1];
        let operation_byte = data[2];

        let operation = match operation_byte {
            0 => HotkeyOperation::Assign,
            1 => HotkeyOperation::Select,
            2 => HotkeyOperation::AddToGroup,
            n => HotkeyOperation::Unknown(n),
        };

        // The basic hotkey action is 3 bytes
        // TODO: Some operations may have additional data (unit IDs for assign)
        // For now, we consume only the minimum 3 bytes

        Ok((
            HotkeyAction { group, operation },
            Self::MIN_SIZE,
        ))
    }

    /// Returns whether this is an assign operation.
    #[must_use]
    pub fn is_assign(&self) -> bool {
        matches!(self.operation, HotkeyOperation::Assign)
    }

    /// Returns whether this is a select operation.
    #[must_use]
    pub fn is_select(&self) -> bool {
        matches!(self.operation, HotkeyOperation::Select)
    }

    /// Returns whether the group number is valid (0-9).
    #[must_use]
    pub fn is_valid_group(&self) -> bool {
        self.group < 10
    }
}

impl fmt::Display for HotkeyAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hotkey: {} group {}", self.operation, self.group)
    }
}

/// Type of hotkey operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyOperation {
    /// Assign current selection to group (Ctrl+N).
    Assign,
    /// Select group (press N).
    Select,
    /// Add current selection to group (Shift+N).
    AddToGroup,
    /// Unknown operation.
    Unknown(u8),
}

impl HotkeyOperation {
    /// Returns the raw operation byte.
    #[must_use]
    pub fn as_byte(&self) -> u8 {
        match self {
            HotkeyOperation::Assign => 0,
            HotkeyOperation::Select => 1,
            HotkeyOperation::AddToGroup => 2,
            HotkeyOperation::Unknown(n) => *n,
        }
    }
}

impl fmt::Display for HotkeyOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HotkeyOperation::Assign => write!(f, "Assign"),
            HotkeyOperation::Select => write!(f, "Select"),
            HotkeyOperation::AddToGroup => write!(f, "Add to"),
            HotkeyOperation::Unknown(n) => write!(f, "Unknown(0x{n:02X})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotkey_assign() {
        let data: &[u8] = &[0x17, 0x01, 0x00]; // Assign to group 1

        let (action, consumed) = HotkeyAction::parse(data).unwrap();

        assert_eq!(action.group, 1);
        assert_eq!(action.operation, HotkeyOperation::Assign);
        assert!(action.is_assign());
        assert!(!action.is_select());
        assert!(action.is_valid_group());
        assert_eq!(consumed, 3);
    }

    #[test]
    fn test_hotkey_select() {
        let data: &[u8] = &[0x17, 0x05, 0x01]; // Select group 5

        let (action, _) = HotkeyAction::parse(data).unwrap();

        assert_eq!(action.group, 5);
        assert_eq!(action.operation, HotkeyOperation::Select);
        assert!(action.is_select());
    }

    #[test]
    fn test_hotkey_add_to_group() {
        let data: &[u8] = &[0x17, 0x03, 0x02]; // Add to group 3

        let (action, _) = HotkeyAction::parse(data).unwrap();

        assert_eq!(action.group, 3);
        assert_eq!(action.operation, HotkeyOperation::AddToGroup);
    }

    #[test]
    fn test_hotkey_unknown_operation() {
        let data: &[u8] = &[0x17, 0x00, 0x05]; // Unknown operation 5

        let (action, _) = HotkeyAction::parse(data).unwrap();

        assert_eq!(action.operation, HotkeyOperation::Unknown(5));
    }

    #[test]
    fn test_hotkey_invalid_marker() {
        let data: &[u8] = &[0x16, 0x00, 0x00]; // Wrong marker

        let result = HotkeyAction::parse(data);
        assert!(matches!(result, Err(ParserError::InvalidHeader { .. })));
    }

    #[test]
    fn test_hotkey_truncated() {
        let data: &[u8] = &[0x17, 0x01]; // Missing operation byte

        let result = HotkeyAction::parse(data);
        assert!(matches!(result, Err(ParserError::UnexpectedEof { .. })));
    }

    #[test]
    fn test_hotkey_operation_display() {
        assert_eq!(format!("{}", HotkeyOperation::Assign), "Assign");
        assert_eq!(format!("{}", HotkeyOperation::Select), "Select");
        assert_eq!(format!("{}", HotkeyOperation::AddToGroup), "Add to");
        assert_eq!(format!("{}", HotkeyOperation::Unknown(10)), "Unknown(0x0A)");
    }

    #[test]
    fn test_hotkey_invalid_group() {
        let data: &[u8] = &[0x17, 0x0F, 0x01]; // Group 15 (invalid)

        let (action, _) = HotkeyAction::parse(data).unwrap();

        assert!(!action.is_valid_group());
    }
}
