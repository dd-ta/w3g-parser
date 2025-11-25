# Phase 4: Decompressed Data Analysis Report

**Date**: 2025-11-25
**Analyst**: Rex (Binary Analysis Agent)
**Status**: Complete

## Executive Summary

This analysis examined the internal structure of decompressed W3G replay data across three different replay types:
- Classic Type A (build v26): `replay_5000.w3g`
- Classic Type B (build v10036): `replay_100000.w3g`
- GRBN/Reforged: `replay_1.w3g`

Key findings include the identification of player record formats, TimeFrame structures, checksum patterns, and ability code encoding.

---

## Methodology

### 1. Data Extraction

Created a dump binary (`src/bin/dump.rs`) using the w3g-parser library to extract decompressed data:

```bash
cargo run --bin dump replay_5000.w3g analysis/classic_5000.bin
cargo run --bin dump replay_100000.w3g analysis/classic_100000.bin
cargo run --bin dump replay_1.w3g analysis/grbn_1.bin
```

### 2. String Extraction

Used `strings` command to identify readable content:
- Player names
- Chat messages
- Ability codes
- Map-related strings

### 3. Hex Pattern Analysis

Used `xxd` to examine binary structure:
- Identified record type markers (0x1F, 0x22, 0x16, etc.)
- Traced data boundaries
- Cross-referenced patterns across files

### 4. Statistical Analysis

Counted byte pattern occurrences to identify common structures:
```
0x1f (TimeFrame): 15,229 occurrences
0x22 (Checksum):  13,349 occurrences
0x16 (Selection): 1,930 occurrences
0x1a (Action):    1,943 occurrences
```

---

## Findings

### 1. Game Record Header

**Confidence**: CONFIRMED

All Classic decompressed data starts with:
```
10 01 00 00 00 XX [player_name] 00 ...
```

Where:
- `10 01 00 00` = Record type 0x00000110
- `XX` = Host player slot number
- Player name is null-terminated

**Evidence**:
```
classic_5000.bin offset 0x00:
10 01 00 00 00 03 6b 61 69 73 65 72 69 73 00
               ^^ ^^^^^^^^^^^^^^^^^^^^^^^^
               |  "kaiseris"
               Host slot 3

classic_100000.bin offset 0x00:
10 01 00 00 00 01 4d 69 73 74 65 72 57 69 6e 6e 65 72 23 32 31 36 37 30 00
               ^^ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
               |  "MisterWinner#21670"
               Host slot 1
```

### 2. Player Slot Records

**Confidence**: CONFIRMED

Player slots appear after the encoded game settings section with marker 0x16:
```
16 [slot_id] [name_string] 00 01 00 00 00 00 00
```

**Evidence** (classic_5000.bin):
```
0x007A: 16 04 GreenField 00 01 00 00 00 00 00
0x008B: 16 07 B2W.LeeYongDae 00 01 00 00 00 00 00
0x00A4: 16 08 Slash- 00 01 00 00 00 00 00
0x00B4: 16 09 fnatic.e-spider 00 01 00 00 00 00 00
```

### 3. TimeFrame Structure

**Confidence**: CONFIRMED

TimeFrames use marker 0x1F followed by time increment:
```
1F [time_ms:2 LE] [flags:2] [action_data...]
```

**Evidence**:
```
Empty frame (no actions):
0x0233: 1f 02 00 00 00 22 04 00 00 00 00
        time=2ms, flags=0x0000, immediately followed by checksum

Frame with actions:
0x0372: 1f 3c 00 64 00 01 1a 00 16 01...
        time=60ms, flags=0x0064, followed by 57 bytes of action data
```

### 4. Checksum Records

**Confidence**: CONFIRMED

Checksum blocks use marker 0x22 0x04:
```
22 04 [checksum:4]
```

These appear after TimeFrames and contain game state verification data.

### 5. Ability Codes

**Confidence**: CONFIRMED

Abilities are identified by 4-byte FourCC codes preceded by 0x1A 0x19:
```
1a 19 [ability_code:4]
```

Codes observed:
| Stored | Reversed | Count |
|--------|----------|-------|
| medE   | Edem     | 436   |
| moae   | eaom     | 278   |
| lote   | etol     | 178   |
| etae   | eate     | 166   |
| eekE   | Ekee     | 148   |
| pswe   | ewsp     | 77    |

### 6. GRBN Metadata Structure

**Confidence**: CONFIRMED

GRBN decompressed data contains two sections:
1. **Protobuf metadata** (~2500 bytes): Player profiles, game settings, UTF-8 strings
2. **Classic game data** (remainder): Standard action stream format

**Evidence** (grbn_1.bin):
```
Metadata section: 0x0000 - 0x09BC (2,493 bytes)
  Contains: Player names (Chinese UTF-8), version strings, map name

Game data section: 0x09BD onwards (647,168 bytes)
  Starts with: 10 01 00 00 00 01 [player_name]
```

Strings found in metadata:
- Player: "gonnabealright"
- Map: "(2)DalaranJ"
- Version: "1.28.0"
- Team: "TeamA1"

### 7. Chat Message Records

**Confidence**: LIKELY

Chat messages use marker 0x20:
```
20 03 [msg_id:2] [flags] [message] 00
```

**Evidence**:
```
0x0186: 20 03 3a 00 20 00 00 00 00 "Shortest load by player [kaiseris] was 1.83 seconds." 00
```

### 8. Encoded Game Settings

**Confidence**: INVESTIGATING

The section between host player and slot records contains encoded game/map info. Encoding scheme uses 0x01 as escape byte.

Partial decoding reveals:
- "aqs" likely encodes "Maps"
- "W3C" visible in newer replays (W3Champions)
- "w3y" likely encodes "w3x" (map extension)

---

## Open Questions

1. **TimeFrame flags interpretation**: The bytes 3-4 after time increment (e.g., 0x0064) don't directly correspond to action data length. Further investigation needed.

2. **Encoded string algorithm**: The exact encoding scheme for game settings section needs to be reverse engineered.

3. **Action command substructure**: The internal format of action data between TimeFrame header and checksum needs more detailed analysis.

4. **Unit Object IDs**: The 4-byte unit handles (e.g., 0x00003B08) follow some pattern - possibly sequential allocation.

5. **Checksum algorithm**: The 4-byte checksum values use an unknown hashing algorithm.

---

## Recommendations for Phase 5

### Immediate Implementation

1. **Player Record Parser**: Implement parsing for:
   - Game record header (host player info)
   - Player slot records (0x16 marker)

2. **TimeFrame Iterator**: Create an iterator that:
   - Yields time increments
   - Provides access to raw action data
   - Handles checksum verification

3. **Basic Action Decoder**: Parse ability commands:
   - Extract ability codes from 0x1A 0x19 sequences
   - Track unit selections (0x16 within actions)

### Future Work

1. **String Decoding**: Reverse engineer the encoded game settings to extract map path and game name.

2. **Full Action Parsing**: Implement complete action command parsing for:
   - Unit movement
   - Ability use
   - Item interactions
   - Building construction

3. **GRBN Metadata**: Consider protobuf parsing for rich metadata extraction.

---

## File References

| File | Description | Size |
|------|-------------|------|
| `/Users/felipeh/Development/w3g/analysis/classic_5000.bin` | Decompressed Type A replay | 278,528 bytes |
| `/Users/felipeh/Development/w3g/analysis/classic_100000.bin` | Decompressed Type B replay | 868,352 bytes |
| `/Users/felipeh/Development/w3g/analysis/grbn_1.bin` | Decompressed GRBN replay | 649,661 bytes |
| `/Users/felipeh/Development/w3g/w3g-parser/src/bin/dump.rs` | Extraction tool | - |
| `/Users/felipeh/Development/w3g/FORMAT.md` | Updated format documentation | - |

---

## Appendix: Hex Evidence

### Player Names Found

**classic_5000.bin**:
- kaiseris (host)
- GreenField
- B2W.LeeYongDae
- Slash-
- fnatic.e-spider
- HuntressShaped
- UP.Abstrakt
- malimeda

**classic_100000.bin**:
- MisterWinner#21670 (host)
- Liqs#21977
- Kover00#2421

**grbn_1.bin**:
- gonnabealright
- (Chinese characters in player profile)

### Chat Messages Found

```
"Shortest load by player [kaiseris] was 1.83 seconds."
"Longest load by player [GreenField] was 5.58 seconds."
"Your load time was 5.58 seconds."
"HuntressShaped has left the game voluntarily."
"malimeda has left the game voluntarily."
"B2W.LeeYongDae has left the game voluntarily."
"Warning! Desync detected!"
"Slash- was dropped due to desync."
```
