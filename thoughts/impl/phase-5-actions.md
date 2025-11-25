# Phase 5: Action Parsing Implementation Report

**Date**: 2025-11-25
**Implementer**: Cody (Implementation Agent)
**Status**: Complete

## Summary

Successfully implemented action parsing for W3G replay files, enabling extraction of player commands from TimeFrame records. The implementation supports:

- Selection actions (0x16)
- Ability actions (0x1A 0x19, 0x1A 0x00)
- Instant ability actions (0x0F 0x00)
- Movement actions (0x00 0x0D)
- Hotkey actions (0x17)
- Unknown action handling with forward compatibility

## Files Created

| File | Purpose |
|------|---------|
| `src/actions/mod.rs` | Module root with exports |
| `src/actions/types.rs` | Core `Action` and `ActionType` types |
| `src/actions/selection.rs` | Selection action (0x16) parsing |
| `src/actions/ability.rs` | Ability actions (0x1A, 0x0F) and `AbilityCode` |
| `src/actions/movement.rs` | Movement action (0x00 0x0D) and `Position` |
| `src/actions/hotkey.rs` | Hotkey action (0x17) parsing |
| `src/actions/parser.rs` | `ActionIterator` and `ActionStatistics` |
| `tests/action_integration.rs` | Integration tests for action parsing |

## Data Structures Implemented

### Core Types

```rust
pub struct Action {
    pub player_id: u8,
    pub action_type: ActionType,
    pub timestamp_ms: u32,
}

pub enum ActionType {
    Selection(SelectionAction),
    Ability(AbilityAction),
    AbilityWithSelection(AbilityWithSelectionAction),
    InstantAbility(InstantAbilityAction),
    Movement(MovementAction),
    Hotkey(HotkeyAction),
    Unknown { type_id: u8, subcommand: Option<u8>, data: Vec<u8> },
}
```

### AbilityCode

```rust
#[derive(Clone, Copy)]
pub struct AbilityCode([u8; 4]);

impl AbilityCode {
    pub fn from_raw(bytes: [u8; 4]) -> Self;
    pub fn as_string(&self) -> String;  // Reverses bytes for display
    pub fn race(&self) -> Option<Race>;
    pub fn is_hero_ability(&self) -> bool;
}
```

### Position

```rust
pub struct Position {
    pub x: f32,  // IEEE 754 float
    pub y: f32,
}

impl Position {
    pub fn is_valid(&self) -> bool;
    pub fn distance_to(&self, other: &Position) -> f32;
}
```

## Action Sizes

| Action | Size | Format |
|--------|------|--------|
| Selection (0x16) | 4 + 8*count | marker + count + mode + flags + unit_ids |
| Direct Ability (0x1A 0x19) | 14 bytes | markers + FourCC + target |
| Ability with Selection (0x1A 0x00) | variable | markers + selection block + ability |
| Instant Ability (0x0F 0x00) | 18 bytes | markers + flags + unknown + FourCC + target |
| Movement (0x00 0x0D) | 28 bytes | markers + flags + target + x + y + extra |
| Hotkey (0x17) | 3+ bytes | marker + group + operation |

## Sample Ability Codes Found

From Rex's analysis of replays:

| Raw (hex) | Canonical | Race | Description |
|-----------|-----------|------|-------------|
| 77 6F 74 68 | htow | Human | Town Hall |
| 70 73 77 65 | ewsp | Night Elf | Wisp |
| 67 6D 61 48 | Hamg | Human | Archmage (Hero) |
| 6D 65 64 45 | Edem | Night Elf | Demon Hunter (Hero) |
| 65 72 64 55 | Udre | Undead | Dread Lord (Hero) |
| 6D 6F 61 65 | eaom | Night Elf | Eat Tree / Ancient of War |

## Test Results

**Unit Tests**: 159 passed
- 39 action module tests
- 120 other module tests

**Integration Tests**: 53 passed
- 13 action-specific tests
- 40 other integration tests

**Clippy**: No warnings on library code

## Deviations from Plan

### 1. TimeFrame Integration Limitations

The `TimeFrame.actions()` method was implemented, but TimeFrame iteration has known issues with:
- Type B Classic replays (build >= 10000) with chat messages
- Chat message parsing doesn't properly handle all metadata fields

This results in early termination of TimeFrame iteration for some replays, but the action parsing logic itself is correct.

### 2. Unknown Action Type (0x11)

Rex's analysis identified action type 0x11 as very frequent (5033 occurrences in replay_100000), possibly heartbeat/sync. This is handled as an Unknown action type for forward compatibility.

### 3. Simplified Hotkey Parsing

Hotkey actions may have additional data beyond the 3-byte minimum (e.g., unit IDs for assign operations). The current implementation parses only the base 3 bytes. Future work could extend this.

## API Usage Example

```rust
use w3g_parser::{GameRecord, ActionType, decompress, Header};

// Parse replay
let data = std::fs::read("replay.w3g")?;
let header = Header::parse(&data)?;
let decompressed = decompress(&data, &header)?;
let game = GameRecord::parse(&decompressed)?;

// Iterate actions
for frame_result in game.timeframes(&decompressed) {
    let frame = frame_result?;
    for action_result in frame.actions() {
        let action = action_result?;
        match &action.action_type {
            ActionType::Ability(ab) => {
                println!("Player {} used ability: {}",
                    action.player_id,
                    ab.ability_code.as_string());
            }
            ActionType::Movement(mov) => {
                println!("Player {} moved to ({}, {})",
                    action.player_id, mov.x, mov.y);
            }
            _ => {}
        }
    }
}
```

## Known Limitations

1. **TimeFrame iteration** may stop early for some replay types due to chat message parsing issues
2. **Action type 0x11** is unimplemented (stored as Unknown)
3. **Selection modes** (1-10) are not fully documented
4. **Hotkey operations** beyond basic assign/select are stored as Unknown

## Future Work

1. Improve chat message parsing for Type B Classic replays
2. Implement action type 0x11 (likely heartbeat/sync)
3. Add FourCC ability code registry for human-readable names
4. Extend hotkey parsing for full operation support
5. Add coordinate validation against map bounds

## Conclusion

Phase 5 successfully implements the core action parsing framework. All major action types (selection, ability, movement, hotkey) are supported with proper byte parsing and type-safe structures. The implementation handles unknown actions gracefully and provides comprehensive statistics tracking.

The action parsing logic is verified correct through unit tests. Integration testing is limited by TimeFrame iteration issues in the underlying records module, which should be addressed in a future phase.
