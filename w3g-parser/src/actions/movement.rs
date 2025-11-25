//! Movement action parsing (0x00 with various subcommands).
//!
//! Movement actions represent right-click commands (move/attack)
//! with target coordinates and optional target unit.

use crate::error::{ParserError, Result};
use std::fmt;

/// Type of movement command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementType {
    /// Regular move (right-click ground) - 0x0D.
    Move,
    /// Attack-move (A-click) - 0x0E.
    AttackMove,
    /// Patrol command - 0x0F.
    Patrol,
    /// Hold position - 0x10.
    HoldPosition,
    /// Smart right-click (context-aware) - 0x12.
    SmartClick,
    /// Unknown movement subcommand.
    Unknown(u8),
}

impl MovementType {
    /// Creates a MovementType from a subcommand byte.
    #[must_use]
    pub fn from_subcommand(sub: u8) -> Self {
        match sub {
            0x0D => MovementType::Move,
            0x0E => MovementType::AttackMove,
            0x0F => MovementType::Patrol,
            0x10 => MovementType::HoldPosition,
            0x12 => MovementType::SmartClick,
            n => MovementType::Unknown(n),
        }
    }

    /// Returns the subcommand byte for this movement type.
    #[must_use]
    pub fn as_subcommand(&self) -> u8 {
        match self {
            MovementType::Move => 0x0D,
            MovementType::AttackMove => 0x0E,
            MovementType::Patrol => 0x0F,
            MovementType::HoldPosition => 0x10,
            MovementType::SmartClick => 0x12,
            MovementType::Unknown(n) => *n,
        }
    }
}

impl fmt::Display for MovementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MovementType::Move => write!(f, "Move"),
            MovementType::AttackMove => write!(f, "Attack-Move"),
            MovementType::Patrol => write!(f, "Patrol"),
            MovementType::HoldPosition => write!(f, "Hold Position"),
            MovementType::SmartClick => write!(f, "Smart Click"),
            MovementType::Unknown(n) => write!(f, "Unknown(0x{n:02X})"),
        }
    }
}

/// Map position in game coordinates.
///
/// Warcraft 3 uses IEEE 754 single-precision floats for map coordinates.
/// The coordinate system has:
/// - Negative X = West, Positive X = East
/// - Negative Y = South, Positive Y = North
/// - Typical range: approximately -10000 to +10000
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    /// X coordinate (West-East axis).
    pub x: f32,
    /// Y coordinate (South-North axis).
    pub y: f32,
}

impl Position {
    /// Creates a new position.
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Calculates the distance to another position.
    #[must_use]
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Returns whether the coordinates are within valid map bounds.
    ///
    /// Valid coordinates are finite and within a reasonable range.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        const MAP_BOUND: f32 = 15000.0;
        self.x.is_finite()
            && self.y.is_finite()
            && self.x.abs() < MAP_BOUND
            && self.y.abs() < MAP_BOUND
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.1}, {:.1})", self.x, self.y)
    }
}

/// A move or attack command.
///
/// Movement actions represent right-click commands with target coordinates
/// and optional target unit.
///
/// # Format
///
/// ```text
/// 00 [sub] [flags: 2] [target: 8 bytes] [x: 4 float] [y: 4 float] [extra: 8]
/// ```
///
/// Subcommands: 0x0D=Move, 0x0E=Attack-Move, 0x0F=Patrol, 0x10=Hold, 0x12=Smart
/// Total size: 28 bytes.
#[derive(Debug, Clone)]
pub struct MovementAction {
    /// Type of movement command (Move, Attack-Move, Patrol, etc.).
    pub movement_type: MovementType,

    /// Command flags (meaning under investigation).
    pub flags: u16,

    /// Target unit ID (if right-clicking on a unit).
    /// Will be `None` if targeting ground (0xFFFFFFFF in data).
    pub target_unit: Option<u32>,

    /// X coordinate (IEEE 754 float, little-endian).
    pub x: f32,

    /// Y coordinate (IEEE 754 float, little-endian).
    pub y: f32,

    /// Additional data (8 bytes, purpose under investigation).
    /// May contain item ID or secondary target.
    pub extra_data: [u8; 8],
}

impl MovementAction {
    /// Action type marker.
    pub const MARKER: u8 = 0x00;

    /// Subcommand for move/attack.
    pub const SUBCOMMAND: u8 = 0x0D;

    /// Fixed size of a movement action.
    pub const SIZE: usize = 28;

    /// Parses a movement action from raw data (default subcommand 0x0D).
    ///
    /// # Arguments
    ///
    /// * `data` - Raw action data starting at 0x00 0x0D markers
    ///
    /// # Returns
    ///
    /// A tuple of `(MovementAction, bytes_consumed)`.
    ///
    /// # Errors
    ///
    /// - `ParserError::InvalidHeader` if markers don't match
    /// - `ParserError::UnexpectedEof` if data is truncated
    pub fn parse(data: &[u8]) -> Result<(Self, usize)> {
        Self::parse_with_subcommand(data, Self::SUBCOMMAND)
    }

    /// Parses a movement action with a specified subcommand.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw action data starting at 0x00 marker
    /// * `expected_subcommand` - Expected subcommand byte (0x0D, 0x0E, 0x0F, 0x10, 0x12)
    ///
    /// # Returns
    ///
    /// A tuple of `(MovementAction, bytes_consumed)`.
    pub fn parse_with_subcommand(data: &[u8], expected_subcommand: u8) -> Result<(Self, usize)> {
        if data.len() < Self::SIZE {
            return Err(ParserError::unexpected_eof(Self::SIZE, data.len()));
        }

        if data[0] != Self::MARKER || data[1] != expected_subcommand {
            return Err(ParserError::InvalidHeader {
                reason: format!(
                    "Invalid movement markers: expected 0x{:02X} 0x{:02X}, found 0x{:02X} 0x{:02X}",
                    Self::MARKER,
                    expected_subcommand,
                    data[0],
                    data[1]
                ),
            });
        }

        let movement_type = MovementType::from_subcommand(data[1]);

        // Flags (bytes 2-3)
        let flags = u16::from_le_bytes([data[2], data[3]]);

        // Target unit (bytes 4-11)
        // If all 0xFF, targeting ground
        let target_bytes = &data[4..12];
        let target_unit = if target_bytes.iter().all(|&b| b == 0xFF) {
            None
        } else {
            Some(u32::from_le_bytes([
                target_bytes[0],
                target_bytes[1],
                target_bytes[2],
                target_bytes[3],
            ]))
        };

        // X coordinate (bytes 12-15, IEEE 754 float LE)
        let x = f32::from_le_bytes([data[12], data[13], data[14], data[15]]);

        // Y coordinate (bytes 16-19, IEEE 754 float LE)
        let y = f32::from_le_bytes([data[16], data[17], data[18], data[19]]);

        // Extra data (bytes 20-27)
        let mut extra_data = [0u8; 8];
        extra_data.copy_from_slice(&data[20..28]);

        Ok((
            MovementAction {
                movement_type,
                flags,
                target_unit,
                x,
                y,
                extra_data,
            },
            Self::SIZE,
        ))
    }

    /// Check if this is a ground-target command (no unit target).
    #[must_use]
    pub fn is_ground_target(&self) -> bool {
        self.target_unit.is_none()
    }

    /// Get the target position as a `Position` struct.
    #[must_use]
    pub fn position(&self) -> Position {
        Position { x: self.x, y: self.y }
    }

    /// Check if coordinates are within valid map range.
    #[must_use]
    pub fn is_valid_position(&self) -> bool {
        self.position().is_valid()
    }
}

impl fmt::Display for MovementAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(target) = self.target_unit {
            write!(
                f,
                "{} to {} targeting unit 0x{:08X}",
                self.movement_type,
                self.position(),
                target
            )
        } else {
            write!(f, "{} to {}", self.movement_type, self.position())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movement_ground_target() {
        // From Rex's analysis
        let data: &[u8] = &[
            0x00, 0x0D, 0x00, 0x00, // Move command, flags=0
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // No target (ground)
            0x00, 0x00, 0xB0, 0xC5, // X = -5632.0
            0x00, 0x00, 0x60, 0x45, // Y = 3584.0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Extra
        ];

        let (action, consumed) = MovementAction::parse(data).unwrap();

        assert_eq!(consumed, 28);
        assert!(action.is_ground_target());
        assert!((action.x - (-5632.0)).abs() < 0.1);
        assert!((action.y - 3584.0).abs() < 0.1);
        assert!(action.is_valid_position());
    }

    #[test]
    fn test_movement_unit_target() {
        let data: &[u8] = &[
            0x00, 0x0D, 0x01, 0x00, // Move command, flags=1
            0x34, 0x12, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00, // Target unit 0x1234
            0x00, 0x00, 0x80, 0x44, // X = 1024.0
            0x00, 0x00, 0x00, 0x45, // Y = 2048.0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Extra
        ];

        let (action, _) = MovementAction::parse(data).unwrap();

        assert!(!action.is_ground_target());
        assert_eq!(action.target_unit, Some(0x1234));
        assert!((action.x - 1024.0).abs() < 0.1);
        assert!((action.y - 2048.0).abs() < 0.1);
    }

    #[test]
    fn test_movement_invalid_markers() {
        let data: &[u8] = &[
            0x01, 0x0D, // Wrong first byte
            0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let result = MovementAction::parse(data);
        assert!(matches!(result, Err(ParserError::InvalidHeader { .. })));
    }

    #[test]
    fn test_movement_truncated() {
        let data: &[u8] = &[0x00, 0x0D, 0x00, 0x00]; // Only 4 bytes
        let result = MovementAction::parse(data);
        assert!(matches!(result, Err(ParserError::UnexpectedEof { .. })));
    }

    #[test]
    fn test_position_distance() {
        let p1 = Position::new(0.0, 0.0);
        let p2 = Position::new(3.0, 4.0);

        assert!((p1.distance_to(&p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_position_validity() {
        assert!(Position::new(0.0, 0.0).is_valid());
        assert!(Position::new(-5000.0, 5000.0).is_valid());
        assert!(!Position::new(f32::NAN, 0.0).is_valid());
        assert!(!Position::new(0.0, f32::INFINITY).is_valid());
        assert!(!Position::new(20000.0, 0.0).is_valid());
    }

    #[test]
    fn test_position_display() {
        let pos = Position::new(1234.5, -6789.0);
        assert_eq!(format!("{pos}"), "(1234.5, -6789.0)");
    }
}
