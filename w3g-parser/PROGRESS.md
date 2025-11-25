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

## Next Steps (Optional Enhancements)

- [ ] Expose chat messages in library API (currently CLI-only)
- [ ] Add sender player ID to chat messages
- [x] ~~Improve action type identification (reduce "Unknown" actions)~~ - Completed
- [ ] Add game metadata decoding (map name, game name from encoded settings)
- [ ] Further research on 0x10-0x14 action types for proper validation
