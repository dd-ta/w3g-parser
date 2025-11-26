# Encoded Settings Research Report

## Executive Summary

This document describes the reverse engineering findings for the `encoded_settings` field in the W3G `GameRecordHeader` structure. The field contains game metadata including map name and game name.

**Key Findings:**

1. **Game Name Location:**
   - In replays with custom game names, it appears at offset 0x01 as a null-terminated string
   - In replays without custom names, the game name field is empty

2. **Map Path Encoding:**
   - The map file path is obfuscated by interleaving actual path characters with "junk" bytes
   - Decoding strategy: Extract all printable ASCII characters (0x20-0x7E) from the encoded section
   - The decoded path contains recognizable patterns like "Maps/", ".w3x", and directory names
   - Some character corruption or extra characters may be present in the decoded output

3. **Format Variations:**
   - Two structural variants identified based on whether a custom game name is present
   - Reforged format uses a completely different structure (likely Protocol Buffers)

## Overview

This research analyzes the `encoded_settings` field in the W3G `GameRecordHeader` structure to extract game metadata including map name and game name.

## Format Structure

### Classic Format (Pre-Reforged)

The `encoded_settings` field follows this structure:

**Note:** The structure varies depending on whether the game has a custom name or not.

#### Variant A: With Custom Game Name (e.g., replay_80852.w3g)
```
Offset  | Size | Type        | Description
--------|------|-------------|------------------------------------------
0x00    | 1    | u8          | Initial byte (usually 0x00)
0x01    | var  | string      | Game name (null-terminated, ASCII)
var     | 1    | u8          | Null terminator for game name
var+1   | ~12  | bytes       | Game settings bytes (exact format TBD)
var+13  | var  | encoded str | Encoded map file path
var     | 1    | u8          | Null terminator for map path
var     | var  | bytes       | Additional settings data
```

#### Variant B: Without Custom Game Name (e.g., replay_5000.w3g)
```
Offset  | Size | Type        | Description
--------|------|-------------|------------------------------------------
0x00    | var  | string      | Copy of additional_data field from header
var     | 2    | u8[2]       | Two null bytes (0x00 0x00)
var+2   | ~6   | bytes       | Game settings bytes
var+8   | var  | encoded str | Encoded map file path
var     | 1    | u8          | Null terminator for map path
var     | var  | bytes       | Additional settings data
```

In Variant B, the game name is empty, and the encoded_settings starts with the host's clan tag or additional data.

## Detailed Analysis

### Hex Dump Annotations

#### Sample 1 (replay_80852.w3g) - Annotated

```
Offset  Bytes                                            ASCII          Description
------  -----------------------------------------------  -------------  -----------------------
0x00    00                                               .              Initial byte (0x00)
0x01    46 4c 4f 2d 53 54 52 45 41 4d                   FLO-STREAM     Game name
0x0b    00                                               .              Null terminator
0x0c    00                                               .              Settings byte
0x0d    01 03 79 07 01 01 01 01 f9 01 01 0d             .              Settings bytes (12)
0x19    6f c1 7f 4d fb 61 71 73                          o..M.aqs       Encoded map path (start)
0x21    2f 57 33 43 6d 69 61 6d 71 69 6f 6f             /W3Cmiamqioo   Encoded map path (cont)
0x2d    fb 73 5d 77 33 63 5f 73 f7 31 33                .s]w3c_s.13    Encoded map path (cont)
0x38    2f 31 5f 43 6f 5d 6f 63 65 61 6d 65             /1_Co]oceame   Encoded map path (cont)
0x44    65 c5 49 69 6d 6d 2f 77 33 21 79                e.Iimm/w3!y    Encoded map path (cont)
...

Decoded map path (printable chars only):
"oMaqs/W3Cmiamqioos]w3c_s13/1_Co]oceameeIimm/w3!yGMOy!q9i/So5i=?"
```

#### Sample 2 (replay_5000.w3g) - Annotated

```
Offset  Bytes                                            ASCII          Description
------  -----------------------------------------------  -------------  -----------------------
0x00    72 69 63 68                                      rich           Additional data copy
0x04    00 00                                            ..             Two null bytes
0x06    81 03 79 07 01 01 c1 07 a5 c1 07                ..y.......     Settings bytes (~8-10)
0x11    99 b3 97 31 4d 8b 61 71 73 5d 47                ...1M.aqs]G    Encoded map path (start)
0x1c    73 6f 85 7b 65 6f 55 69 73 6f c5                so.{eoUiso.    Encoded map path (cont)
0x27    6f 65 5d 29 33 29 45 bb 63 69 6f                oe])3)E.cio    Encoded map path (cont)
0x32    49 73 6d 65 9b 73 2f 77 33 79                   Isme.s/w3y     Encoded map path (cont)
...

Decoded map path (printable chars only):
"y1Maqs]Gso{eoUisooe])3)EcioIsmes/w3yKaseo..."
```

### Key Observations

1. **Obfuscation Pattern:**
   - Both samples show a similar pattern where printable ASCII characters representing the map path are interspersed with non-printable bytes
   - The non-printable bytes serve as "junk" or "mask" bytes that obfuscate the true path
   - Pattern is not perfectly regular (not simply alternating)

2. **Common Artifacts in Decoded Paths:**
   - Leading garbage characters (e.g., "o" before "Maps", "y1" before "Maps")
   - Character substitutions (e.g., "Maqs" instead of "Maps", "]" appears in multiple positions)
   - Path components visible: "Maps/", "W3C", "miami", "w3", directory separators

3. **Null Terminators:**
   - Map path section ends with 0x00 byte
   - Additional settings data follows the map path

## Field Extraction

### 1. Game Name

**Location:** Offset 0x01 (after initial byte)

**Format:** Null-terminated ASCII string

**Example:**
```
Hex:  00 46 4c 4f 2d 53 54 52 45 41 4d 00 ...
      ^  ^----- Game Name -----^  ^
      |                            |
    Initial                   Null term
```

**Extracted:** `"FLO-STREAM"`

### 2. Map File Path

**Location:** Variable offset (~13 bytes after game name null terminator)

**Format:** Obfuscated/encoded string with alternating junk bytes

**Encoding Method:**

The map path is encoded by interleaving the actual path characters with "junk" or "mask" bytes. The encoding pattern appears to be:

- Real path characters appear at certain byte positions (mostly printable ASCII)
- Non-path bytes (junk/mask bytes) are interspersed between them
- The exact pattern may vary, but extracting only printable ASCII bytes yields a recognizable path

**Decoding Strategy:**

Extract all printable ASCII bytes (0x20-0x7E) from the encoded section until a null terminator or end marker.

**Example from replay_80852.w3g:**

```
Raw hex (starting at offset 0x1d):
6f c1 7f 4d fb 61 71 73 2f 57 33 43 6d 69 61 6d 71 69 6f 6f fb 73 5d ...

Byte analysis:
Pos  Hex  ASCII  Printable?
0    6f   'o'    ✓
1    c1   N/A    ✗
2    7f   N/A    ✗
3    4d   'M'    ✓
4    fb   N/A    ✗
5    61   'a'    ✓
6    71   'q'    ✓
7    73   's'    ✓
8    2f   '/'    ✓
9    57   'W'    ✓
10   33   '3'    ✓
11   43   'C'    ✓
...

Extracting all printable ASCII bytes:
"oMaqs/W3Cmiamqioos]w3c_s13/1_Co]oceameeIimm/w3!y..."
```

**Interpretation:**

The extracted string contains the map path, though some characters may be corrupted or extra characters may be present. Common patterns to look for:

- `Maps/` prefix (standard W3 map directory)
- `/` path separators
- `.w3x` or `.w3m` file extensions (map file types)

**Possible cleaned path:**
`Maps/W3C_miami_qos/w3c_s_13/1_Coconut_ee_Timm.w3x` (or similar)

## Implementation Notes

### Byte Offsets (Classic Format)

For `replay_80852.w3g`:

```
Field            | Offset | Value
-----------------|--------|----------------------------------
Initial byte     | 0x00   | 0x00
Game name start  | 0x01   | "FLO-STREAM"
Game name end    | 0x0b   | (null terminator at 0x0c)
Settings bytes   | 0x0d   | 00 01 03 79 07 01 01 01 01 f9 01 01 0d
Encoded map start| 0x1d   | 6f c1 7f 4d fb ...
```

### Extraction Algorithm (Pseudocode)

```rust
fn extract_game_name(encoded_settings: &[u8]) -> String {
    if encoded_settings[0] != 0x00 {
        return String::new();
    }

    // Find null terminator after byte 1
    for i in 1..encoded_settings.len() {
        if encoded_settings[i] == 0x00 {
            return String::from_utf8_lossy(&encoded_settings[1..i]).to_string();
        }
    }
    String::new()
}

fn extract_map_path(encoded_settings: &[u8]) -> String {
    // Skip past game name and settings bytes
    let start = find_encoded_map_start(encoded_settings);

    // Extract all printable ASCII bytes
    let mut path_chars = Vec::new();
    for &byte in &encoded_settings[start..] {
        if byte == 0x00 {
            break;
        }
        if byte >= 0x20 && byte <= 0x7E {
            path_chars.push(byte);
        }
    }

    String::from_utf8_lossy(&path_chars).to_string()
}

fn find_encoded_map_start(settings: &[u8]) -> usize {
    // Find game name end
    let game_name_end = settings.iter()
        .position(|&b| b == 0x00)
        .unwrap_or(0) + 1;

    // Skip ~12-15 bytes of settings data
    game_name_end + 13
}
```

## Reforged Format (GRBN)

### Status

The Reforged format uses a completely different structure for the decompressed data. It appears to use Protocol Buffers (protobuf) encoding rather than the Classic binary format.

**Evidence:**
```
First bytes of decompressed GRBN data:
08 02 10 00 1a f1 12 08 e0 d8 ac cc 85 a5 b4 bf d0 01 12 05 48 34 70 70 79 ...
```

The byte patterns (field tags like `08`, `10`, `12`, `1a`) are characteristic of protobuf encoding.

**Recommendation:** Use a protobuf decoder/schema to extract metadata from Reforged replays.

## Sample Data

### Sample 1: replay_80852.w3g (Classic, with game name)

**Full encoded_settings hex dump:**
```
0000: 00 46 4c 4f 2d 53 54 52  45 41 4d 00 00 01 03 79  |.FLO-STREAM....y|
0010: 07 01 01 01 01 f9 01 01  0d 6f c1 7f 4d fb 61 71  |.........o..M.aq|
0020: 73 2f 57 33 43 6d 69 61  6d 71 69 6f 6f fb 73 5d  |s/W3Cmiamqioo.s]|
0030: 77 33 63 5f 73 f7 31 33  2f 31 5f 43 6f 5d 6f 63  |w3c_s.13/1_Co]oc|
0040: 65 61 6d 65 65 c5 49 69  6d 6d 2f 77 33 21 79 01  |eamee.Iimm/w3!y.|
0050: 47 4d 4f 01 01 c5 fb 79  21 d9 71 39 0d 85 69 2f  |GMO....y!.q9..i/|
0060: 53 e5 6f 35 69 11 3d 3f  a1 ff f1 31 00 18 00 00  |S.o5i.=?...1....|
0070: 00 00 00 10 00 00 00 00  00                       |.........|
```

**Extracted fields:**
- **Host:** `Destiny#514792`
- **Game name:** `"FLO-STREAM"`
- **Map path (raw):** `"oMaqs/W3Cmiamqioos]w3c_s13/1_Co]oceameeIimm/w3!yGMOy!q9i/So5i=?"`
- **Map path (likely):** Path related to W3C/Miami maps, possibly `Maps/W3C_miami_*/w3c_s*_13/1_Coconut_*_*.w3x`

### Sample 2: replay_5000.w3g (Classic, without game name)

**Full encoded_settings hex dump:**
```
0000: 72 69 63 68 00 00 81 03  79 07 01 01 c1 07 a5 c1  |rich....y.......|
0010: 07 99 b3 97 31 4d 8b 61  71 73 5d 47 73 6f 85 7b  |....1M.aqs]Gso.{|
0020: 65 6f 55 69 73 6f c5 6f  65 5d 29 33 29 45 bb 63  |eoUiso.oe])3)E.c|
0030: 69 6f 49 73 6d 65 9b 73  2f 77 33 79 01 4b 0b 61  |ioIsme.s/w3y.K.a|
0040: 73 65 6f 01 01 d5 9b c5  8f 97 03 43 7d 47 e3 93  |seo........C}G..|
0050: 31 25 65 ad 8b 5d 1d 7f  7b 2d af f7 00 0c 00 00  |1%e..]..{-......|
0060: 00 00 00 00 00 a8 f8 12  00                       |.........|
```

**Extracted fields:**
- **Host:** `kaiseris`
- **Additional data:** `"rich"` (clan tag or similar)
- **Game name:** (empty - no custom game name)
- **Map path (raw):** `"y1Maqs]Gso{eoUisooe])3)EcioIsmes/w3yKaseo"`
- **Map path (likely):** Starts with "Maps/[something]", contains "/w3" pattern

**Notes:**
- In this replay, the encoded_settings starts with a copy of the additional_data field
- After two null bytes (0x00 0x00), the settings bytes begin
- The map path encoding follows the same pattern as Sample 1

## Format Differences: Classic vs Reforged

| Aspect           | Classic Format          | Reforged (GRBN)         |
|------------------|-------------------------|-------------------------|
| Game Record Magic| 0x00000110             | 0x00100208 (different)  |
| Encoding         | Custom binary + obfuscation | Protobuf-like         |
| Map Path         | Obfuscated ASCII string | Embedded in protobuf    |
| Game Name        | Null-terminated string  | Protobuf field          |

## Next Steps

1. **Classic Format:**
   - Implement the extraction algorithms in production code
   - Test with more replay samples to validate the encoding pattern
   - Identify any variations in the encoding scheme
   - Document the settings bytes (offsets 0x0d-0x19) if they contain useful data

2. **Reforged Format:**
   - Obtain or reverse-engineer the protobuf schema
   - Implement a proper protobuf parser for GRBN game records
   - Extract map name, game name, and other metadata fields

3. **Additional Metadata:**
   - Game speed settings
   - Observer settings
   - Map checksum
   - Difficulty level
   - Other game options

## Testing Files

- **Classic:** `/Users/felipeh/Development/w3g/tests/fixtures/replay_80852.w3g`
- **Reforged:** `/Users/felipeh/Development/w3g/replays/replay_6.w3g`

## References

- W3G Format Documentation (existing)
- Protocol Buffers Specification (for Reforged format)
- Community reverse engineering efforts (WC3 replay parsers)
