# Plan: Phase 4 - Decompressed Data Parsing

## Metadata
- **Agent**: Archie (Planning Agent)
- **Date**: 2025-11-25
- **Research Used**:
  - `thoughts/research/decompressed-analysis.md` - Rex's decompressed data analysis
  - `FORMAT.md` - Complete format specification with decompressed structure
  - `thoughts/plans/header-parsing.md` - Previous plan template (Phases 1-3)
- **Status**: Draft
- **Prerequisites**: Phases 1-3 validated (header parsing + decompression complete)

## Overview

This plan implements parsing of the decompressed W3G replay data, extracting:
1. **Game Record Header** - Host player info and game settings
2. **Player Slot Records** - All players in the game with names, slots, and race flags
3. **TimeFrame Iterator** - Sequential access to game action frames with timestamps

This is the first phase of extracting meaningful game data from replays. Action parsing within TimeFrames is deferred to Phase 5 to keep this phase focused and testable.

## Prerequisites

Before starting Phase 4:
- [x] Phases 1-3 complete and validated (143 tests passing)
- [x] `decompress()` function returns decompressed bytes for all 27 replays
- [x] FORMAT.md documents decompressed data structure
- [x] Rex's analysis identifies record markers (0x10, 0x16, 0x19, 0x1F, 0x20, 0x22)

## Architecture Decision: Module Organization

The decompressed data parsing will be organized under a new `records` module:

```
w3g-parser/src/
  records/
    mod.rs           # Public API, re-exports
    game_header.rs   # Game record header (0x10)
    player.rs        # Player slot records (0x16, 0x19)
    timeframe.rs     # TimeFrame iterator (0x1F)
    checksum.rs      # Checksum records (0x22)
```

---

## Phase 4A: Game Record Header

### Goal

Parse the initial game record header that starts decompressed Classic data, extracting host player information.

### Research Reference

From Rex's analysis (`decompressed-analysis.md`):
```
classic_5000.bin:
00000000: 10 01 00 00 00 03 6b 61 69 73 65 72 69 73 00 01  ......kaiseris..
00000010: 00 72 69 63 68 00 00 81 03 79 07 01 01 c1 07 a5  .rich....y......

Interpretation:
  0x10 0x01 0x00 0x00 = Record type 0x00000110
  0x00 = Unknown byte
  0x03 = Host player slot 3
  "kaiseris" + 0x00 = Host player name
```

### Files to Create/Modify

- `src/records/mod.rs` - Module root with re-exports
- `src/records/game_header.rs` - Game record header parser
- `src/lib.rs` - Add `records` module declaration

### Data Structures

```rust
/// The initial game record that appears at the start of decompressed data.
///
/// This contains the host player information and encoded game settings.
pub struct GameRecordHeader {
    /// Record type marker (always 0x00000110)
    pub record_type: u32,

    /// Unknown byte (usually 0x00)
    pub unknown_1: u8,

    /// Slot number of the host player (1-12)
    pub host_slot: u8,

    /// Name of the host player (null-terminated)
    pub host_name: String,

    /// Additional flag byte after host name (usually 0x01 or 0x02)
    pub host_flags: u8,

    /// Additional data after host name (clan tag, custom data, etc.)
    pub additional_data: String,

    /// Raw encoded game settings (encoding not yet reverse engineered)
    pub encoded_settings: Vec<u8>,

    /// Total bytes consumed by this record
    pub byte_length: usize,
}
```

### Implementation Steps

1. **Create module structure** (`src/records/mod.rs`)
   - Declare submodules: `game_header`, `player`, `timeframe`, `checksum`
   - Re-export public types
   - Add module documentation

2. **Implement `GameRecordHeader::parse()`** (`src/records/game_header.rs`)
   - Validate record type magic (0x10 0x01 0x00 0x00)
   - Read unknown byte at offset 4
   - Read host slot at offset 5
   - Read host name (null-terminated string starting at offset 6)
   - Read flags byte after null terminator
   - Read additional data string (null-terminated)
   - Capture encoded settings until next record marker
   - Track total bytes consumed for offset calculation

3. **Handle encoded settings boundary detection**
   - Scan for 0x16 (player slot marker) or 0x19 (slot record marker)
   - Store raw bytes - decoding deferred to future phase

4. **Add to lib.rs**
   - Add `pub mod records;`
   - Re-export `GameRecordHeader`

### Validation Criteria

- [ ] Parse game record header from all 27 decompressed replays
- [ ] Extract host player name correctly for all replays
- [ ] Host slot number is in valid range (1-12)
- [ ] `byte_length` correctly points to next record
- [ ] Unit tests cover edge cases (empty additional data, long names)

### Test Cases

```rust
#[test]
fn test_game_record_header_classic_5000() {
    let data = decompress_replay("replays/replay_5000.w3g");
    let header = GameRecordHeader::parse(&data).unwrap();

    assert_eq!(header.record_type, 0x00000110);
    assert_eq!(header.host_slot, 3);
    assert_eq!(header.host_name, "kaiseris");
}

#[test]
fn test_game_record_header_classic_100000() {
    let data = decompress_replay("replays/replay_100000.w3g");
    let header = GameRecordHeader::parse(&data).unwrap();

    assert_eq!(header.host_name, "MisterWinner#21670");
    assert!(header.additional_data.contains("FLO")); // FLO-STREAM hosting
}

#[test]
fn test_game_record_header_all_replays() {
    for replay in all_replays() {
        let data = decompress_replay(&replay);
        let header = GameRecordHeader::parse(&data);
        assert!(header.is_ok(), "Failed to parse game header from {:?}", replay);

        let header = header.unwrap();
        assert!(!header.host_name.is_empty());
        assert!(header.host_slot >= 1 && header.host_slot <= 12);
    }
}
```

### Estimated Complexity

**Low-Medium** - Straightforward parsing with one complexity: finding the boundary of encoded settings requires scanning for marker bytes.

---

## Phase 4B: Player Slot Records

### Goal

Parse all player slot records (0x16 marker) to build a complete roster of players in the game.

### Research Reference

From Rex's analysis:
```
Player slots found in classic_5000:
  Slot 4:  GreenField
  Slot 7:  B2W.LeeYongDae
  Slot 8:  Slash-
  Slot 9:  fnatic.e-spider
  Slot 10: HuntressShaped
  Slot 12: UP.Abstrakt
  Slot 1:  malimeda

Record format:
0x007A: 16 04 GreenField 00 01 00 00 00 00 00
        ^^ ^^ ^^^^^^^^^^ ^^ ^^^^^^^^^^^^^^^^^
        |  |  name       |  trailing data (6 bytes)
        |  slot_id
        marker 0x16
```

### Files to Create/Modify

- `src/records/player.rs` - Player slot record parser

### Data Structures

```rust
/// A player slot record (0x16 marker).
///
/// These records appear after the game header and define all
/// players participating in the game.
#[derive(Debug, Clone)]
pub struct PlayerSlot {
    /// Slot number (1-12)
    pub slot_id: u8,

    /// Player name (null-terminated)
    pub player_name: String,

    /// Trailing data (usually 0x01 0x00 0x00 0x00 0x00 0x00)
    /// May contain race flags or additional info
    pub trailing_data: [u8; 6],

    /// Total bytes consumed by this record
    pub byte_length: usize,
}

/// Alternative slot record format (0x19 marker).
///
/// Some replays use this format instead of or in addition to 0x16.
#[derive(Debug, Clone)]
pub struct SlotRecord {
    /// Slot number
    pub slot_id: u8,

    /// Player name
    pub player_name: String,

    /// Additional data (variable length)
    pub additional_data: Vec<u8>,

    /// Total bytes consumed
    pub byte_length: usize,
}

/// Unified player record that can be either format.
#[derive(Debug, Clone)]
pub enum PlayerRecord {
    /// Standard player slot (0x16)
    PlayerSlot(PlayerSlot),

    /// Alternative slot record (0x19)
    SlotRecord(SlotRecord),
}

/// Collection of all players in a game.
#[derive(Debug, Default)]
pub struct PlayerRoster {
    /// All player records found
    pub players: Vec<PlayerRecord>,

    /// Total bytes consumed by all player records
    pub byte_length: usize,
}
```

### Implementation Steps

1. **Implement `PlayerSlot::parse()`**
   - Validate marker byte (0x16)
   - Read slot ID at offset 1
   - Read player name (null-terminated) starting at offset 2
   - Read 6 trailing bytes
   - Calculate total byte length

2. **Implement `SlotRecord::parse()`** (0x19 marker)
   - Similar structure to PlayerSlot
   - May have different trailing data format

3. **Implement `PlayerRoster::parse()`**
   - Start at offset provided (after game header)
   - Loop while current byte is 0x16 or 0x19
   - Parse appropriate record type
   - Accumulate into `players` vector
   - Stop when encountering different marker (likely 0x1F TimeFrame or 0x20 chat)

4. **Add helper methods**
   - `PlayerRoster::get_by_slot(slot_id: u8) -> Option<&PlayerRecord>`
   - `PlayerRoster::player_names() -> Vec<&str>`
   - `PlayerSlot::is_human() -> bool` (based on trailing data)

### Validation Criteria

- [ ] Parse player roster from all 27 replays without errors
- [ ] Extract all player names as readable UTF-8 strings
- [ ] Slot IDs are in valid range (1-12)
- [ ] Player count matches expected (2-12 players per game)
- [ ] No duplicate slot IDs within a replay
- [ ] `byte_length` correctly points to first non-player record

### Test Cases

```rust
#[test]
fn test_player_roster_classic_5000() {
    let data = decompress_replay("replays/replay_5000.w3g");
    let header = GameRecordHeader::parse(&data).unwrap();
    let roster = PlayerRoster::parse(&data[header.byte_length..]).unwrap();

    // From Rex's analysis: 7 players
    assert_eq!(roster.players.len(), 7);

    let names: Vec<_> = roster.player_names();
    assert!(names.contains(&"GreenField"));
    assert!(names.contains(&"B2W.LeeYongDae"));
    assert!(names.contains(&"Slash-"));
}

#[test]
fn test_player_roster_classic_100000() {
    let data = decompress_replay("replays/replay_100000.w3g");
    let header = GameRecordHeader::parse(&data).unwrap();
    let roster = PlayerRoster::parse(&data[header.byte_length..]).unwrap();

    let names: Vec<_> = roster.player_names();
    assert!(names.contains(&"MisterWinner#21670"));
    assert!(names.contains(&"Liqs#21977"));
    assert!(names.contains(&"Kover00#2421"));
}

#[test]
fn test_player_names_all_replays() {
    for replay in all_replays() {
        let data = decompress_replay(&replay);
        let header = GameRecordHeader::parse(&data).unwrap();
        let roster = PlayerRoster::parse(&data[header.byte_length..]).unwrap();

        assert!(roster.players.len() >= 2, "Game should have at least 2 players");

        for player in &roster.players {
            match player {
                PlayerRecord::PlayerSlot(p) => {
                    assert!(!p.player_name.is_empty());
                    assert!(p.slot_id >= 1 && p.slot_id <= 12);
                }
                PlayerRecord::SlotRecord(s) => {
                    assert!(!s.player_name.is_empty());
                }
            }
        }
    }
}
```

### Estimated Complexity

**Medium** - Multiple record formats require careful handling, and boundary detection between player records and subsequent data (game start record, TimeFrames) needs attention.

---

## Phase 4C: TimeFrame Iterator

### Goal

Create an iterator over TimeFrame records (0x1F marker) that provides sequential access to game action frames with accumulated timestamps.

### Research Reference

From Rex's analysis:
```
TimeFrame format:
1F [time_ms:2 LE] [action_length:2 LE] [action_data...]

Evidence:
0x0233: 1f 02 00 00 00 22 04 00 00 00 00
        ^^ ^^^^^ ^^^^^ ^^^^^^^^^^^^^^^
        |  |     |     Checksum (next record)
        |  |     Action length = 0
        |  Time = 2ms
        TimeFrame marker

0x0372: 1f 3c 00 64 00 01 1a 00 16 01...
        Time = 60ms, length indicator = 0x64 = 100
```

**Note**: The exact interpretation of bytes 3-4 requires investigation. Rex noted that 0x64 doesn't match actual action data length. We'll implement cautiously and document findings.

### Files to Create/Modify

- `src/records/timeframe.rs` - TimeFrame record and iterator
- `src/records/checksum.rs` - Checksum record parser (0x22)

### Data Structures

```rust
/// A single TimeFrame record containing game actions.
///
/// TimeFrames represent a slice of game time and contain all
/// player actions that occurred during that period.
#[derive(Debug, Clone)]
pub struct TimeFrame {
    /// Time increment in milliseconds since the previous TimeFrame
    pub time_delta_ms: u16,

    /// Raw action data (parsing deferred to Phase 5)
    pub action_data: Vec<u8>,

    /// Accumulated game time in milliseconds from game start
    pub accumulated_time_ms: u32,
}

/// A checksum record that follows TimeFrames.
#[derive(Debug, Clone)]
pub struct ChecksumRecord {
    /// Record type marker (0x22)
    pub marker: u8,

    /// Checksum type (usually 0x04)
    pub checksum_type: u8,

    /// Checksum value (4 bytes)
    pub checksum: u32,
}

/// Iterator over TimeFrame records in decompressed replay data.
///
/// This iterator yields TimeFrames sequentially, handling the
/// interleaved checksum records automatically.
pub struct TimeFrameIterator<'a> {
    /// Reference to the decompressed data
    data: &'a [u8],

    /// Current position in the data
    offset: usize,

    /// Accumulated game time in milliseconds
    accumulated_time: u32,

    /// Number of TimeFrames yielded so far
    frame_count: usize,
}
```

### Implementation Steps

1. **Implement `ChecksumRecord::parse()`** (`src/records/checksum.rs`)
   - Validate marker (0x22)
   - Read checksum type (offset 1)
   - Read checksum value (offset 2, 4 bytes)
   - Return total size (6 bytes)

2. **Implement `TimeFrame::parse()`**
   - Validate marker (0x1F)
   - Read time delta (offset 1, u16 LE)
   - Read action length indicator (offset 3, u16 LE)
   - **Investigation needed**: Determine how to find true action data boundary
   - Options:
     a. Use length indicator if reliable
     b. Scan for next 0x1F or 0x22 marker
     c. Parse until checksum record
   - Extract action data bytes

3. **Implement `TimeFrameIterator`**
   - `new(data: &[u8], start_offset: usize)` - Initialize at given offset
   - Implement `Iterator<Item = Result<TimeFrame>>`
   - Handle interleaved checksum records (skip them)
   - Track accumulated time
   - Stop at end of data or unrecognized marker

4. **Add helper methods**
   - `TimeFrameIterator::total_time_ms()` - Get final accumulated time
   - `TimeFrameIterator::frame_count()` - Get number of frames seen
   - `TimeFrame::is_empty()` - Check if no action data

### Implementation Note: Action Length Mystery

Rex's analysis indicates the bytes at offset 3-4 may not be a simple length field. Proposed approach:

```rust
fn find_action_boundary(data: &[u8], offset: usize) -> usize {
    // Strategy 1: Look for next record marker
    // 0x1F = TimeFrame, 0x22 = Checksum, 0x20 = Chat
    for i in offset..data.len() {
        match data[i] {
            0x1F | 0x22 | 0x20 | 0x17 => return i,
            _ => continue,
        }
    }
    data.len()
}
```

This may need refinement based on testing with real data.

### Validation Criteria

- [ ] Iterator produces TimeFrames from all 27 replays
- [ ] `time_delta_ms` values are reasonable (0-1000ms typical)
- [ ] Accumulated time increases monotonically
- [ ] Total game time approximately matches header's duration field
- [ ] No crashes or infinite loops on any replay
- [ ] Checksum records are properly skipped

### Test Cases

```rust
#[test]
fn test_timeframe_iterator_basic() {
    let data = decompress_replay("replays/replay_5000.w3g");
    let start_offset = find_timeframe_start(&data);

    let mut frame_count = 0;
    let mut last_time = 0u32;

    for result in TimeFrameIterator::new(&data, start_offset) {
        let frame = result.expect("Failed to parse TimeFrame");

        assert!(frame.accumulated_time_ms >= last_time);
        last_time = frame.accumulated_time_ms;
        frame_count += 1;
    }

    // From Rex's analysis: ~15,229 TimeFrames in large replays
    assert!(frame_count > 1000, "Expected many TimeFrames");
}

#[test]
fn test_timeframe_time_accumulation() {
    let data = decompress_replay("replays/replay_5000.w3g");
    let iter = TimeFrameIterator::new(&data, find_timeframe_start(&data));

    // Collect all frames
    let frames: Vec<_> = iter.collect::<Result<Vec<_>, _>>().unwrap();

    // Total time should be close to game duration
    if let Some(last) = frames.last() {
        // replay_5000 duration: ~650,600 ms from header
        let expected = 650_600;
        let tolerance = expected / 10; // 10% tolerance
        assert!(
            (last.accumulated_time_ms as i64 - expected as i64).abs() < tolerance as i64,
            "Total time {} too far from expected {}",
            last.accumulated_time_ms,
            expected
        );
    }
}

#[test]
fn test_timeframe_iterator_all_replays() {
    for replay in all_replays() {
        let data = decompress_replay(&replay);
        let iter = TimeFrameIterator::new(&data, find_timeframe_start(&data));

        // Should be able to iterate without crashes
        let mut count = 0;
        for result in iter {
            assert!(result.is_ok(), "Failed in {:?}: {:?}", replay, result);
            count += 1;

            // Safety limit to avoid infinite loops during development
            if count > 100_000 {
                break;
            }
        }

        assert!(count > 0, "Expected at least some TimeFrames in {:?}", replay);
    }
}
```

### Estimated Complexity

**Medium-High** - The action length interpretation requires investigation, and handling interleaved record types adds complexity. Iterator implementation needs careful state management.

---

## Phase 4D: Integration and Public API (Optional)

### Goal

Create a unified API for accessing parsed replay data. This phase can be deferred if time is limited.

### Data Structures

```rust
/// Parsed game record containing all extracted metadata.
pub struct GameRecord {
    /// Game record header with host player info
    pub header: GameRecordHeader,

    /// All players in the game
    pub players: PlayerRoster,

    /// Offset where TimeFrames begin
    pub timeframe_offset: usize,
}

impl GameRecord {
    /// Parse a game record from decompressed replay data.
    pub fn parse(data: &[u8]) -> Result<Self>;

    /// Get an iterator over TimeFrames.
    pub fn timeframes<'a>(&self, data: &'a [u8]) -> TimeFrameIterator<'a>;

    /// Get the host player's name.
    pub fn host_name(&self) -> &str;

    /// Get all player names.
    pub fn player_names(&self) -> Vec<&str>;
}
```

### Files to Create/Modify

- `src/records/mod.rs` - Add `GameRecord` struct and unified API

### Implementation Steps

1. Implement `GameRecord::parse()` that chains:
   - `GameRecordHeader::parse()`
   - `PlayerRoster::parse()`
   - Calculate `timeframe_offset`

2. Add convenience methods for common operations

3. Update `src/lib.rs` to re-export `GameRecord`

### Validation Criteria

- [ ] `GameRecord::parse()` succeeds for all 27 replays
- [ ] Can access all metadata through unified API
- [ ] TimeFrame iterator starts at correct offset

---

## Edge Cases

### Empty or Short Replays
- **Detection**: Decompressed size < 100 bytes
- **Handling**: Return `ParserError::InvalidHeader` with descriptive message

### Malformed Player Names
- **Detection**: Non-UTF8 bytes in name field
- **Handling**: Use `String::from_utf8_lossy()` and log warning

### Unexpected Record Markers
- **Detection**: Byte doesn't match known markers (0x10, 0x16, 0x19, 0x1F, 0x20, 0x22)
- **Handling**: Log warning with offset and byte value, attempt to continue

### Zero-Length TimeFrames
- **Detection**: `action_data.len() == 0`
- **Handling**: Valid case - represents time passing with no actions

### GRBN Metadata Section
- **Context**: GRBN replays have protobuf metadata before Classic game data
- **Handling**: Phase 4 focuses on Classic data; GRBN metadata parsing deferred
- **Implementation**: Start parsing after GRBN metadata (already handled by decompress)

### Truncated Data
- **Detection**: Unexpected EOF during parsing
- **Handling**: Return accumulated data with `RecoverableError` in permissive mode

---

## Risks

### Risk 1: Action Length Field Interpretation
- **Issue**: Rex's analysis shows bytes 3-4 don't directly encode action length
- **Impact**: May cause incorrect TimeFrame boundaries
- **Mitigation**: Implement marker-based boundary detection as fallback
- **Contingency**: If boundary detection fails, defer TimeFrame iteration to Phase 5

### Risk 2: Unknown Record Types
- **Issue**: May encounter record markers not documented in analysis
- **Impact**: Could disrupt parsing sequence
- **Mitigation**: Skip unknown records, log for investigation
- **Contingency**: Add verbose mode that dumps unknown records for Rex to analyze

### Risk 3: GRBN vs Classic Data Structure Differences
- **Issue**: GRBN embedded Classic data may have subtle differences
- **Impact**: Parsing logic may fail for GRBN replays
- **Mitigation**: Test with both format types early and often
- **Contingency**: Add format-specific handling if needed

### Risk 4: Performance on Large Replays
- **Issue**: Some replays have 15,000+ TimeFrames
- **Impact**: Memory usage, iteration performance
- **Mitigation**: Iterator pattern (lazy evaluation), don't load all at once
- **Contingency**: Add streaming mode that discards processed frames

---

## Success Criteria

The Phase 4 implementation succeeds when:

1. **All 27 test replays parse without errors**
   - Game record header extracted
   - Player roster built
   - TimeFrame iterator completes

2. **Player names are readable**
   - All player names are valid UTF-8 strings
   - Names match those found by Rex's manual analysis

3. **TimeFrame statistics are reasonable**
   - Frame counts in thousands (matching Rex's analysis)
   - Accumulated time approximately matches header duration
   - No infinite loops or crashes

4. **Code quality meets project standards**
   - No `unwrap()` in library code
   - All public types documented
   - Tests cover happy path and error cases
   - No clippy warnings

5. **Foundation ready for Phase 5 (Action Parsing)**
   - TimeFrame `action_data` available for detailed parsing
   - Player roster available for correlating actions to players

---

## Dependencies

### External Crates (Already in Cargo.toml)
- `thiserror` - Error derive macros
- `flate2` - Zlib decompression (used in Phases 1-3)

### No New Dependencies Required

Phase 4 uses only existing infrastructure and standard library.

---

## Appendix: Record Marker Reference

| Marker | Record Type | Phase | Confidence |
|--------|-------------|-------|------------|
| 0x10 0x01 0x00 0x00 | Game Record Header | 4A | CONFIRMED |
| 0x16 | Player Slot | 4B | CONFIRMED |
| 0x19 | Slot Record (alternative) | 4B | LIKELY |
| 0x1F | TimeFrame | 4C | CONFIRMED |
| 0x22 0x04 | Checksum | 4C | CONFIRMED |
| 0x20 | Chat Message | Future | LIKELY |
| 0x17 | Leave Game? | Future | UNKNOWN |
| 0x1A | Action Command | Phase 5 | LIKELY |

---

## Appendix: Expected Player Names from Research

### replay_5000.w3g (Classic Type A)
- Host: kaiseris (slot 3)
- Players: GreenField, B2W.LeeYongDae, Slash-, fnatic.e-spider, HuntressShaped, UP.Abstrakt, malimeda

### replay_100000.w3g (Classic Type B)
- Host: MisterWinner#21670 (slot 1)
- Players: Liqs#21977, Kover00#2421

### replay_1.w3g (GRBN)
- Metadata: gonnabealright
- Map: (2)DalaranJ
- Version: 1.28.0

---

## Implementation Order

1. **Phase 4A** (Day 1): Game Record Header
   - Create module structure
   - Implement header parsing
   - Write unit tests

2. **Phase 4B** (Day 1-2): Player Records
   - Implement PlayerSlot parsing
   - Implement PlayerRoster
   - Integration tests with real replays

3. **Phase 4C** (Day 2-3): TimeFrame Iterator
   - Implement ChecksumRecord
   - Implement TimeFrame parsing
   - Implement TimeFrameIterator
   - Extensive testing with all replays

4. **Validation** (Day 3): Full validation pass
   - Run all tests
   - Verify against Rex's analysis
   - Document any discoveries
