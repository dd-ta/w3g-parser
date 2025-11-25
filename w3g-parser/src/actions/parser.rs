//! Action parsing framework and iterator.
//!
//! This module provides the `ActionIterator` for parsing individual player actions
//! from `TimeFrame` action data.

use super::ability::{AbilityAction, AbilityWithSelectionAction, InstantAbilityAction};
use super::hotkey::HotkeyAction;
use super::movement::MovementAction;
use super::selection::SelectionAction;
use super::types::{Action, ActionType};
use crate::error::{ParserError, Result};

/// Context for parsing actions within a `TimeFrame`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ActionContext {
    /// Timestamp from the containing `TimeFrame`.
    pub timestamp_ms: u32,

    /// Frame number for debugging.
    pub frame_number: u32,
}

impl ActionContext {
    /// Creates a new action context.
    #[must_use]
    pub fn new(timestamp_ms: u32, frame_number: u32) -> Self {
        Self {
            timestamp_ms,
            frame_number,
        }
    }
}

/// Iterator over actions within a `TimeFrame`'s action data.
///
/// This iterator parses action blocks from raw action data, yielding
/// `Action` structs with player IDs, action types, and timestamps.
///
/// # W3G Format Structure
///
/// The action data contains sequential player actions:
/// - 1 byte: PlayerID (1-15)
/// - 1 byte: ActionType
/// - n bytes: Action data (varies by action type)
///
/// # Example
///
/// ```ignore
/// use w3g_parser::actions::{ActionIterator, ActionContext};
///
/// let action_data: &[u8] = &[...]; // From TimeFrame
/// let ctx = ActionContext::new(1000, 0);
///
/// for result in ActionIterator::new(action_data, ctx) {
///     match result {
///         Ok(action) => println!("Player {}: {:?}", action.player_id, action.action_type),
///         Err(e) => eprintln!("Parse error: {}", e),
///     }
/// }
/// ```
pub struct ActionIterator<'a> {
    /// Raw action data bytes.
    data: &'a [u8],

    /// Current offset within data.
    offset: usize,

    /// Context from containing `TimeFrame`.
    context: ActionContext,

    /// Whether iteration has finished.
    finished: bool,
}

impl<'a> ActionIterator<'a> {
    /// Creates a new action iterator.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw action data from a `TimeFrame`
    /// * `context` - Context with timestamp and frame number
    #[must_use]
    pub fn new(data: &'a [u8], context: ActionContext) -> Self {
        Self {
            data,
            offset: 0,
            context,
            finished: false,
        }
    }

    /// Returns the current offset in the data.
    #[must_use]
    pub fn current_offset(&self) -> usize {
        self.offset
    }

    /// Returns the remaining bytes to parse.
    #[must_use]
    pub fn remaining_bytes(&self) -> usize {
        self.data.len().saturating_sub(self.offset)
    }

    /// Returns whether iteration is finished.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Parses the next action from the data.
    fn parse_next(&mut self) -> Result<Action> {
        if self.offset >= self.data.len() {
            return Err(ParserError::unexpected_eof(1, 0));
        }

        let data = &self.data[self.offset..];

        // First byte should be player ID (1-15)
        let player_id = data[0];
        if player_id == 0 || player_id > 15 {
            return Err(ParserError::InvalidHeader {
                reason: format!(
                    "Invalid player ID {} at offset {}, expected 1-15",
                    player_id, self.offset
                ),
            });
        }

        // Check we have at least 2 bytes for action type
        if data.len() < 2 {
            return Err(ParserError::unexpected_eof(2, data.len()));
        }

        let action_type_byte = data[1];
        let subcommand = data.get(2).copied();

        // Dispatch based on action type
        let (action_type, bytes_consumed) =
            Self::parse_action_type(action_type_byte, subcommand, &data[1..])?;

        // Total consumed includes player ID byte
        let total_consumed = 1 + bytes_consumed;
        self.offset += total_consumed;

        Ok(Action::new(player_id, action_type, self.context.timestamp_ms))
    }

    /// Parses the action type from the remaining data.
    fn parse_action_type(
        action_type: u8,
        subcommand: Option<u8>,
        data: &[u8],
    ) -> Result<(ActionType, usize)> {
        match (action_type, subcommand) {
            // Movement commands (0x00 with various subcommands)
            // 0x0D = Move, 0x0E = Attack-move, 0x0F = Patrol, 0x10 = Hold, 0x12 = Smart-click
            (0x00, Some(sub @ (0x0D | 0x0E | 0x0F | 0x10 | 0x12))) => {
                let (mov, consumed) = MovementAction::parse_with_subcommand(data, sub)?;
                Ok((ActionType::Movement(mov), consumed))
            }

            // Instant ability (0x0F 0x00)
            (0x0F, Some(0x00)) => {
                let (ab, consumed) = InstantAbilityAction::parse(data)?;
                Ok((ActionType::InstantAbility(ab), consumed))
            }

            // Selection (0x16)
            (0x16, _) => {
                let (sel, consumed) = SelectionAction::parse(data)?;
                Ok((ActionType::Selection(sel), consumed))
            }

            // Hotkey (0x17)
            (0x17, _) => {
                let (hk, consumed) = HotkeyAction::parse(data)?;
                Ok((ActionType::Hotkey(hk), consumed))
            }

            // Ability with selection (0x1A 0x00)
            (0x1A, Some(0x00)) => {
                let (ab, consumed) = AbilityWithSelectionAction::parse(data)?;
                Ok((ActionType::AbilityWithSelection(ab), consumed))
            }

            // Direct ability (0x1A 0x19)
            (0x1A, Some(0x19)) => {
                let (ab, consumed) = AbilityAction::parse(data)?;
                Ok((ActionType::Ability(ab), consumed))
            }

            // ESC key (0x18) - cancel current action
            (0x18, _) => {
                // ESC consumes just the action type byte
                Ok((ActionType::EscapeKey, 1))
            }

            // Item action (0x1B)
            (0x1B, _) => {
                // Item actions: type(1) + flags(1) + item_id(4) + target(8) = 14 bytes
                let consumed = 14.min(data.len());
                let item_id = if data.len() >= 6 {
                    u32::from_le_bytes([data[2], data[3], data[4], data[5]])
                } else {
                    0
                };
                Ok((ActionType::ItemAction { item_id }, consumed))
            }

            // Basic command (0x1C) - stop, hold position, patrol
            (0x1C, _) => {
                // Basic commands: type(1) + flags(1) + command_id(4) + target(8) = 14 bytes
                let consumed = 14.min(data.len());
                let command_id = if data.len() >= 6 {
                    u32::from_le_bytes([data[2], data[3], data[4], data[5]])
                } else {
                    0
                };
                Ok((ActionType::BasicCommand { command_id }, consumed))
            }

            // Build/Train command (0x1D)
            (0x1D, _) => {
                // Build/train: type(1) + flags(1) + unit_code(4) + position(8+) = ~14 bytes
                let consumed = 14.min(data.len());
                let unit_code = if data.len() >= 6 {
                    [data[2], data[3], data[4], data[5]]
                } else {
                    [0, 0, 0, 0]
                };
                Ok((ActionType::BuildTrain { unit_code }, consumed))
            }

            // Select subgroup (0x19)
            (0x19, _) => {
                // SelectSubgroup: type(1) + item(1) + object_id1(4) + object_id2(4) + unknown(3) = 13 bytes
                let consumed = 13.min(data.len());
                if data.len() >= 10 {
                    let item = data[1];
                    let object_id1 = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
                    let object_id2 = u32::from_le_bytes([data[6], data[7], data[8], data[9]]);
                    Ok((
                        ActionType::SelectSubgroup {
                            item,
                            object_id1,
                            object_id2,
                        },
                        consumed,
                    ))
                } else {
                    Ok(Self::parse_unknown_action(action_type, subcommand, data))
                }
            }

            // Remove from queue (0x1E)
            (0x1E, _) => {
                // RemoveFromQueue: type(1) + slot(1) + unit_id(4) = 6 bytes
                let consumed = 6.min(data.len());
                if data.len() >= 6 {
                    let slot = data[1];
                    let unit_id = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
                    Ok((ActionType::RemoveFromQueue { slot, unit_id }, consumed))
                } else {
                    Ok(Self::parse_unknown_action(action_type, subcommand, data))
                }
            }

            // Change ally options (0x50)
            (0x50, _) => {
                // ChangeAllyOptions: type(1) + slot(1) + flags(4) = 6 bytes
                let consumed = 6.min(data.len());
                if data.len() >= 6 {
                    let slot = data[1];
                    let flags = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
                    Ok((ActionType::ChangeAllyOptions { slot, flags }, consumed))
                } else {
                    Ok(Self::parse_unknown_action(action_type, subcommand, data))
                }
            }

            // Transfer resources (0x51)
            (0x51, _) => {
                // TransferResources: type(1) + slot(1) + gold(4) + lumber(4) = 10 bytes
                let consumed = 10.min(data.len());
                if data.len() >= 10 {
                    let slot = data[1];
                    let gold = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
                    let lumber = u32::from_le_bytes([data[6], data[7], data[8], data[9]]);
                    Ok((
                        ActionType::TransferResources { slot, gold, lumber },
                        consumed,
                    ))
                } else {
                    Ok(Self::parse_unknown_action(action_type, subcommand, data))
                }
            }

            // Minimap ping (0x68)
            (0x68, _) => {
                // MinimapPing: type(1) + x(4) + y(4) + unknown(4) = 13 bytes
                let consumed = 13.min(data.len());
                if data.len() >= 13 {
                    let x = f32::from_le_bytes([data[1], data[2], data[3], data[4]]);
                    let y = f32::from_le_bytes([data[5], data[6], data[7], data[8]]);
                    let unknown = u32::from_le_bytes([data[9], data[10], data[11], data[12]]);
                    Ok((ActionType::MinimapPing { x, y, unknown }, consumed))
                } else {
                    Ok(Self::parse_unknown_action(action_type, subcommand, data))
                }
            }

            // Note: Action types 0x10-0x14 are theoretically unit ability types
            // according to W3G documentation, but in practice they seem to
            // appear as data bytes within other actions in most replays.
            // They are left to fall through to the Unknown handler to avoid
            // desynchronization. The ActionType enum still has these variants
            // defined for future use when we can properly validate them.

            // Unknown action types - try to find the next action boundary
            _ => Ok(Self::parse_unknown_action(action_type, subcommand, data)),
        }
    }

    /// Handles unknown action types by finding the next action boundary.
    fn parse_unknown_action(
        type_id: u8,
        subcommand: Option<u8>,
        data: &[u8],
    ) -> (ActionType, usize) {
        // Try to find the next valid action start
        // Look for a byte 1-15 followed by a known action type
        let mut end_offset = 1;

        while end_offset < data.len() {
            let potential_player = data[end_offset];
            if (1..=15).contains(&potential_player) {
                // Check if next byte looks like a valid action type
                if let Some(&next_type) = data.get(end_offset + 1) {
                    if is_known_action_type(next_type) {
                        // Found what looks like next action
                        break;
                    }
                }
            }
            end_offset += 1;
        }

        // Capture the unknown action data
        let action_data = data[1..end_offset].to_vec();

        (
            ActionType::Unknown {
                type_id,
                subcommand,
                data: action_data,
            },
            end_offset,
        )
    }
}

impl Iterator for ActionIterator<'_> {
    type Item = Result<Action>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.offset >= self.data.len() {
            return None;
        }

        match self.parse_next() {
            Ok(action) => Some(Ok(action)),
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}

/// Returns whether the byte is a known action type marker.
///
/// Note: 0x10-0x14 are unit ability types but are NOT included here
/// because they can also appear as data bytes within other actions,
/// making boundary detection unreliable. They are still parsed when
/// encountered, but we don't use them for finding action boundaries.
fn is_known_action_type(byte: u8) -> bool {
    matches!(
        byte,
        0x00 | 0x0F | 0x16 | 0x17 | 0x18 | 0x19 | 0x1A | 0x1B | 0x1C | 0x1D | 0x1E | 0x50 | 0x51 | 0x68
    )
}

/// Statistics about actions parsed from a replay.
#[derive(Debug, Default, Clone)]
pub struct ActionStatistics {
    /// Total number of actions parsed.
    pub total_actions: u32,

    /// Number of selection actions.
    pub selection_actions: u32,

    /// Number of ability actions (all types).
    pub ability_actions: u32,

    /// Number of movement actions.
    pub movement_actions: u32,

    /// Number of hotkey actions.
    pub hotkey_actions: u32,

    /// Number of unknown actions.
    pub unknown_actions: u32,

    /// Actions per player (keyed by player ID).
    pub actions_per_player: std::collections::HashMap<u8, u32>,

    /// Unique ability codes seen.
    pub unique_ability_codes: std::collections::HashSet<[u8; 4]>,
}

impl ActionStatistics {
    /// Creates new empty statistics.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records an action in the statistics.
    pub fn record(&mut self, action: &Action) {
        self.total_actions += 1;
        *self.actions_per_player.entry(action.player_id).or_insert(0) += 1;

        match &action.action_type {
            ActionType::Selection(_) => self.selection_actions += 1,
            ActionType::Ability(ab) => {
                self.ability_actions += 1;
                self.unique_ability_codes.insert(ab.ability_code.raw_bytes());
            }
            ActionType::AbilityWithSelection(ab) => {
                self.ability_actions += 1;
                self.unique_ability_codes
                    .insert(ab.ability.ability_code.raw_bytes());
            }
            ActionType::InstantAbility(ab) => {
                self.ability_actions += 1;
                self.unique_ability_codes.insert(ab.ability_code.raw_bytes());
            }
            ActionType::Movement(_) => self.movement_actions += 1,
            ActionType::Hotkey(_) => self.hotkey_actions += 1,
            ActionType::EscapeKey => {
                // ESC is a known action type, counts toward total
            }
            ActionType::ItemAction { .. } => {
                // Item usage counts as ability-like
                self.ability_actions += 1;
            }
            ActionType::BasicCommand { .. } => {
                // Basic commands (stop, hold) count as movement-like
                self.movement_actions += 1;
            }
            ActionType::BuildTrain { unit_code } => {
                self.ability_actions += 1;
                self.unique_ability_codes.insert(*unit_code);
            }
            ActionType::UnitAbilityNoTarget { ability_code, .. } => {
                self.ability_actions += 1;
                self.unique_ability_codes.insert(*ability_code);
            }
            ActionType::UnitAbilityGroundTarget { ability_code, .. } => {
                self.ability_actions += 1;
                self.unique_ability_codes.insert(*ability_code);
            }
            ActionType::UnitAbilityUnitTarget { ability_code, .. } => {
                self.ability_actions += 1;
                self.unique_ability_codes.insert(*ability_code);
            }
            ActionType::GiveDropItem { item_code, .. } => {
                self.ability_actions += 1;
                self.unique_ability_codes.insert(*item_code);
            }
            ActionType::UnitAbilityTwoTargets { ability_code, .. } => {
                self.ability_actions += 1;
                self.unique_ability_codes.insert(*ability_code);
            }
            ActionType::SelectSubgroup { .. } => {
                // SelectSubgroup is selection-like
                self.selection_actions += 1;
            }
            ActionType::RemoveFromQueue { .. } => {
                // Removing from queue is ability-like
                self.ability_actions += 1;
            }
            ActionType::ChangeAllyOptions { .. } => {
                // Ally options are ESC-like (meta actions)
            }
            ActionType::TransferResources { .. } => {
                // Resource transfer is ability-like
                self.ability_actions += 1;
            }
            ActionType::MinimapPing { .. } => {
                // Pings are movement-like (map interaction)
                self.movement_actions += 1;
            }
            ActionType::Unknown { .. } => self.unknown_actions += 1,
        }
    }

    /// Returns the number of unique ability codes.
    #[must_use]
    pub fn unique_ability_count(&self) -> usize {
        self.unique_ability_codes.len()
    }

    /// Returns the error rate (unknown actions / total actions).
    #[must_use]
    pub fn error_rate(&self) -> f64 {
        if self.total_actions == 0 {
            0.0
        } else {
            f64::from(self.unknown_actions) / f64::from(self.total_actions)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_iterator_single_selection() {
        // Player 4, Selection with 1 unit
        let data: &[u8] = &[
            0x04, // Player 4
            0x16, 0x01, 0x01, 0x00, // Selection: 1 unit, mode 1
            0x3B, 0x3A, 0x00, 0x00, 0x3B, 0x3A, 0x00, 0x00, // Unit ID
        ];

        let ctx = ActionContext::new(1000, 1);
        let mut iter = ActionIterator::new(data, ctx);

        let action = iter.next().unwrap().unwrap();
        assert_eq!(action.player_id, 4);
        assert_eq!(action.timestamp_ms, 1000);
        assert!(matches!(action.action_type, ActionType::Selection(_)));

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_action_iterator_direct_ability() {
        // Player 3, Direct ability
        let data: &[u8] = &[
            0x03, // Player 3
            0x1A, 0x19, // Ability command
            0x77, 0x6F, 0x74, 0x68, // "woth"
            0x3B, 0x3A, 0x00, 0x00, 0x3B, 0x3A, 0x00, 0x00, // Target
        ];

        let ctx = ActionContext::new(2000, 2);
        let mut iter = ActionIterator::new(data, ctx);

        let action = iter.next().unwrap().unwrap();
        assert_eq!(action.player_id, 3);
        if let ActionType::Ability(ab) = &action.action_type {
            assert_eq!(ab.ability_code.as_string(), "htow");
        } else {
            panic!("Expected Ability action type");
        }
    }

    #[test]
    fn test_action_iterator_movement() {
        // Player 5, Movement command
        let data: &[u8] = &[
            0x05, // Player 5
            0x00, 0x0D, 0x00, 0x00, // Move command
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // No target
            0x00, 0x00, 0xB0, 0xC5, // X
            0x00, 0x00, 0x60, 0x45, // Y
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Extra
        ];

        let ctx = ActionContext::new(3000, 3);
        let mut iter = ActionIterator::new(data, ctx);

        let action = iter.next().unwrap().unwrap();
        assert_eq!(action.player_id, 5);
        assert!(matches!(action.action_type, ActionType::Movement(_)));
    }

    #[test]
    fn test_action_iterator_multiple_players() {
        // From Rex's analysis: two instant abilities from different players
        let data: &[u8] = &[
            // Player 5's instant ability
            0x05, 0x0F, 0x00, 0x10, 0x42, 0x00, 0x00, 0x70, 0x73, 0x77, 0x65, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            // Player 4's instant ability
            0x04, 0x0F, 0x00, 0x10, 0x42, 0x00, 0x00, 0x61, 0x65, 0x70, 0x68, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let ctx = ActionContext::new(4000, 4);
        let iter = ActionIterator::new(data, ctx);
        let actions: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();

        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].player_id, 5);
        assert_eq!(actions[1].player_id, 4);
    }

    #[test]
    fn test_action_iterator_empty_data() {
        let data: &[u8] = &[];
        let ctx = ActionContext::default();
        let mut iter = ActionIterator::new(data, ctx);

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_action_iterator_invalid_player() {
        let data: &[u8] = &[0x00, 0x16, 0x01, 0x01, 0x00]; // Player 0 is invalid

        let ctx = ActionContext::default();
        let mut iter = ActionIterator::new(data, ctx);

        let result = iter.next().unwrap();
        assert!(matches!(result, Err(ParserError::InvalidHeader { .. })));
    }

    #[test]
    fn test_action_context() {
        let ctx = ActionContext::new(5000, 10);
        assert_eq!(ctx.timestamp_ms, 5000);
        assert_eq!(ctx.frame_number, 10);

        let default_ctx = ActionContext::default();
        assert_eq!(default_ctx.timestamp_ms, 0);
        assert_eq!(default_ctx.frame_number, 0);
    }

    #[test]
    fn test_action_statistics() {
        let mut stats = ActionStatistics::new();

        let action1 = Action::new(
            1,
            ActionType::Selection(SelectionAction {
                unit_count: 1,
                mode: 1,
                flags: 0,
                unit_ids: vec![0x1234],
            }),
            1000,
        );

        let action2 = Action::new(
            2,
            ActionType::Ability(AbilityAction {
                ability_code: crate::actions::AbilityCode::from_raw([0x77, 0x6F, 0x74, 0x68]),
                target_unit: None,
            }),
            2000,
        );

        stats.record(&action1);
        stats.record(&action2);

        assert_eq!(stats.total_actions, 2);
        assert_eq!(stats.selection_actions, 1);
        assert_eq!(stats.ability_actions, 1);
        assert_eq!(stats.unique_ability_count(), 1);
        assert_eq!(stats.actions_per_player.get(&1), Some(&1));
        assert_eq!(stats.actions_per_player.get(&2), Some(&1));
    }

    #[test]
    fn test_is_known_action_type() {
        assert!(is_known_action_type(0x00)); // Movement
        assert!(is_known_action_type(0x0F)); // InstantAbility
        // Note: 0x10-0x14 are NOT in is_known_action_type because they
        // can appear as data bytes, making boundary detection unreliable
        assert!(!is_known_action_type(0x10)); // Not used for boundary detection
        assert!(!is_known_action_type(0x11)); // Not used for boundary detection
        assert!(!is_known_action_type(0x12)); // Not used for boundary detection
        assert!(!is_known_action_type(0x13)); // Not used for boundary detection
        assert!(!is_known_action_type(0x14)); // Not used for boundary detection
        assert!(is_known_action_type(0x16)); // Selection
        assert!(is_known_action_type(0x17)); // Hotkey
        assert!(is_known_action_type(0x18)); // ESC
        assert!(is_known_action_type(0x1A)); // Ability
        assert!(is_known_action_type(0x1B)); // Item
        assert!(is_known_action_type(0x1C)); // BasicCommand
        assert!(is_known_action_type(0x1D)); // BuildTrain
        assert!(!is_known_action_type(0x15)); // Unknown
        assert!(!is_known_action_type(0xFF)); // Unknown
    }
}
