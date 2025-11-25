//! Core action types and structures.
//!
//! This module defines the main `Action` struct and `ActionType` enum that
//! represent parsed player actions.

use super::ability::{AbilityAction, AbilityWithSelectionAction, InstantAbilityAction};
use super::hotkey::HotkeyAction;
use super::movement::MovementAction;
use super::selection::SelectionAction;
use std::fmt;

/// A parsed action from a `TimeFrame`.
///
/// Actions represent individual commands issued by players during gameplay.
/// Each action has a player ID, timestamp, and action-specific data.
#[derive(Debug, Clone)]
pub struct Action {
    /// Player ID who issued this action (1-15).
    pub player_id: u8,

    /// Action type with parsed data.
    pub action_type: ActionType,

    /// Timestamp in milliseconds from game start.
    /// Inherited from the containing `TimeFrame`.
    pub timestamp_ms: u32,
}

impl Action {
    /// Creates a new action with the given parameters.
    #[must_use]
    pub fn new(player_id: u8, action_type: ActionType, timestamp_ms: u32) -> Self {
        Self {
            player_id,
            action_type,
            timestamp_ms,
        }
    }

    /// Returns a human-readable description of this action.
    #[must_use]
    pub fn description(&self) -> String {
        format!(
            "Player {} at {}ms: {}",
            self.player_id,
            self.timestamp_ms,
            self.action_type.type_name()
        )
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[P{} @{}ms] {}",
            self.player_id,
            self.timestamp_ms,
            self.action_type
        )
    }
}

/// Enumeration of all known action types.
///
/// Each variant contains the parsed data specific to that action type.
/// Unknown actions preserve their raw data for debugging and forward compatibility.
#[derive(Debug, Clone)]
pub enum ActionType {
    /// Unit selection (0x16).
    Selection(SelectionAction),

    /// Direct ability use (0x1A 0x19).
    Ability(AbilityAction),

    /// Ability use with selection (0x1A 0x00).
    AbilityWithSelection(AbilityWithSelectionAction),

    /// Instant ability (0x0F 0x00).
    InstantAbility(InstantAbilityAction),

    /// Move/Attack command (0x00 0x0D).
    Movement(MovementAction),

    /// Control group hotkey (0x17).
    Hotkey(HotkeyAction),

    /// ESC key press - cancel action (0x18).
    EscapeKey,

    /// Item usage or drop (0x1B).
    ItemAction {
        /// The item's FourCC identifier.
        item_id: u32,
    },

    /// Basic command like stop, hold position, patrol (0x1C).
    BasicCommand {
        /// The command FourCC identifier.
        command_id: u32,
    },

    /// Build or train command (0x1D).
    BuildTrain {
        /// The unit type's FourCC code.
        unit_code: [u8; 4],
    },

    /// Unit ability without target (0x10).
    UnitAbilityNoTarget {
        /// Flags byte.
        flags: u8,
        /// The ability's FourCC code.
        ability_code: [u8; 4],
    },

    /// Unit ability with ground target (0x11).
    UnitAbilityGroundTarget {
        /// Flags byte.
        flags: u8,
        /// The ability's FourCC code.
        ability_code: [u8; 4],
        /// Target X coordinate.
        x: f32,
        /// Target Y coordinate.
        y: f32,
    },

    /// Unit ability with unit target (0x12).
    UnitAbilityUnitTarget {
        /// Flags byte.
        flags: u8,
        /// The ability's FourCC code.
        ability_code: [u8; 4],
        /// Target unit ID.
        target_unit: u32,
        /// Target X coordinate.
        x: f32,
        /// Target Y coordinate.
        y: f32,
    },

    /// Give or drop item (0x13).
    GiveDropItem {
        /// Flags byte.
        flags: u8,
        /// Item FourCC code.
        item_code: [u8; 4],
        /// Target unit ID.
        target_unit: u32,
        /// Target X coordinate.
        x: f32,
        /// Target Y coordinate.
        y: f32,
    },

    /// Unit ability with two position targets (0x14).
    UnitAbilityTwoTargets {
        /// Flags byte.
        flags: u8,
        /// The ability's FourCC code.
        ability_code: [u8; 4],
        /// First target X coordinate.
        x1: f32,
        /// First target Y coordinate.
        y1: f32,
        /// Second target X coordinate.
        x2: f32,
        /// Second target Y coordinate.
        y2: f32,
    },

    /// Select subgroup (0x19).
    SelectSubgroup {
        /// Item/subgroup index.
        item: u8,
        /// Object ID 1.
        object_id1: u32,
        /// Object ID 2.
        object_id2: u32,
    },

    /// Remove unit from queue (0x1E).
    RemoveFromQueue {
        /// Slot position in queue.
        slot: u8,
        /// Unit type ID.
        unit_id: u32,
    },

    /// Change ally options (0x50).
    ChangeAllyOptions {
        /// Slot number.
        slot: u8,
        /// Flags for ally settings.
        flags: u32,
    },

    /// Transfer resources (0x51).
    TransferResources {
        /// Target slot.
        slot: u8,
        /// Gold amount.
        gold: u32,
        /// Lumber amount.
        lumber: u32,
    },

    /// Minimap ping (0x68).
    MinimapPing {
        /// X coordinate.
        x: f32,
        /// Y coordinate.
        y: f32,
        /// Unknown data.
        unknown: u32,
    },

    /// Unknown action type - preserved for forward compatibility.
    Unknown {
        /// Raw action type byte.
        type_id: u8,
        /// Raw subcommand byte (if present).
        subcommand: Option<u8>,
        /// Raw action data.
        data: Vec<u8>,
    },
}

impl ActionType {
    /// Returns the name of this action type.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            ActionType::Selection(_) => "Selection",
            ActionType::Ability(_) => "Ability",
            ActionType::AbilityWithSelection(_) => "AbilityWithSelection",
            ActionType::InstantAbility(_) => "InstantAbility",
            ActionType::Movement(_) => "Movement",
            ActionType::Hotkey(_) => "Hotkey",
            ActionType::EscapeKey => "EscapeKey",
            ActionType::ItemAction { .. } => "ItemAction",
            ActionType::BasicCommand { .. } => "BasicCommand",
            ActionType::BuildTrain { .. } => "BuildTrain",
            ActionType::UnitAbilityNoTarget { .. } => "UnitAbilityNoTarget",
            ActionType::UnitAbilityGroundTarget { .. } => "UnitAbilityGroundTarget",
            ActionType::UnitAbilityUnitTarget { .. } => "UnitAbilityUnitTarget",
            ActionType::GiveDropItem { .. } => "GiveDropItem",
            ActionType::UnitAbilityTwoTargets { .. } => "UnitAbilityTwoTargets",
            ActionType::SelectSubgroup { .. } => "SelectSubgroup",
            ActionType::RemoveFromQueue { .. } => "RemoveFromQueue",
            ActionType::ChangeAllyOptions { .. } => "ChangeAllyOptions",
            ActionType::TransferResources { .. } => "TransferResources",
            ActionType::MinimapPing { .. } => "MinimapPing",
            ActionType::Unknown { .. } => "Unknown",
        }
    }

    /// Returns the raw action type byte for this action.
    #[must_use]
    pub fn type_byte(&self) -> u8 {
        match self {
            ActionType::Selection(_) => 0x16,
            ActionType::Ability(_) | ActionType::AbilityWithSelection(_) => 0x1A,
            ActionType::InstantAbility(_) => 0x0F,
            ActionType::Movement(_) => 0x00,
            ActionType::Hotkey(_) => 0x17,
            ActionType::EscapeKey => 0x18,
            ActionType::ItemAction { .. } => 0x1B,
            ActionType::BasicCommand { .. } => 0x1C,
            ActionType::BuildTrain { .. } => 0x1D,
            ActionType::UnitAbilityNoTarget { .. } => 0x10,
            ActionType::UnitAbilityGroundTarget { .. } => 0x11,
            ActionType::UnitAbilityUnitTarget { .. } => 0x12,
            ActionType::GiveDropItem { .. } => 0x13,
            ActionType::UnitAbilityTwoTargets { .. } => 0x14,
            ActionType::SelectSubgroup { .. } => 0x19,
            ActionType::RemoveFromQueue { .. } => 0x1E,
            ActionType::ChangeAllyOptions { .. } => 0x50,
            ActionType::TransferResources { .. } => 0x51,
            ActionType::MinimapPing { .. } => 0x68,
            ActionType::Unknown { type_id, .. } => *type_id,
        }
    }

    /// Returns `true` if this is an unknown action type.
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        matches!(self, ActionType::Unknown { .. })
    }
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionType::Selection(sel) => {
                write!(f, "Selection: {} unit(s), mode {}", sel.unit_count, sel.mode)
            }
            ActionType::Ability(ab) => {
                write!(f, "Ability: {}", ab.ability_code)
            }
            ActionType::AbilityWithSelection(ab) => {
                write!(
                    f,
                    "AbilityWithSelection: {} (with {} unit(s))",
                    ab.ability.ability_code, ab.selection.unit_count
                )
            }
            ActionType::InstantAbility(ab) => {
                write!(f, "InstantAbility: {}", ab.ability_code)
            }
            ActionType::Movement(mov) => {
                write!(
                    f,
                    "Movement: ({:.1}, {:.1}){}",
                    mov.x,
                    mov.y,
                    if mov.target_unit.is_some() {
                        " (target unit)"
                    } else {
                        ""
                    }
                )
            }
            ActionType::Hotkey(hk) => {
                write!(f, "Hotkey: group {} {:?}", hk.group, hk.operation)
            }
            ActionType::EscapeKey => {
                write!(f, "EscapeKey: cancel")
            }
            ActionType::ItemAction { item_id } => {
                write!(f, "ItemAction: 0x{item_id:08X}")
            }
            ActionType::BasicCommand { command_id } => {
                write!(f, "BasicCommand: 0x{command_id:08X}")
            }
            ActionType::BuildTrain { unit_code } => {
                let code_str = std::str::from_utf8(unit_code)
                    .unwrap_or("????")
                    .chars()
                    .rev()
                    .collect::<String>();
                write!(f, "BuildTrain: {code_str}")
            }
            ActionType::UnitAbilityNoTarget { ability_code, .. } => {
                let code_str = std::str::from_utf8(ability_code)
                    .unwrap_or("????")
                    .chars()
                    .rev()
                    .collect::<String>();
                write!(f, "UnitAbilityNoTarget: {code_str}")
            }
            ActionType::UnitAbilityGroundTarget {
                ability_code, x, y, ..
            } => {
                let code_str = std::str::from_utf8(ability_code)
                    .unwrap_or("????")
                    .chars()
                    .rev()
                    .collect::<String>();
                write!(f, "UnitAbilityGroundTarget: {code_str} at ({x:.1}, {y:.1})")
            }
            ActionType::UnitAbilityUnitTarget {
                ability_code,
                target_unit,
                ..
            } => {
                let code_str = std::str::from_utf8(ability_code)
                    .unwrap_or("????")
                    .chars()
                    .rev()
                    .collect::<String>();
                write!(
                    f,
                    "UnitAbilityUnitTarget: {code_str} -> 0x{target_unit:08X}"
                )
            }
            ActionType::GiveDropItem {
                item_code,
                target_unit,
                ..
            } => {
                let code_str = std::str::from_utf8(item_code)
                    .unwrap_or("????")
                    .chars()
                    .rev()
                    .collect::<String>();
                write!(f, "GiveDropItem: {code_str} -> 0x{target_unit:08X}")
            }
            ActionType::UnitAbilityTwoTargets {
                ability_code,
                x1,
                y1,
                x2,
                y2,
                ..
            } => {
                let code_str = std::str::from_utf8(ability_code)
                    .unwrap_or("????")
                    .chars()
                    .rev()
                    .collect::<String>();
                write!(
                    f,
                    "UnitAbilityTwoTargets: {code_str} ({x1:.1}, {y1:.1}) -> ({x2:.1}, {y2:.1})"
                )
            }
            ActionType::SelectSubgroup {
                item,
                object_id1,
                object_id2,
            } => {
                write!(
                    f,
                    "SelectSubgroup: item {} (0x{:08X}, 0x{:08X})",
                    item, object_id1, object_id2
                )
            }
            ActionType::RemoveFromQueue { slot, unit_id } => {
                write!(f, "RemoveFromQueue: slot {} unit 0x{:08X}", slot, unit_id)
            }
            ActionType::ChangeAllyOptions { slot, flags } => {
                write!(f, "ChangeAllyOptions: slot {} flags 0x{:08X}", slot, flags)
            }
            ActionType::TransferResources { slot, gold, lumber } => {
                write!(
                    f,
                    "TransferResources: slot {} gold {} lumber {}",
                    slot, gold, lumber
                )
            }
            ActionType::MinimapPing { x, y, .. } => {
                write!(f, "MinimapPing: ({x:.1}, {y:.1})")
            }
            ActionType::Unknown {
                type_id,
                subcommand,
                data,
            } => {
                let sub_str = subcommand
                    .map(|s| format!(" 0x{s:02X}"))
                    .unwrap_or_default();
                write!(
                    f,
                    "Unknown: 0x{:02X}{} ({} bytes)",
                    type_id,
                    sub_str,
                    data.len()
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_type_name() {
        let sel = ActionType::Selection(SelectionAction {
            unit_count: 1,
            mode: 1,
            flags: 0,
            unit_ids: vec![0x1234],
        });
        assert_eq!(sel.type_name(), "Selection");

        let unknown = ActionType::Unknown {
            type_id: 0xFF,
            subcommand: None,
            data: vec![],
        };
        assert_eq!(unknown.type_name(), "Unknown");
        assert!(unknown.is_unknown());
    }

    #[test]
    fn test_action_display() {
        let action = Action::new(
            3,
            ActionType::Selection(SelectionAction {
                unit_count: 2,
                mode: 1,
                flags: 0,
                unit_ids: vec![0x1234, 0x5678],
            }),
            1000,
        );

        let display = format!("{action}");
        assert!(display.contains("P3"));
        assert!(display.contains("1000ms"));
        assert!(display.contains("Selection"));
    }
}
