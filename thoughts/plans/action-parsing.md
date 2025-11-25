# Plan: Phase 5 - Action Parsing

## Metadata
- **Agent**: Archie (Planning Agent)
- **Date**: 2025-11-25
- **Research Used**:
  - `thoughts/research/action-analysis.md` - Rex's action data structure analysis
  - `FORMAT.md` - Updated format specification with action structure
  - `thoughts/plans/decompressed-parsing.md` - Previous plan template (Phase 4)
- **Status**: Draft
- **Prerequisites**: Phases 1-4 validated (160 tests passing)

## Overview

This plan implements parsing of individual player actions within TimeFrame records. Rex's binary analysis has identified the structure of action blocks, including:
- Selection commands (unit selection)
- Ability commands (with FourCC codes)
- Movement commands (with IEEE 754 coordinates)
- Hotkey operations (control groups)

This phase transforms raw action bytes into structured, queryable action data, enabling game analysis, APM calculation, and replay visualization.

## Prerequisites

Before starting Phase 5:
- [x] Phases 1-4 complete and validated (160 tests passing)
- [x] TimeFrame iterator yields raw action data bytes
- [x] Rex's action analysis identifies action block structure
- [x] FORMAT.md documents action types and their formats

## Architecture Decision: Module Organization

Action parsing will be organized under a new `actions` module within the existing structure:

```
w3g-parser/src/
  actions/
    mod.rs           # Public API, ActionIterator, re-exports
    parser.rs        # Core action parsing logic
    types.rs         # Action enum and subtypes
    selection.rs     # Selection action (0x16)
    ability.rs       # Ability actions (0x1A, 0x0F)
    movement.rs      # Move/Attack action (0x00 0x0D)
    hotkey.rs        # Hotkey operations (0x17)
    fourcc.rs        # FourCC code handling and registry
```

---

## Phase 5A: Action Block Parsing Framework

### Goal

Create the core framework for parsing action blocks within TimeFrame data, extracting player IDs and dispatching to action-specific parsers.

### Research Reference

From Rex's analysis (`action-analysis.md`):
```
Each action block within a TimeFrame follows this structure:

[Player ID: 1 byte] [Action Type: 1 byte] [Subcommand: 1 byte] [Data: variable]

Player ID Range: 1-15 (0x01-0x0F)

Evidence:
Frame 17 from replay_5001:
04 1A 00 16 01 01 00 3B 3A 00 00 3B 3A 00 00 1A 19 77 6F 74 68 ...
^^ Player 4

Frame 30 from replay_5001:
03 00 0D 00 FF FF FF FF FF FF FF FF 00 00 B0 C5 00 00 60 45 ...
^^ Player 3
```

### Files to Create/Modify

- `src/actions/mod.rs` - Module root with re-exports and ActionIterator
- `src/actions/types.rs` - Action enum and base structures
- `src/actions/parser.rs` - Core parsing dispatch logic
- `src/lib.rs` - Add `pub mod actions;`

### Data Structures

```rust
/// A parsed action from a TimeFrame.
///
/// Actions represent individual commands issued by players during gameplay.
/// Each action has a player ID, timestamp, and action-specific data.
#[derive(Debug, Clone)]
pub struct Action {
    /// Player ID who issued this action (1-15)
    pub player_id: u8,

    /// Action type with parsed data
    pub action_type: ActionType,

    /// Timestamp in milliseconds from game start
    /// (inherited from containing TimeFrame)
    pub timestamp_ms: u32,
}

/// Enumeration of all known action types.
///
/// Each variant contains the parsed data specific to that action type.
#[derive(Debug, Clone)]
pub enum ActionType {
    /// Unit selection (0x16)
    Selection(SelectionAction),

    /// Ability use with selection (0x1A 0x00)
    AbilityWithSelection(AbilityWithSelectionAction),

    /// Direct ability use (0x1A 0x19)
    Ability(AbilityAction),

    /// Instant ability (0x0F 0x00)
    InstantAbility(InstantAbilityAction),

    /// Move/Attack command (0x00 0x0D)
    Movement(MovementAction),

    /// Control group hotkey (0x17)
    Hotkey(HotkeyAction),

    /// Pause/Resume game (0x10, 0x11)
    GameControl(GameControlAction),

    /// Unknown action type - preserved for forward compatibility
    Unknown {
        /// Raw action type byte
        type_id: u8,
        /// Raw subcommand byte (if present)
        subcommand: Option<u8>,
        /// Raw action data
        data: Vec<u8>,
    },
}

/// Context for parsing actions within a TimeFrame.
#[derive(Debug)]
pub struct ActionContext {
    /// Timestamp from the containing TimeFrame
    pub timestamp_ms: u32,

    /// Frame number for debugging
    pub frame_number: u32,
}

/// Iterator over actions within a TimeFrame's action data.
pub struct ActionIterator<'a> {
    /// Raw action data bytes
    data: &'a [u8],

    /// Current offset within data
    offset: usize,

    /// Context from containing TimeFrame
    context: ActionContext,
}
```

### Implementation Steps

1. **Create module structure** (`src/actions/mod.rs`)
   - Declare submodules: `types`, `parser`, `selection`, `ability`, `movement`, `hotkey`, `fourcc`
   - Re-export public types: `Action`, `ActionType`, `ActionIterator`
   - Add comprehensive module documentation

2. **Define action types** (`src/actions/types.rs`)
   - Define `Action` struct with player_id, action_type, timestamp_ms
   - Define `ActionType` enum with all variants
   - Implement `Display` for human-readable action descriptions
   - Implement `ActionType::type_name()` for action type names

3. **Implement ActionIterator** (`src/actions/parser.rs`)
   - `ActionIterator::new(data: &[u8], context: ActionContext)`
   - Implement `Iterator<Item = Result<Action, ParserError>>`
   - Dispatch based on action type byte to specific parsers
   - Track bytes consumed for offset advancement
   - Handle multi-player action frames (concatenated actions)

4. **Add to TimeFrame** (`src/records/timeframe.rs`)
   - Add `TimeFrame::actions(&self) -> ActionIterator`
   - Pass timestamp context to ActionIterator

5. **Update lib.rs**
   - Add `pub mod actions;`
   - Re-export key types

### Action Type Detection Logic

```rust
fn parse_next_action(data: &[u8], offset: usize, ctx: &ActionContext) -> Result<(Action, usize)> {
    if offset >= data.len() {
        return Err(ParserError::EndOfData);
    }

    let player_id = data[offset];
    if player_id == 0 || player_id > 15 {
        return Err(ParserError::InvalidPlayerId { value: player_id, offset });
    }

    let action_type = data.get(offset + 1).ok_or(ParserError::UnexpectedEof)?;
    let subcommand = data.get(offset + 2).copied();

    match (action_type, subcommand) {
        (0x00, Some(0x0D)) => parse_movement(data, offset, ctx),
        (0x0F, Some(0x00)) => parse_instant_ability(data, offset, ctx),
        (0x10, _) | (0x11, _) => parse_game_control(data, offset, ctx),
        (0x16, _) => parse_selection(data, offset, ctx),
        (0x17, _) => parse_hotkey(data, offset, ctx),
        (0x1A, Some(0x00)) => parse_ability_with_selection(data, offset, ctx),
        (0x1A, Some(0x19)) => parse_ability(data, offset, ctx),
        _ => parse_unknown(data, offset, ctx),
    }
}
```

### Validation Criteria

- [ ] ActionIterator compiles and basic structure in place
- [ ] Can iterate over TimeFrame action data without panic
- [ ] Player IDs extracted correctly (range 1-15)
- [ ] Action type bytes correctly dispatched
- [ ] Unknown actions captured with raw data preserved
- [ ] Offset tracking correctly advances through action data

### Test Cases

```rust
#[test]
fn test_action_iterator_basic() {
    let data = decompress_replay("replays/replay_5000.w3g");
    let game = GameRecord::parse(&data).unwrap();

    let mut total_actions = 0;
    for frame in game.timeframes(&data) {
        let frame = frame.unwrap();
        for action in frame.actions() {
            let action = action.unwrap();
            assert!(action.player_id >= 1 && action.player_id <= 15);
            total_actions += 1;
        }
    }

    assert!(total_actions > 0, "Expected some actions in replay");
}

#[test]
fn test_action_player_ids() {
    let data = decompress_replay("replays/replay_5001.w3g");
    let game = GameRecord::parse(&data).unwrap();

    let mut seen_players = std::collections::HashSet::new();
    for frame in game.timeframes(&data).take(100) {
        for action in frame.unwrap().actions() {
            if let Ok(a) = action {
                seen_players.insert(a.player_id);
            }
        }
    }

    // Should see multiple players
    assert!(seen_players.len() >= 2, "Expected multiple players");
}
```

### Estimated Complexity

**Medium** - Core framework setup with dispatch logic. Main complexity is handling variable-length actions and multi-player frames.

---

## Phase 5B: Selection Actions (0x16)

### Goal

Parse unit selection commands, extracting selected unit IDs and selection modes.

### Research Reference

From Rex's analysis:
```
Selection Block Format (0x16):
16 [count: 1] [mode: 1] [flags: 1] [unit_ids: 8*count]

Unit ID Format: Each unit uses 8 bytes (4-byte object ID appearing twice).

Evidence:
Single unit selection:
16 01 01 00 3B 3A 00 00 3B 3A 00 00
   ^^ 1 unit
      ^^ Mode 1
         ^^ Flags 0
            ^^^^^^^^^^^^^^^^^^^^^^ Unit 0x00003A3B (8 bytes)

Multiple units:
16 02 05 00 23 3B 00 00 26 3B 00 00 39 3B 00 00 3C 3B 00 00 ...
   ^^ 2 units
      ^^ Mode 5

Selection Modes Observed: 1-10
```

### Files to Create/Modify

- `src/actions/selection.rs` - Selection action parser

### Data Structures

```rust
/// A unit selection action.
///
/// Selection actions specify which units are currently selected,
/// used as context for subsequent ability and movement commands.
#[derive(Debug, Clone)]
pub struct SelectionAction {
    /// Number of units selected (1-12 typical)
    pub unit_count: u8,

    /// Selection mode (1-10 observed)
    /// - Mode 1: Most common, single selection
    /// - Mode 5: Multi-unit selection
    /// - Other modes: Under investigation
    pub mode: u8,

    /// Flags byte (usually 0x00)
    pub flags: u8,

    /// Object IDs of selected units
    /// Each ID is 4 bytes, stored twice in 8-byte blocks
    pub unit_ids: Vec<u32>,
}

/// Selection mode interpretation (from analysis).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// Replace current selection entirely
    Replace,
    /// Add to current selection
    Add,
    /// Remove from current selection
    Remove,
    /// Unknown mode
    Unknown(u8),
}

impl From<u8> for SelectionMode {
    fn from(value: u8) -> Self {
        match value {
            1 => SelectionMode::Replace,
            // Other mappings TBD based on further analysis
            _ => SelectionMode::Unknown(value),
        }
    }
}
```

### Implementation Steps

1. **Implement `SelectionAction::parse()`** (`src/actions/selection.rs`)
   - Validate marker byte (0x16) at offset
   - Read unit count at offset + 1
   - Read mode at offset + 2
   - Read flags at offset + 3
   - Read `unit_count * 8` bytes for unit IDs
   - Extract 4-byte unit IDs from 8-byte blocks (skip duplicate)
   - Return `(SelectionAction, bytes_consumed)`

2. **Validate unit ID extraction**
   - Extract first 4 bytes of each 8-byte block
   - Verify second 4 bytes match (or document pattern if different)

3. **Add selection mode helpers**
   - `SelectionAction::selection_mode() -> SelectionMode`
   - `SelectionAction::is_multi_select() -> bool`

### Implementation Note: 8-Byte Unit ID Blocks

Rex's analysis shows units stored as 8-byte blocks with the 4-byte ID appearing twice. This may be:
- Object ID + Counter (for duplicate detection)
- Object ID + Reference ID
- Simple redundancy

For now, extract the first 4 bytes as the unit ID and document if the second differs.

```rust
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
```

### Validation Criteria

- [ ] Parse selection actions from all 27 replays
- [ ] Unit counts match expected range (1-12)
- [ ] Unit IDs are non-zero
- [ ] Selection modes in observed range (1-10)
- [ ] Bytes consumed calculation is correct
- [ ] No panics on edge cases (0 units, max units)

### Test Cases

```rust
#[test]
fn test_selection_single_unit() {
    // From Rex's analysis
    let data: &[u8] = &[
        0x04, // Player 4
        0x16, 0x01, 0x01, 0x00, // Selection: 1 unit, mode 1, flags 0
        0x3B, 0x3A, 0x00, 0x00, 0x3B, 0x3A, 0x00, 0x00, // Unit ID 0x00003A3B
    ];

    let ctx = ActionContext { timestamp_ms: 1000, frame_number: 1 };
    let (action, consumed) = parse_selection(data, 0, &ctx).unwrap();

    assert_eq!(action.player_id, 4);
    if let ActionType::Selection(sel) = action.action_type {
        assert_eq!(sel.unit_count, 1);
        assert_eq!(sel.mode, 1);
        assert_eq!(sel.unit_ids, vec![0x00003A3B]);
    } else {
        panic!("Expected Selection action");
    }
}

#[test]
fn test_selection_multiple_units() {
    // Verify multi-unit selection parsing
    let data: &[u8] = &[
        0x03, // Player 3
        0x16, 0x02, 0x05, 0x00, // Selection: 2 units, mode 5
        0x23, 0x3B, 0x00, 0x00, 0x26, 0x3B, 0x00, 0x00, // Unit 1
        0x39, 0x3B, 0x00, 0x00, 0x3C, 0x3B, 0x00, 0x00, // Unit 2
    ];

    let ctx = ActionContext { timestamp_ms: 2000, frame_number: 2 };
    let (action, _) = parse_selection(data, 0, &ctx).unwrap();

    if let ActionType::Selection(sel) = action.action_type {
        assert_eq!(sel.unit_count, 2);
        assert_eq!(sel.mode, 5);
        assert_eq!(sel.unit_ids.len(), 2);
    }
}
```

### Estimated Complexity

**Low-Medium** - Straightforward fixed-size parsing once unit count is known. Edge case: very large selections (12 units = 96 bytes).

---

## Phase 5C: Ability Actions (0x1A, 0x0F)

### Goal

Parse ability commands including FourCC ability codes and optional targets.

### Research Reference

From Rex's analysis:
```
Direct Ability (0x1A 0x19):
1A 19 [ability: 4 bytes FourCC] [target: 8 bytes]

Evidence:
1A 19 77 6F 74 68 3B 3A 00 00 3B 3A 00 00
      ^^^^^^^^^^^ "woth" -> reversed "htow" = Town Hall
                  ^^^^^^^^^^^^^^^^^^^^^^ Target unit(s)

Ability With Selection (0x1A 0x00):
1A 00 16 [selection block...] 1A 19 [ability code] [target]

Instant Ability (0x0F 0x00):
0F 00 10 42 00 [ability: 4 bytes] FF FF FF FF FF FF FF FF
               ^^^^^^^^^^^ FourCC ability code
                           ^^^^^^^^^^^^^^^^^^^^^^^^ No target (padding)
```

### Files to Create/Modify

- `src/actions/ability.rs` - Ability action parsers
- `src/actions/fourcc.rs` - FourCC code handling

### Data Structures

```rust
/// A FourCC ability code.
///
/// Ability codes are 4-byte identifiers stored in reverse byte order.
/// For example, "htow" (Town Hall) is stored as "woth".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AbilityCode([u8; 4]);

impl AbilityCode {
    /// Create from raw bytes (as stored in replay).
    pub fn from_raw(bytes: [u8; 4]) -> Self {
        Self(bytes)
    }

    /// Get the reversed/canonical form as a string.
    pub fn as_string(&self) -> String {
        let reversed = [self.0[3], self.0[2], self.0[1], self.0[0]];
        String::from_utf8_lossy(&reversed).to_string()
    }

    /// Get the raw bytes as stored.
    pub fn raw_bytes(&self) -> [u8; 4] {
        self.0
    }

    /// Get race prefix if identifiable.
    pub fn race(&self) -> Option<Race> {
        // Reversed first byte determines race
        match self.0[3] {
            b'h' => Some(Race::Human),
            b'H' => Some(Race::Human),      // Hero
            b'o' => Some(Race::Orc),
            b'O' => Some(Race::Orc),        // Hero
            b'u' => Some(Race::Undead),
            b'U' => Some(Race::Undead),     // Hero
            b'e' => Some(Race::NightElf),
            b'E' => Some(Race::NightElf),   // Hero
            b'n' | b'N' => Some(Race::Neutral),
            _ => None,
        }
    }
}

impl std::fmt::Display for AbilityCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

/// A direct ability use action (0x1A 0x19).
#[derive(Debug, Clone)]
pub struct AbilityAction {
    /// The ability FourCC code
    pub ability_code: AbilityCode,

    /// Target unit ID (if targeting a unit)
    pub target_unit: Option<u32>,

    /// Target position (if targeting ground)
    pub target_position: Option<Position>,
}

/// An ability with preceding selection (0x1A 0x00).
#[derive(Debug, Clone)]
pub struct AbilityWithSelectionAction {
    /// The selection that precedes this ability
    pub selection: SelectionAction,

    /// The ability being used
    pub ability: AbilityAction,
}

/// An instant ability (0x0F 0x00).
#[derive(Debug, Clone)]
pub struct InstantAbilityAction {
    /// Unknown flags (2 bytes)
    pub flags: u16,

    /// The ability FourCC code
    pub ability_code: AbilityCode,

    /// Whether this has a target (0xFF padding means no target)
    pub has_target: bool,
}

/// Map position in game coordinates.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
```

### Implementation Steps

1. **Implement FourCC handling** (`src/actions/fourcc.rs`)
   - `AbilityCode::from_raw()` - Create from 4 bytes
   - `AbilityCode::as_string()` - Get reversed/readable form
   - `AbilityCode::race()` - Identify race from prefix
   - Implement `Display`, `Debug`, `Hash`, `Eq`

2. **Implement direct ability parser** (`src/actions/ability.rs`)
   - Parse 0x1A 0x19 format: 2 + 4 + 8 = 14 bytes minimum
   - Extract FourCC from bytes 2-5
   - Parse target from bytes 6-13 (unit ID or check for 0xFF padding)
   - Return `(AbilityAction, bytes_consumed)`

3. **Implement ability with selection parser**
   - Parse 0x1A 0x00 format
   - Detect and parse embedded 0x16 selection block
   - Continue to parse 0x1A 0x19 ability after selection
   - Handle nested structure correctly

4. **Implement instant ability parser**
   - Parse 0x0F 0x00 format: 2 + 2 + 2 + 4 + 8 = 18 bytes
   - Extract flags (bytes 2-3)
   - Extract unknown (bytes 4-5)
   - Extract FourCC (bytes 6-9)
   - Check for target vs 0xFF padding

5. **Build known ability registry** (optional enhancement)
   - Create static lookup table for common abilities
   - `AbilityCode::description() -> Option<&'static str>`

### FourCC Code Examples from Analysis

| Stored | Reversed | Race | Description |
|--------|----------|------|-------------|
| woth | htow | Human | Town Hall |
| eekh | hkee | Human | Keep |
| gmaH | Hamg | Human | Archmage |
| medE | Edem | Night Elf | Demon Hunter |
| lote | etol | Night Elf | Tree of Life |
| pswe | ewsp | Night Elf | Wisp |
| erdU | Udre | Undead | Dread Lord |
| sgnN | Nngs | Neutral | Naga Siren |

### Validation Criteria

- [ ] Parse ability actions from all 27 replays
- [ ] FourCC codes readable as 4-character strings
- [ ] Race detection works for known prefixes
- [ ] Target parsing handles both unit and ground targets
- [ ] Instant ability parsing handles 0xFF padding
- [ ] Ability with selection correctly parses nested structure

### Test Cases

```rust
#[test]
fn test_ability_fourcc_parsing() {
    let raw = [0x77, 0x6F, 0x74, 0x68]; // "woth"
    let code = AbilityCode::from_raw(raw);

    assert_eq!(code.as_string(), "htow");
    assert_eq!(code.race(), Some(Race::Human));
}

#[test]
fn test_direct_ability_action() {
    // From Rex's analysis
    let data: &[u8] = &[
        0x04, // Player 4
        0x1A, 0x19, // Ability command
        0x77, 0x6F, 0x74, 0x68, // "woth" = htow (Town Hall)
        0x3B, 0x3A, 0x00, 0x00, 0x3B, 0x3A, 0x00, 0x00, // Target
    ];

    let ctx = ActionContext { timestamp_ms: 1000, frame_number: 1 };
    let (action, _) = parse_ability(data, 0, &ctx).unwrap();

    if let ActionType::Ability(ab) = action.action_type {
        assert_eq!(ab.ability_code.as_string(), "htow");
    }
}

#[test]
fn test_ability_code_registry() {
    // Verify ability codes from replays
    let data = decompress_replay("replays/replay_5000.w3g");
    let game = GameRecord::parse(&data).unwrap();

    let mut ability_counts: HashMap<AbilityCode, u32> = HashMap::new();

    for frame in game.timeframes(&data) {
        for action in frame.unwrap().actions() {
            if let Ok(Action { action_type: ActionType::Ability(ab), .. }) = action {
                *ability_counts.entry(ab.ability_code).or_insert(0) += 1;
            }
        }
    }

    // Should find Night Elf abilities in replay_5000
    assert!(!ability_counts.is_empty());
}
```

### Estimated Complexity

**Medium** - FourCC handling straightforward, but nested ability-with-selection requires careful offset tracking.

---

## Phase 5D: Movement Actions (0x00 0x0D)

### Goal

Parse move and attack commands, extracting IEEE 754 float coordinates and target information.

### Research Reference

From Rex's analysis:
```
Move/Attack Command (0x00 0x0D):
00 0D [flags: 2] [target: 8 bytes] [x: 4 float] [y: 4 float] [extra: 8]

Total size: 28 bytes

Coordinate System:
- IEEE 754 single-precision floats
- Little-endian byte order
- Range: approximately -10000 to +10000
- Negative X = West, Positive X = East
- Negative Y = South, Positive Y = North

Evidence:
03 00 0D 00 FF FF FF FF FF FF FF FF 00 00 B0 C5 00 00 60 45
                                    ^^^^^^^^^^^ X = -5632.0
                                                ^^^^^^^^^^^ Y = 3584.0
```

### Files to Create/Modify

- `src/actions/movement.rs` - Movement action parser

### Data Structures

```rust
/// A move or attack command.
///
/// Movement actions represent right-click commands (move/attack)
/// with target coordinates and optional target unit.
#[derive(Debug, Clone)]
pub struct MovementAction {
    /// Command flags (meaning under investigation)
    pub flags: u16,

    /// Target unit ID (if right-clicking on a unit)
    /// Will be 0xFFFFFFFF if targeting ground
    pub target_unit: Option<u32>,

    /// X coordinate (IEEE 754 float, little-endian)
    /// Range: approximately -10000 to +10000
    pub x: f32,

    /// Y coordinate (IEEE 754 float, little-endian)
    /// Range: approximately -10000 to +10000
    pub y: f32,

    /// Additional data (8 bytes, purpose under investigation)
    /// May contain item ID or secondary target
    pub extra_data: [u8; 8],
}

impl MovementAction {
    /// Check if this is a ground-target command (no unit target).
    pub fn is_ground_target(&self) -> bool {
        self.target_unit.is_none()
    }

    /// Get the target position as a Position struct.
    pub fn position(&self) -> Position {
        Position { x: self.x, y: self.y }
    }

    /// Check if coordinates are within valid map range.
    pub fn is_valid_position(&self) -> bool {
        const MAP_BOUND: f32 = 15000.0;
        self.x.is_finite() && self.y.is_finite()
            && self.x.abs() < MAP_BOUND && self.y.abs() < MAP_BOUND
    }
}
```

### Implementation Steps

1. **Implement IEEE 754 float parsing**
   - Use `f32::from_le_bytes()` for coordinate extraction
   - Validate floats are finite (not NaN or Infinity)
   - Check for reasonable map coordinate range

2. **Implement `MovementAction::parse()`** (`src/actions/movement.rs`)
   - Validate action type (0x00) and subcommand (0x0D)
   - Read flags (bytes 2-3, u16 LE)
   - Read target unit (bytes 4-11, check for 0xFF padding)
   - Read X coordinate (bytes 12-15, f32 LE)
   - Read Y coordinate (bytes 16-19, f32 LE)
   - Read extra data (bytes 20-27)
   - Total: 28 bytes consumed

3. **Target unit detection**
   - If bytes 4-11 are all 0xFF, target is ground (no unit)
   - Otherwise, extract unit ID from bytes 4-7 (same 8-byte format as selection)

4. **Add coordinate helpers**
   - `MovementAction::position() -> Position`
   - `MovementAction::distance_to(other: &Position) -> f32`
   - `MovementAction::is_attack() -> bool` (based on flags?)

### Coordinate Parsing

```rust
fn parse_movement(data: &[u8], offset: usize, ctx: &ActionContext) -> Result<(Action, usize)> {
    const SIZE: usize = 28;

    if data.len() < offset + SIZE {
        return Err(ParserError::UnexpectedEof);
    }

    let player_id = data[offset];
    let flags = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);

    // Check for target unit vs ground
    let target_bytes = &data[offset + 4..offset + 12];
    let target_unit = if target_bytes.iter().all(|&b| b == 0xFF) {
        None
    } else {
        Some(u32::from_le_bytes([
            target_bytes[0], target_bytes[1], target_bytes[2], target_bytes[3]
        ]))
    };

    // Parse IEEE 754 floats (little-endian)
    let x = f32::from_le_bytes([
        data[offset + 12], data[offset + 13], data[offset + 14], data[offset + 15]
    ]);
    let y = f32::from_le_bytes([
        data[offset + 16], data[offset + 17], data[offset + 18], data[offset + 19]
    ]);

    let mut extra_data = [0u8; 8];
    extra_data.copy_from_slice(&data[offset + 20..offset + 28]);

    Ok((
        Action {
            player_id,
            action_type: ActionType::Movement(MovementAction {
                flags,
                target_unit,
                x,
                y,
                extra_data,
            }),
            timestamp_ms: ctx.timestamp_ms,
        },
        SIZE,
    ))
}
```

### Validation Criteria

- [ ] Parse movement actions from all 27 replays
- [ ] Coordinates are finite floats (not NaN/Infinity)
- [ ] Coordinate values in reasonable range (-15000 to +15000)
- [ ] Target unit detection (0xFF vs unit ID) correct
- [ ] Bytes consumed always 28 for 0x00 0x0D actions
- [ ] Position helper methods work correctly

### Test Cases

```rust
#[test]
fn test_movement_ground_target() {
    // From Rex's analysis
    let data: &[u8] = &[
        0x03, // Player 3
        0x00, 0x0D, 0x00, // Move command
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // No target (ground)
        0x00, 0x00, 0xB0, 0xC5, // X = -5632.0
        0x00, 0x00, 0x60, 0x45, // Y = 3584.0
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Extra
    ];

    let ctx = ActionContext { timestamp_ms: 1000, frame_number: 1 };
    let (action, consumed) = parse_movement(data, 0, &ctx).unwrap();

    assert_eq!(consumed, 28);
    assert_eq!(action.player_id, 3);

    if let ActionType::Movement(mov) = action.action_type {
        assert!(mov.is_ground_target());
        assert!((mov.x - (-5632.0)).abs() < 0.1);
        assert!((mov.y - 3584.0).abs() < 0.1);
    }
}

#[test]
fn test_movement_coordinate_range() {
    let data = decompress_replay("replays/replay_100000.w3g");
    let game = GameRecord::parse(&data).unwrap();

    for frame in game.timeframes(&data) {
        for action in frame.unwrap().actions() {
            if let Ok(Action { action_type: ActionType::Movement(mov), .. }) = action {
                assert!(mov.is_valid_position(),
                    "Invalid coordinates: ({}, {})", mov.x, mov.y);
            }
        }
    }
}
```

### Estimated Complexity

**Low-Medium** - Fixed-size format with straightforward IEEE 754 parsing. Main validation is ensuring coordinate sanity.

---

## Phase 5E: Hotkey Actions (0x17)

### Goal

Parse control group hotkey operations (assign/select groups 0-9).

### Research Reference

From Rex's analysis:
```
Group Hotkey (0x17):
0x17 [group: 1] [action: 1] [data: variable]

Observed patterns:
- Assigning units to control group
- Selecting control group
- Adding to control group
```

### Files to Create/Modify

- `src/actions/hotkey.rs` - Hotkey action parser

### Data Structures

```rust
/// A hotkey (control group) operation.
#[derive(Debug, Clone)]
pub struct HotkeyAction {
    /// Control group number (0-9)
    pub group: u8,

    /// Operation type
    pub operation: HotkeyOperation,

    /// Associated unit IDs (for assign operations)
    pub unit_ids: Vec<u32>,
}

/// Type of hotkey operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyOperation {
    /// Assign current selection to group (Ctrl+N)
    Assign,
    /// Select group (press N)
    Select,
    /// Add current selection to group (Shift+N)
    AddToGroup,
    /// Unknown operation
    Unknown(u8),
}
```

### Implementation Steps

1. **Analyze hotkey data patterns** (may need additional research)
   - Determine exact byte layout for 0x17 actions
   - Identify assign vs select vs add operations

2. **Implement `HotkeyAction::parse()`**
   - Read group number (0-9)
   - Read operation byte
   - Parse variable unit data if present
   - Calculate bytes consumed

3. **Add validation**
   - Group number in range 0-9
   - Operation byte matches known values

### Implementation Note

Hotkey parsing is less well-documented in Rex's analysis. Implementation may need iteration based on real data examination.

```rust
fn parse_hotkey(data: &[u8], offset: usize, ctx: &ActionContext) -> Result<(Action, usize)> {
    // Initial implementation - may need refinement
    let player_id = data[offset];
    let group = data.get(offset + 2).copied().unwrap_or(0);
    let operation_byte = data.get(offset + 3).copied().unwrap_or(0);

    let operation = match operation_byte {
        0 => HotkeyOperation::Assign,
        1 => HotkeyOperation::Select,
        2 => HotkeyOperation::AddToGroup,
        n => HotkeyOperation::Unknown(n),
    };

    // TODO: Parse unit IDs for assign operations
    // This requires more analysis of hotkey data format

    Ok((
        Action {
            player_id,
            action_type: ActionType::Hotkey(HotkeyAction {
                group,
                operation,
                unit_ids: vec![],
            }),
            timestamp_ms: ctx.timestamp_ms,
        },
        4, // Minimum size, may need adjustment
    ))
}
```

### Validation Criteria

- [ ] Parse hotkey actions without panic
- [ ] Group numbers in valid range (0-9)
- [ ] Operation types correctly identified
- [ ] Graceful handling of unknown hotkey formats

### Estimated Complexity

**Medium** - Less documented, may require iterative refinement based on real data.

---

## Phase 5F: Integration and Testing

### Goal

Integrate all action parsers and provide comprehensive testing across all replays.

### Files to Create/Modify

- `src/actions/mod.rs` - Complete integration
- `tests/action_tests.rs` - Integration tests

### Implementation Steps

1. **Wire all parsers into ActionIterator**
   - Dispatch to correct parser based on action type
   - Handle parse errors gracefully
   - Track statistics on unknown actions

2. **Extend TimeFrame API**
   ```rust
   impl TimeFrame {
       /// Get an iterator over actions in this TimeFrame.
       pub fn actions(&self) -> ActionIterator {
           ActionIterator::new(&self.action_data, ActionContext {
               timestamp_ms: self.accumulated_time_ms,
               frame_number: 0,
           })
       }

       /// Get action count without full parsing.
       pub fn action_count_estimate(&self) -> usize {
           // Rough estimate based on data size
           self.action_data.len() / 10
       }
   }
   ```

3. **Create statistics helper**
   ```rust
   /// Statistics about actions in a replay.
   #[derive(Debug, Default)]
   pub struct ActionStatistics {
       pub total_actions: u32,
       pub selection_actions: u32,
       pub ability_actions: u32,
       pub movement_actions: u32,
       pub hotkey_actions: u32,
       pub unknown_actions: u32,
       pub actions_per_player: HashMap<u8, u32>,
       pub unique_ability_codes: HashSet<AbilityCode>,
   }

   impl ActionStatistics {
       pub fn from_replay(game: &GameRecord, data: &[u8]) -> Self {
           // Iterate all frames and actions, collecting stats
       }
   }
   ```

4. **Create comprehensive tests**
   - Test all 27 replays parse without panic
   - Validate action counts match Rex's analysis
   - Verify ability code frequencies
   - Check coordinate ranges

### Validation Criteria (Overall Phase 5)

- [ ] All 27 replays parse actions without panic
- [ ] Selection actions parsed correctly (unit counts, IDs)
- [ ] Ability actions parsed with readable FourCC codes
- [ ] Movement actions have valid coordinates
- [ ] Unknown actions preserved with raw data
- [ ] Action statistics match Rex's analysis (within tolerance)
- [ ] No clippy warnings
- [ ] No unwrap() in library code
- [ ] Full documentation on public types

### Test Cases

```rust
#[test]
fn test_all_replays_parse_actions() {
    for replay in all_replays() {
        let data = decompress_replay(&replay);
        let game = GameRecord::parse(&data).unwrap();

        let mut action_count = 0;
        let mut error_count = 0;

        for frame in game.timeframes(&data) {
            let frame = frame.unwrap();
            for action in frame.actions() {
                match action {
                    Ok(_) => action_count += 1,
                    Err(_) => error_count += 1,
                }
            }
        }

        assert!(action_count > 0,
            "{:?} should have actions", replay);
        assert!(error_count < action_count / 10,
            "{:?} has too many errors: {}/{}", replay, error_count, action_count);
    }
}

#[test]
fn test_action_statistics_replay_5000() {
    let data = decompress_replay("replays/replay_5000.w3g");
    let game = GameRecord::parse(&data).unwrap();
    let stats = ActionStatistics::from_replay(&game, &data);

    // From Rex's analysis: replay_5000 has Night Elf abilities
    assert!(stats.unique_ability_codes.iter()
        .any(|c| c.as_string() == "Edem"),
        "Expected Demon Hunter ability in replay_5000");
}

#[test]
fn test_action_statistics_replay_100000() {
    let data = decompress_replay("replays/replay_100000.w3g");
    let game = GameRecord::parse(&data).unwrap();
    let stats = ActionStatistics::from_replay(&game, &data);

    // From Rex's analysis: ~5400 ability uses, 53 unique codes
    assert!(stats.ability_actions > 1000,
        "Expected many ability actions");
    assert!(stats.unique_ability_codes.len() > 20,
        "Expected many unique ability codes");
}
```

---

## Edge Cases

### Empty Action Data
- **Detection**: `TimeFrame.action_data.len() == 0`
- **Handling**: ActionIterator yields no items (valid case)

### Truncated Action Data
- **Detection**: Unexpected EOF during action parsing
- **Handling**: Return `ParserError::UnexpectedEof` with offset, yield no more actions

### Unknown Action Types
- **Detection**: Action type byte not in known set
- **Handling**: Create `ActionType::Unknown` with raw data, continue parsing

### Invalid Player IDs
- **Detection**: Player ID is 0 or > 15
- **Handling**: Return error with offset for debugging

### Multi-Player Frames
- **Detection**: After parsing one action, next byte is valid player ID (1-15)
- **Handling**: Continue parsing additional actions in same frame

### Malformed FourCC
- **Detection**: Non-ASCII bytes in ability code
- **Handling**: Store as raw bytes, use `from_utf8_lossy` for display

### Extreme Coordinates
- **Detection**: Float values NaN, Infinity, or |value| > 15000
- **Handling**: Mark as suspicious but preserve value, log warning

---

## Risks

### Risk 1: Variable Action Lengths
- **Issue**: Some actions have variable length based on content
- **Impact**: Incorrect offset advancement corrupts subsequent parsing
- **Mitigation**: Extensive testing with all replays, careful bytes_consumed tracking
- **Contingency**: If parsing fails, scan for next valid player ID

### Risk 2: Undocumented Action Types
- **Issue**: May encounter action types not in Rex's analysis
- **Impact**: Unknown actions could have unexpected sizes
- **Mitigation**: Log unknown types with full context for investigation
- **Contingency**: Add new action types as they're discovered

### Risk 3: 8-Byte Unit ID Mystery
- **Issue**: Why are unit IDs stored twice in 8-byte blocks?
- **Impact**: May miss important data if second 4 bytes differ
- **Mitigation**: Initially ignore second copy, validate they match
- **Contingency**: If they differ, investigate pattern and update struct

### Risk 4: Hotkey Format Uncertainty
- **Issue**: Hotkey action format less well-documented
- **Impact**: May need multiple implementation iterations
- **Mitigation**: Start with minimal parsing, expand based on testing
- **Contingency**: Accept higher unknown rate for hotkeys initially

### Risk 5: Action Boundary Detection
- **Issue**: No explicit action-end markers
- **Impact**: Could misparse concatenated actions
- **Mitigation**: Track expected sizes per action type, validate player IDs
- **Contingency**: If boundary detection fails, implement marker scanning

---

## Success Criteria

Phase 5 implementation succeeds when:

1. **All 27 test replays parse actions without panic**
   - ActionIterator completes for all TimeFrames
   - Error rate < 10% of total actions

2. **Selection actions correctly parsed**
   - Unit counts match actual unit data
   - Unit IDs are non-zero 32-bit values
   - Selection modes in expected range

3. **Ability actions correctly parsed**
   - FourCC codes readable as 4-character strings
   - Race detection works for known prefixes
   - Ability counts approximately match Rex's analysis

4. **Movement actions correctly parsed**
   - Coordinates are finite floats
   - Coordinate values in valid map range
   - Target detection (ground vs unit) correct

5. **Code quality meets project standards**
   - No `unwrap()` in library code
   - All public types documented
   - Tests cover happy path and error cases
   - No clippy warnings

6. **Performance acceptable**
   - Parse 1000+ TimeFrames per second
   - Memory usage proportional to action data size

---

## Dependencies

### External Crates (Already in Cargo.toml)
- `thiserror` - Error derive macros
- `flate2` - Zlib decompression (used in earlier phases)

### No New Dependencies Required

Phase 5 uses only existing infrastructure:
- IEEE 754 float parsing: `f32::from_le_bytes()` (std)
- Little-endian integers: `u16/u32::from_le_bytes()` (std)
- Hash collections: `std::collections::{HashMap, HashSet}`

---

## Implementation Order

1. **Phase 5A** (Day 1): Action Block Framework
   - Create module structure
   - Implement ActionIterator shell
   - Define action types
   - Add to TimeFrame API

2. **Phase 5B** (Day 1-2): Selection Actions
   - Implement selection parser
   - Test with known selection data
   - Validate unit ID extraction

3. **Phase 5C** (Day 2): Ability Actions
   - Implement FourCC handling
   - Implement direct ability parser
   - Implement ability with selection
   - Implement instant ability

4. **Phase 5D** (Day 2-3): Movement Actions
   - Implement coordinate parsing
   - Validate IEEE 754 extraction
   - Test coordinate ranges

5. **Phase 5E** (Day 3): Hotkey Actions
   - Implement basic hotkey parsing
   - Accept higher unknown rate initially

6. **Phase 5F** (Day 3-4): Integration and Testing
   - Wire all parsers together
   - Run comprehensive tests
   - Generate statistics
   - Document findings

---

## Appendix: Action Type Quick Reference

| Type | Sub | Size | Name | Confidence |
|------|-----|------|------|------------|
| 0x00 | 0x0D | 28 | Move/Attack | CONFIRMED |
| 0x0F | 0x00 | 18 | Instant Ability | LIKELY |
| 0x10 | - | var | Pause | UNKNOWN |
| 0x11 | - | var | Resume/Heartbeat | UNKNOWN |
| 0x16 | - | 4+8n | Selection | CONFIRMED |
| 0x17 | - | var | Hotkey | LIKELY |
| 0x1A | 0x00 | var | Ability with Selection | CONFIRMED |
| 0x1A | 0x19 | 14 | Direct Ability | CONFIRMED |

---

## Appendix: Expected Results from Rex's Analysis

### replay_5000 (Night Elf, 10.8 min)
- Unique ability codes: 2+ (pswe, lote observed)
- Selection actions: ~1930
- Players: 1, 3, 10

### replay_100000 (Human vs Undead, 21.8 min)
- Unique ability codes: 53
- Total ability uses: 5,401
- Move commands: 4,458
- Top codes: erdU (816), gmaH (537), aeph (326)

### grbn_1 (Night Elf, ~17 min)
- Unique ability codes: 39
- Total ability uses: 5,838
- Top codes: medE (2254), sgnN (1482), crae (305)

---

## Appendix: FourCC Code Reference

### Human (h/H prefix)
| Stored | Reversed | Description |
|--------|----------|-------------|
| woth | htow | Town Hall |
| eekh | hkee | Keep |
| sach | hcas | Castle |
| aeph | hpea | Peasant |
| gmaH | Hamg | Archmage |
| gkmH | Hmkg | Mountain King |
| lapH | Hpal | Paladin |

### Night Elf (e/E prefix)
| Stored | Reversed | Description |
|--------|----------|-------------|
| lote | etol | Tree of Life |
| pswe | ewsp | Wisp |
| medE | Edem | Demon Hunter |
| eekE | Ekee | Keeper of the Grove |
| crae | earc | Archer |

### Undead (u/U prefix)
| Stored | Reversed | Description |
|--------|----------|-------------|
| erdU | Udre | Dread Lord |
| cilU | Ulic | Lich |
| ocau | uaco | Acolyte |

### Neutral (n/N prefix)
| Stored | Reversed | Description |
|--------|----------|-------------|
| sgnN | Nngs | Naga Siren |
| nitN | Ntin | Neutral building |
