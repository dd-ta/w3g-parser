# Phase 4: Decompressed Data Parsing - Implementation Report

## Overview

Phase 4 implements parsing of decompressed W3G replay data, including:
- Game record headers (0x10 0x01 0x00 0x00 magic)
- Player slot records (0x16 marker)
- TimeFrame iteration (0x1F / 0x1E markers)

## Implementation Summary

### Files Created

```
w3g-parser/src/records/
  mod.rs           - Module root, GameRecord struct, public API
  game_header.rs   - GameRecordHeader parsing
  player.rs        - PlayerSlot, SlotRecord, PlayerRoster parsing
  timeframe.rs     - TimeFrame, TimeFrameIterator, related records
```

### Key Data Structures

#### GameRecordHeader (`game_header.rs`)
```rust
pub struct GameRecordHeader {
    pub record_type: u32,       // Always 0x00000110
    pub unknown_1: u8,
    pub host_slot: u8,
    pub host_name: String,
    pub host_flags: u8,
    pub additional_data: String,
    pub encoded_settings: Vec<u8>,
    pub byte_length: usize,
}
```

#### PlayerSlot (`player.rs`)
```rust
pub struct PlayerSlot {
    pub slot_id: u8,
    pub player_name: String,
    pub trailing_data: Vec<u8>,
    pub byte_length: usize,
}

pub struct PlayerRoster {
    players: Vec<PlayerRecord>,
    pub byte_length: usize,
}
```

#### TimeFrame (`timeframe.rs`)
```rust
pub struct TimeFrame {
    pub time_delta_ms: u16,
    pub action_data: Vec<u8>,
    pub accumulated_time_ms: u32,
}

pub struct TimeFrameIterator<'a> { ... }
pub struct TimeFrameStats { ... }
```

#### GameRecord (`mod.rs`)
```rust
pub struct GameRecord {
    pub header: GameRecordHeader,
    pub players: PlayerRoster,
    pub timeframe_offset: usize,
}
```

## Deviations from Original Plan

### 1. GRBN Replay Handling

**Discovery**: GRBN (Reforged) replays have two format variants:
1. **Embedded Classic**: GRBN metadata (protobuf) + embedded Classic game record
2. **Protobuf-only**: Newer GRBN replays that contain only protobuf data

**Solution**: Added `find_game_record_start()` function that searches for the game record magic bytes (0x10 0x01 0x00 0x00) within decompressed data. This handles both cases:
- Classic replays: Game record starts at offset 0
- GRBN with embedded Classic: Game record found within the data
- GRBN protobuf-only: Returns error (parsing not implemented)

### 2. TimeFrame Count Discrepancy

**Observation**: The TimeFrame count is lower than initially expected. This appears to be due to:
- Action data boundary detection finding markers earlier than expected
- Some action data contains bytes that match record markers (0x17, 0x20, etc.)

**Note**: The TimeFrame iterator successfully parses frames with accumulated time, but the total count may be lower than the actual number of frames in the replay. This is a known limitation that would require more sophisticated action boundary detection.

### 3. Additional Record Types

Implemented additional record types beyond the original plan:
- `ChecksumRecord` (0x22): Game state verification
- `ChatMessage` (0x20): In-game chat messages
- `LeaveRecord` (0x17): Player leaving the game
- `SlotRecord` (0x19): Alternative player slot format

## Test Results

### Unit Tests
- 120 unit tests pass
- All record parsing tests pass

### Integration Tests
- 12 records integration tests pass
- All 12 Classic replays parse successfully
- GRBN replays handled gracefully (with/without embedded Classic)

### Clippy
- No warnings in library code
- Only 2 minor warnings in dump.rs binary (format string style)

## Player Names Extracted from Test Replays

### Classic Type A Replays (build < 10000)

**replay_5000.w3g**
- Host: kaiseris (slot 3)
- Players: GreenField, B2W.LeeYongDae, Slash-, fnatic.e-spider, HuntressShaped, UP.Abstrakt, malimeda

**replay_5001.w3g**
- Host: kaiseris
- Players: WoLv, IzhStyle.KemPeR, LeMei

**replay_5002.w3g**
- Host: B2W.MacWinner
- Players: kaiseris, WemadeFOX.Shaky, ruvinob, HazyFoggy, zhasmina123, 123456789012345, WoLv

### Classic Type B Replays (build >= 10000)

**replay_100000.w3g**
- Host: MisterWinner#21670
- Players: Liqs#21977, Kover00#2421

**replay_100001.w3g**
- Host: Liqs#21977
- Players: D3xTer#21492, Kover00#2421

**replay_10000.w3g**
- Host: cammed
- Players: 20150828, x3-DemoN, JUSTANOTHERORC, AnotherSmurF-

### GRBN Replays (with embedded Classic)

**replay_1.w3g**
- Host: 如果没有你 (Chinese characters)
- 0 players in slot records

**replay_3.w3g**
- Host: LuciferLNMS
- Players: 2

## Validation Criteria Results

- [x] Game record header parses for all Classic replays (12/12)
- [x] Host player name extracted correctly
- [x] Player slots parsed (verified against strings in decompressed data)
- [x] TimeFrame iterator produces non-zero frames
- [x] No crashes on any replay (all 27 handled gracefully)
- [x] All tests pass (120 unit + 12 integration + 28 previous)
- [x] Clippy clean (no library warnings)

## Known Limitations

1. **GRBN Protobuf-only Format**: Newer GRBN replays that don't contain embedded Classic game records cannot be fully parsed. This would require implementing protobuf parsing.

2. **TimeFrame Boundary Detection**: Action data may contain bytes that match record markers, causing some TimeFrames to be truncated. A more sophisticated approach would use the length hint field or understand action structure.

3. **Encoded Settings**: The game settings bytes after the host name are not decoded. The encoding format is not yet reverse engineered.

## Next Steps (Phase 5)

Phase 5 would implement:
- Action data parsing within TimeFrames
- Unit selection, movement, ability usage commands
- More detailed player action analysis
