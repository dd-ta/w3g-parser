//! Ability action parsing (0x1A, 0x0F).
//!
//! This module handles parsing of ability commands including:
//! - Direct abilities (0x1A 0x19)
//! - Abilities with selection (0x1A 0x00)
//! - Instant abilities (0x0F 0x00)

use super::selection::SelectionAction;
use crate::error::{ParserError, Result};
use std::fmt;

/// A `FourCC` ability code.
///
/// Ability codes are 4-byte identifiers stored in reverse byte order.
/// For example, "htow" (Town Hall) is stored as "woth" [77 6F 74 68].
///
/// # Race Prefixes
///
/// The first character of the canonical (reversed) form indicates race:
/// - `h`/`H`: Human (lowercase = units/buildings, uppercase = heroes)
/// - `o`/`O`: Orc
/// - `u`/`U`: Undead
/// - `e`/`E`: Night Elf
/// - `n`/`N`: Neutral
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct AbilityCode([u8; 4]);

impl AbilityCode {
    /// Creates an `AbilityCode` from raw bytes (as stored in replay).
    #[must_use]
    pub fn from_raw(bytes: [u8; 4]) -> Self {
        Self(bytes)
    }

    /// Gets the reversed/canonical form as a string.
    ///
    /// The canonical form is how Warcraft 3 ability IDs are typically written.
    /// For example, raw bytes [0x77, 0x6F, 0x74, 0x68] ("woth") become "htow".
    #[must_use]
    pub fn as_string(&self) -> String {
        let reversed = [self.0[3], self.0[2], self.0[1], self.0[0]];
        String::from_utf8_lossy(&reversed).to_string()
    }

    /// Gets the raw bytes as stored in the replay.
    #[must_use]
    pub fn raw_bytes(&self) -> [u8; 4] {
        self.0
    }

    /// Gets the race associated with this ability code, if identifiable.
    ///
    /// Race is determined by the first character of the canonical form.
    #[must_use]
    pub fn race(&self) -> Option<Race> {
        // The first character of canonical form is the last byte of raw
        match self.0[3] {
            b'h' | b'H' => Some(Race::Human),
            b'o' | b'O' => Some(Race::Orc),
            b'u' | b'U' => Some(Race::Undead),
            b'e' | b'E' => Some(Race::NightElf),
            b'n' | b'N' => Some(Race::Neutral),
            _ => None,
        }
    }

    /// Returns whether this appears to be a hero ability (uppercase prefix).
    #[must_use]
    pub fn is_hero_ability(&self) -> bool {
        matches!(self.0[3], b'H' | b'O' | b'U' | b'E' | b'N')
    }

    /// Returns whether the bytes are printable ASCII characters.
    #[must_use]
    pub fn is_valid_fourcc(&self) -> bool {
        self.0.iter().all(|&b| b.is_ascii_graphic() || b == b' ')
    }
}

impl fmt::Debug for AbilityCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AbilityCode({:?} -> \"{}\")",
            self.0,
            self.as_string()
        )
    }
}

impl fmt::Display for AbilityCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

/// Race enumeration for ability classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Race {
    /// Human alliance.
    Human,
    /// Orc horde.
    Orc,
    /// Undead scourge.
    Undead,
    /// Night Elf sentinels.
    NightElf,
    /// Neutral units and creeps.
    Neutral,
}

impl fmt::Display for Race {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Race::Human => write!(f, "Human"),
            Race::Orc => write!(f, "Orc"),
            Race::Undead => write!(f, "Undead"),
            Race::NightElf => write!(f, "Night Elf"),
            Race::Neutral => write!(f, "Neutral"),
        }
    }
}

/// A direct ability use action (0x1A 0x19).
///
/// Direct abilities are issued without an embedded selection block.
///
/// # Format
///
/// ```text
/// 1A 19 [ability: 4 bytes FourCC] [target: 8 bytes]
/// ```
///
/// Total size: 14 bytes (including marker bytes).
#[derive(Debug, Clone)]
pub struct AbilityAction {
    /// The ability `FourCC` code.
    pub ability_code: AbilityCode,

    /// Target unit ID (if targeting a unit).
    pub target_unit: Option<u32>,
}

impl AbilityAction {
    /// Action type marker.
    pub const MARKER: u8 = 0x1A;

    /// Subcommand for direct ability.
    pub const SUBCOMMAND_DIRECT: u8 = 0x19;

    /// Fixed size of a direct ability action (after player ID byte).
    pub const SIZE: usize = 14;

    /// Creates an `AbilityAction` from raw bytes (used by Reforged wrapper parsers).
    #[must_use]
    pub fn from_raw(ability_code: [u8; 4], target_unit: Option<u32>) -> Self {
        Self {
            ability_code: AbilityCode::from_raw(ability_code),
            target_unit,
        }
    }

    /// Parses a direct ability action from raw data.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw action data starting at 0x1A marker
    ///
    /// # Returns
    ///
    /// A tuple of `(AbilityAction, bytes_consumed)`.
    ///
    /// # Errors
    ///
    /// - `ParserError::InvalidHeader` if markers don't match
    /// - `ParserError::UnexpectedEof` if data is truncated
    pub fn parse(data: &[u8]) -> Result<(Self, usize)> {
        if data.len() < Self::SIZE {
            return Err(ParserError::unexpected_eof(Self::SIZE, data.len()));
        }

        if data[0] != Self::MARKER || data[1] != Self::SUBCOMMAND_DIRECT {
            return Err(ParserError::InvalidHeader {
                reason: format!(
                    "Invalid ability markers: expected 0x{:02X} 0x{:02X}, found 0x{:02X} 0x{:02X}",
                    Self::MARKER,
                    Self::SUBCOMMAND_DIRECT,
                    data[0],
                    data[1]
                ),
            });
        }

        // Extract FourCC code (bytes 2-5)
        let ability_code = AbilityCode::from_raw([data[2], data[3], data[4], data[5]]);

        // Extract target (bytes 6-13)
        // If all 0xFF, no target unit
        let target_bytes = &data[6..14];
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

        Ok((
            AbilityAction {
                ability_code,
                target_unit,
            },
            Self::SIZE,
        ))
    }
}

/// An ability with preceding selection (0x1A 0x00).
///
/// This action type embeds a selection block followed by an ability command.
///
/// # Format
///
/// ```text
/// 1A 00 [selection block: 16...] [1A 19 ability...]
/// ```
#[derive(Debug, Clone)]
pub struct AbilityWithSelectionAction {
    /// The selection that precedes this ability.
    pub selection: SelectionAction,

    /// The ability being used.
    pub ability: AbilityAction,
}

impl AbilityWithSelectionAction {
    /// Subcommand for ability with selection.
    pub const SUBCOMMAND: u8 = 0x00;

    /// Parses an ability with selection action.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw action data starting at 0x1A 0x00 markers
    ///
    /// # Returns
    ///
    /// A tuple of `(AbilityWithSelectionAction, bytes_consumed)`.
    ///
    /// # Errors
    ///
    /// - `ParserError::InvalidHeader` if markers don't match
    /// - `ParserError::UnexpectedEof` if data is truncated
    pub fn parse(data: &[u8]) -> Result<(Self, usize)> {
        if data.len() < 3 {
            return Err(ParserError::unexpected_eof(3, data.len()));
        }

        if data[0] != AbilityAction::MARKER || data[1] != Self::SUBCOMMAND {
            return Err(ParserError::InvalidHeader {
                reason: format!(
                    "Invalid ability with selection markers: expected 0x{:02X} 0x{:02X}, found 0x{:02X} 0x{:02X}",
                    AbilityAction::MARKER, Self::SUBCOMMAND, data[0], data[1]
                ),
            });
        }

        // Next should be a selection block (0x16)
        if data.len() < 3 || data[2] != SelectionAction::MARKER {
            return Err(ParserError::InvalidHeader {
                reason: format!(
                    "Expected selection marker 0x16 after 0x1A 0x00, found 0x{:02X}",
                    data.get(2).copied().unwrap_or(0)
                ),
            });
        }

        // Parse the selection block starting at offset 2
        let (selection, sel_consumed) = SelectionAction::parse(&data[2..])?;

        // After selection, should be an ability command (0x1A 0x19)
        let ability_offset = 2 + sel_consumed;
        if data.len() < ability_offset + AbilityAction::SIZE {
            return Err(ParserError::unexpected_eof(
                ability_offset + AbilityAction::SIZE,
                data.len(),
            ));
        }

        let (ability, ab_consumed) = AbilityAction::parse(&data[ability_offset..])?;

        let total_consumed = ability_offset + ab_consumed;

        Ok((
            AbilityWithSelectionAction { selection, ability },
            total_consumed,
        ))
    }
}

/// An instant ability (0x0F 0x00).
///
/// Instant abilities are typically auto-cast or queued abilities.
///
/// # Format
///
/// ```text
/// 0F 00 [flags: 2] [unknown: 2] [ability: 4 bytes] [target: 8 bytes]
/// ```
///
/// Total size: 18 bytes.
#[derive(Debug, Clone)]
pub struct InstantAbilityAction {
    /// Flags (2 bytes).
    pub flags: u16,

    /// Unknown field (2 bytes).
    pub unknown: u16,

    /// The ability `FourCC` code.
    pub ability_code: AbilityCode,

    /// Whether this has a target (0xFF padding means no target).
    pub has_target: bool,

    /// Target unit ID if present.
    pub target_unit: Option<u32>,
}

impl InstantAbilityAction {
    /// Action type marker.
    pub const MARKER: u8 = 0x0F;

    /// Subcommand for instant ability.
    pub const SUBCOMMAND: u8 = 0x00;

    /// Fixed size of an instant ability action.
    pub const SIZE: usize = 18;

    /// Parses an instant ability action.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw action data starting at 0x0F 0x00 markers
    ///
    /// # Returns
    ///
    /// A tuple of `(InstantAbilityAction, bytes_consumed)`.
    ///
    /// # Errors
    ///
    /// - `ParserError::InvalidHeader` if markers don't match
    /// - `ParserError::UnexpectedEof` if data is truncated
    pub fn parse(data: &[u8]) -> Result<(Self, usize)> {
        if data.len() < Self::SIZE {
            return Err(ParserError::unexpected_eof(Self::SIZE, data.len()));
        }

        if data[0] != Self::MARKER || data[1] != Self::SUBCOMMAND {
            return Err(ParserError::InvalidHeader {
                reason: format!(
                    "Invalid instant ability markers: expected 0x{:02X} 0x{:02X}, found 0x{:02X} 0x{:02X}",
                    Self::MARKER, Self::SUBCOMMAND, data[0], data[1]
                ),
            });
        }

        // Flags (bytes 2-3)
        let flags = u16::from_le_bytes([data[2], data[3]]);

        // Unknown (bytes 4-5)
        let unknown = u16::from_le_bytes([data[4], data[5]]);

        // Ability code (bytes 6-9)
        let ability_code = AbilityCode::from_raw([data[6], data[7], data[8], data[9]]);

        // Target (bytes 10-17)
        let target_bytes = &data[10..18];
        let has_target = !target_bytes.iter().all(|&b| b == 0xFF);
        let target_unit = if has_target {
            Some(u32::from_le_bytes([
                target_bytes[0],
                target_bytes[1],
                target_bytes[2],
                target_bytes[3],
            ]))
        } else {
            None
        };

        Ok((
            InstantAbilityAction {
                flags,
                unknown,
                ability_code,
                has_target,
                target_unit,
            },
            Self::SIZE,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ability_fourcc_parsing() {
        let raw = [0x77, 0x6F, 0x74, 0x68]; // "woth"
        let code = AbilityCode::from_raw(raw);

        assert_eq!(code.as_string(), "htow");
        assert_eq!(code.race(), Some(Race::Human));
        assert!(!code.is_hero_ability());
        assert!(code.is_valid_fourcc());
    }

    #[test]
    fn test_ability_hero_code() {
        let raw = [0x67, 0x6D, 0x61, 0x48]; // "gmaH" -> "Hamg"
        let code = AbilityCode::from_raw(raw);

        assert_eq!(code.as_string(), "Hamg");
        assert_eq!(code.race(), Some(Race::Human));
        assert!(code.is_hero_ability());
    }

    #[test]
    fn test_ability_night_elf() {
        let raw = [0x6D, 0x65, 0x64, 0x45]; // "medE" -> "Edem"
        let code = AbilityCode::from_raw(raw);

        assert_eq!(code.as_string(), "Edem");
        assert_eq!(code.race(), Some(Race::NightElf));
        assert!(code.is_hero_ability());
    }

    #[test]
    fn test_direct_ability_action() {
        // From Rex's analysis
        let data: &[u8] = &[
            0x1A, 0x19, // Ability command
            0x77, 0x6F, 0x74, 0x68, // "woth" = htow (Town Hall)
            0x3B, 0x3A, 0x00, 0x00, 0x3B, 0x3A, 0x00, 0x00, // Target
        ];

        let (action, consumed) = AbilityAction::parse(data).unwrap();

        assert_eq!(action.ability_code.as_string(), "htow");
        assert_eq!(action.target_unit, Some(0x0000_3A3B));
        assert_eq!(consumed, 14);
    }

    #[test]
    fn test_direct_ability_no_target() {
        let data: &[u8] = &[
            0x1A, 0x19, 0x77, 0x6F, 0x74, 0x68, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let (action, _) = AbilityAction::parse(data).unwrap();

        assert!(action.target_unit.is_none());
    }

    #[test]
    fn test_instant_ability_action() {
        // From Rex's analysis: 0F 00 10 42 00 [ability] [target]
        let data: &[u8] = &[
            0x0F, 0x00, // Instant ability marker
            0x10, 0x42, // Flags
            0x00, 0x00, // Unknown
            0x70, 0x73, 0x77, 0x65, // "pswe" -> "ewsp" (Wisp)
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // No target
        ];

        let (action, consumed) = InstantAbilityAction::parse(data).unwrap();

        assert_eq!(action.ability_code.as_string(), "ewsp");
        assert_eq!(action.flags, 0x4210);
        assert!(!action.has_target);
        assert_eq!(consumed, 18);
    }

    #[test]
    fn test_ability_with_selection() {
        // 0x1A 0x00 followed by selection and then ability
        let data: &[u8] = &[
            0x1A, 0x00, // Ability with selection marker
            0x16, 0x01, 0x01, 0x00, // Selection: 1 unit
            0x3B, 0x3A, 0x00, 0x00, 0x3B, 0x3A, 0x00, 0x00, // Unit ID
            0x1A, 0x19, // Ability command
            0x77, 0x6F, 0x74, 0x68, // "woth"
            0x3B, 0x3A, 0x00, 0x00, 0x3B, 0x3A, 0x00, 0x00, // Target
        ];

        let (action, consumed) = AbilityWithSelectionAction::parse(data).unwrap();

        assert_eq!(action.selection.unit_count, 1);
        assert_eq!(action.ability.ability_code.as_string(), "htow");
        assert_eq!(consumed, 28); // 2 + 12 (selection) + 14 (ability)
    }

    #[test]
    fn test_race_display() {
        assert_eq!(format!("{}", Race::Human), "Human");
        assert_eq!(format!("{}", Race::NightElf), "Night Elf");
    }
}
