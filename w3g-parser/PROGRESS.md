# W3G Parser Progress

## Current State: IMPLEMENTATION COMPLETE

### Phase Summary

The W3G parser is now feature-complete with the following capabilities:

1. **Header Parsing** - Classic (Type A/B) and GRBN formats
2. **Decompression** - zlib decompression for all block types
3. **Game Record Parsing** - Host player info, encoded settings
4. **Player Roster Parsing** - All player slots, including build 10100+ extended metadata
5. **TimeFrame Iteration** - Full action stream parsing with recovery from unknown markers
6. **Action Parsing** - Movement, selection, ability, hotkey, and edge-case actions
7. **Chat Message Parsing** - Player and system messages (newly added)
8. **CLI Tool** - Full-featured `w3g-parser` binary with parse, info, validate, batch commands

### Test Coverage

- 159 unit tests passing
- 53 integration tests passing
- 27 test replays from warcraft3.info API

### Recent Bug Fixes (Build 10100+ compatibility)

1. **Game Header Boundary Detection** - Fixed `find_settings_boundary()` to not early-return on 0x1F/0x20/0x22 bytes within encoded settings
2. **Player Parsing** - Added `find_extended_metadata_end()` to skip extended player metadata in modern builds
3. **TimeFrame Iteration** - Made iterator resilient to unknown markers with recovery scanning

### Chat Message Format (Reverse Engineered)

```
Offset 0: 0x20 (marker)
Offset 1: flags (0x03 = system, 0x07+ = player)
Offset 2-3: message_id (u16 LE)
Offset 4-8: padding (0x20 XX 0x00 0x00 0x00)
Offset 9+: null-terminated message
```

### CLI Usage

```bash
# Basic info
w3g-parser info replay.w3g

# Parse with options
w3g-parser parse replay.w3g --players --stats --chat

# JSON output
w3g-parser parse replay.w3g --output json --chat

# Validate
w3g-parser validate replay.w3g --verbose

# Batch processing
w3g-parser batch ./replays --summary
```

### Action Parser Improvements (2025-11-25)

Implemented comprehensive action type parsing improvements:

1. **Phase 1**: Fixed hotkey parsing bug - Assign operations now properly consume unit IDs
2. **Phase 2**: Added movement subcommands (0x0E Attack-move, 0x0F Patrol, 0x10 Hold, 0x12 Smart-click)
3. **Phase 3**: Investigated 0x10-0x14 unit ability types (disabled due to data byte conflicts)
4. **Phase 4**: Implemented edge-case action types:
   - SelectSubgroup (0x19) - 13 bytes
   - RemoveFromQueue (0x1E) - 6 bytes
   - ChangeAllyOptions (0x50) - 6 bytes
   - TransferResources (0x51) - 10 bytes
   - MinimapPing (0x68) - 13 bytes

All implementations validated with 159 unit tests passing.

### Reforged Action Format (Reverse Engineered 2025-11-25)

Implemented Reforged-specific action parsers through byte pattern analysis:

1. **0x11 Wrapped Ability** - Reforged wraps abilities in a 5-byte header:
   ```
   0x11 0x00 0x18 [counter] 0x03 [inner_action...]
   ```
   The inner action is typically `0x1A 0x19` (direct ability) with FourCC and target.
   Total: 19 bytes for wrapped direct ability.

2. **0x03 Queue/Repeat Action** - 5-byte marker:
   ```
   0x03 0x00 0x18 [counter] 0x03
   ```
   Appears to signal action repetition or queue operations.

3. **0x15 BattleNet Sync** - 24-byte sync payload with Base64-like data.

**Impact**: Reduced total actions from 16713 to 12470 on test Reforged replay by properly parsing wrapped actions instead of fragmenting them.

### Extended Reforged Format Support (2025-11-25)

Implemented comprehensive Reforged action parser improvements achieving **<3% unknown** actions:

1. **0x11 Classic Variant (0x7B marker)** - BattleNet sync packets with variable length:
   ```
   0x11 0x00 0x7B [variable payload 4-18 bytes]
   ```
   Different from Reforged's 0x18 marker, appears in Classic format replays.

2. **Wrapped Selection Types (0x26, 0x36, 0x46, 0x56, 0x2E, 0x3E, 0x4E, 0x5E)**:
   ```
   [type] 0x00 0x16 [selection_data...]
   ```
   Base Selection (0x16) with modifier flags in upper nibble.

3. **Short-Form Markers (0x20-0x7F range)**:
   - Pattern: `[type] 0x00` (2 bytes) - sync/state markers
   - Some contain embedded selection: `[type] 0x00 0x16 [count] [mode] [flags] [unit_ids]`

4. **High-Range Types (0x80-0xFF)**:
   - 0xA0 with subcommand 0x02 - Reforged state sync marker

5. **Classic Wrapped Ability (0x03 0x1A)**:
   ```
   0x03 0x1A 0x19 [fourcc 4] [target 4] = 12 bytes
   ```

6. **Embedded Ability (0x0E 0x00 0x1A 0x19)**:
   Attack-move with embedded direct ability, 14 bytes total.

7. **Boundary Detection Fix**: Removed 0x01 from `is_known_action_type()` to prevent
   action fragmentation when unit ID data contains `[1-15][0x01]` patterns.

**Results**:
- Reforged (GRBN): 185/6482 = **2.9% unknown**
- Classic (10034): 334/7589 = **4.4% unknown**

### Chat Message Enhancements (2025-11-25)

1. **ChatMessage exported in public API** - Now available via `w3g_parser::ChatMessage`
2. **Player slot ID extraction** - `sender_slot: Option<u8>` field identifies sender:
   - `Some(1-24)` = Player slot ID
   - `None` = System message (flags = 0x03)
3. **Helper method** - `is_system_message()` for easy categorization

### Game Metadata Decoding (2025-11-25)

Added methods to `GameRecordHeader`:

1. **`game_name()`** - Extracts lobby name from encoded settings
   - Variant A: First byte 0x00, name at offset 1 (null-terminated)
   - Returns empty string for Variant B (no game name)

2. **`map_path_raw()`** - Extracts obfuscated map path
   - Map path is interleaved with non-ASCII bytes
   - Returns printable ASCII fragments (e.g., "oMaqs/W3Cmiamqioos]w3c_s13...")
   - Recognizable patterns: "Maps", directory separators, file extensions

## Next Steps (Optional Enhancements)

- [x] ~~Expose chat messages in library API~~ - Completed
- [x] ~~Add sender player ID to chat messages~~ - Completed (sender_slot field)
- [x] ~~Improve action type identification~~ - Completed (<5% unknown)
- [x] ~~Add game metadata decoding~~ - Completed (game_name, map_path_raw)
- [ ] Further research on 0x10-0x14 action types for proper validation
- [x] ~~Further Reforged format research~~ - Completed
- [ ] Proper map path deobfuscation (current extraction is raw/partial)
