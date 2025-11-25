# Header Analysis Report

**Analyst**: Rex (Binary Analysis Agent)
**Date**: 2025-11-25
**Files Analyzed**: 27 replay files (15 GRBN, 12 Classic)
**Method**: Ground-up reverse engineering via hex analysis

---

## Executive Summary

Through systematic hex dump analysis and cross-file comparison, I have mapped the complete header structures for both W3G replay format families:

1. **GRBN Format** (Warcraft III: Reforged): 128-byte fixed header + single zlib stream
2. **Classic Format**: 68-byte main header + variable-length data blocks with per-block headers

Key discoveries:
- Two completely different format architectures
- Classic format has two sub-variants (v26 and v10000+) with different block header sizes
- Both use little-endian byte order
- Both use standard zlib compression

---

## Methodology

### Step 1: Initial Format Identification

First hex dump of multiple files revealed two distinct magic signatures:

```bash
xxd -l 32 replay_1.w3g
xxd -l 32 replay_5000.w3g
```

**Finding**: Files split into two families based on first 4-28 bytes.

### Step 2: Cross-File Comparison

For each format, I dumped the first 256 bytes of 6+ files and compared byte-by-byte:

```bash
# GRBN files
xxd -l 256 replay_1.w3g
xxd -l 256 replay_2.w3g
xxd -l 256 replay_1000.w3g
...

# Classic files
xxd -l 256 replay_5000.w3g
xxd -l 256 replay_10000.w3g
xxd -l 256 replay_100000.w3g
...
```

**Finding**: Identified constant vs variable fields by seeing which bytes changed across files.

### Step 3: Size Field Correlation

Cross-referenced file system sizes with hex values:

```bash
ls -la replays/*.w3g  # Get actual file sizes
xxd -s 0x20 -l 4 replay_5000.w3g  # Compare with header values
```

**Finding**: Classic format stores file size at offset 0x20 (verified in 6 files).

### Step 4: Block Structure Analysis

Searched for zlib compression markers (0x78 0x9C, 0x78 0x01) to find data boundaries:

```bash
xxd replay_5000.w3g | grep "78.?01"
```

Then calculated block boundaries using the compressed size fields:

```bash
# First block header at 0x44, compressed size = 3193
# Second block should be at: 0x44 + 8 + 3193 = 0xCC5
xxd -s 0xCC5 -l 16 replay_5000.w3g
```

**Finding**: Block structure fully mapped with two variants.

---

## Detailed Findings

### GRBN Format Structure

**Discovery Process**:

1. All 15 GRBN files have identical bytes 0x00-0x17
2. Bytes 0x18-0x1F vary but follow small integer pattern (0-6, 0-1)
3. Bytes 0x24-0x27 vary widely - hypothesized as size field
4. Bytes 0x28-0x7F all zeros - padding
5. Byte 0x80 consistently 0x78 (zlib marker)

**Hex Evidence**:
```
replay_1.w3g:
00000000: 4752 424e 0200 0000 0b00 0000 00c8 0000  GRBN............
          ^^^^---- magic  ^^^^---- version=2
                         ^^^^---- unknown=11
                                   ^^^^---- unknown=51200

00000080: 789c cd96 4d4c 1341...  <-- zlib starts at fixed offset
```

### Classic Format Structure

**Discovery Process**:

1. 28-byte ASCII magic string always identical
2. Offset 0x1C always 0x44 = 68 - this is header size
3. Offset 0x20 matches file size - CONFIRMED with 6 files
4. Offset 0x34 varies between 26 and 10000+ - version number
5. Version number determines block header format

**File Size Verification**:
```
File                Actual Size   Header @0x20 (LE)   Match?
replay_5000.w3g     100,646       0x00018926          YES
replay_5001.w3g     161,754       0x000277DA          YES
replay_10000.w3g    53,886        0x0000D27E          YES
replay_50000.w3g    209,116       0x000330DC          YES
replay_100000.w3g   395,885       0x00060A6D          YES
replay_100001.w3g   154,121       0x00025A09          YES
```

### Block Format Discovery

**Type A (v26)** - Found by tracing block boundaries:
```
Offset 0x44: 79 0c 00 20 fe c0 86 26 78 01...
             ^^^^       ^^^^               -- 8-byte header
                  ^^^^                     -- compressed size = 3193
                       ^^^^                -- decompressed = 8192

Next block at 0x44 + 8 + 3193 = 0xCC5:
00000cc5: 4a 0a 00 20 c8 c9 f2 f4 78 01...  <-- CONFIRMED
```

**Type B (v10000+)** - Discovered different padding:
```
Offset 0x44: 26 0e 00 00 00 20 00 00 ff bb a2 82 78 01...
             ^^^^                                    -- 12-byte header
                  ^^^^                               -- padding
                       ^^^^                          -- compressed size = 3622
                            ^^^^                     -- decompressed = 8192

Next block at 0x44 + 12 + 3622 = 0xE76:
00000e76: 35 0e 00 00 00 20 00 00 75 ff 4d f6 78 01...  <-- CONFIRMED
```

---

## Confidence Assessment

| Finding | Evidence Strength | Confidence |
|---------|-------------------|------------|
| GRBN magic "GRBN" | 15/15 files | CONFIRMED |
| GRBN header = 128 bytes | All files, zlib at 0x80 | CONFIRMED |
| Classic magic string | 12/12 files | CONFIRMED |
| Classic header = 68 bytes | Field at 0x1C always 0x44 | CONFIRMED |
| Classic file size at 0x20 | 6/6 tested match | CONFIRMED |
| Block count at 0x2C | Correlates with file size | LIKELY |
| Duration at 0x3C | Values plausible (10-22 min) | LIKELY |
| Checksum fields | Varies per-file, unknown algorithm | UNKNOWN |

---

## Open Questions

### High Priority
1. **Block checksum algorithm**: 4 bytes at end of each block header - CRC32? custom?
2. **GRBN Unknown_3/4 fields**: What do values 0-6 and 0-1 represent?

### Medium Priority
3. **Classic flags at 0x38**: Why is high bit always set?
4. **GRBN Unknown_2 = 51200**: Chunk size? Why this specific value?

### Low Priority
5. **Decompressed content structure**: What's inside the zlib data?
6. **Player data location**: Where are player names stored?

---

## Recommendations for Next Analysis

### Phase 2: Decompressed Data Analysis

1. **Decompress first block** of a classic file
2. **Analyze decompressed content** - likely game metadata
3. **Find player name strings** - look for null-terminated ASCII
4. **Map game events** - timestamps and action types

### Phase 3: GRBN Deep Dive

1. **Decompress GRBN zlib stream**
2. **Compare decompressed structure** to classic format
3. **Identify metadata JSON** - GRBN may use different encoding

### Tools Needed

```bash
# Decompress classic block
python3 -c "import zlib; data=open('replay_5000.w3g','rb').read()[0x4C:0x4C+3193]; print(zlib.decompress(data)[:200])"

# Decompress GRBN stream
python3 -c "import zlib; data=open('replay_1.w3g','rb').read()[0x80:]; print(zlib.decompress(data)[:200])"
```

---

## Appendix: Raw Hex Dumps

### GRBN Sample (replay_1.w3g)
```
00000000: 4752 424e 0200 0000 0b00 0000 00c8 0000  GRBN............
00000010: 0000 0000 0000 0000 0100 0000 0000 0000  ................
00000020: 0000 0000 a7d2 1200 0000 0000 0000 0000  ................
00000030: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000040: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000050: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000060: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000070: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000080: 789c cd96 4d4c 1341 14c7 df9b b6cb b010  x...ML.A........
```

### Classic Sample (replay_5000.w3g, v26)
```
00000000: 5761 7263 7261 6674 2049 4949 2072 6563  Warcraft III rec
00000010: 6f72 6465 6420 6761 6d65 1a00 4400 0000  orded game..D...
00000020: 2689 0100 0100 0000 6833 0400 2200 0000  &.......h3.."...
00000030: 5058 3357 1a00 0000 ab17 0080 68ed 0900  PX3W........h...
00000040: 34dc ac44 790c 0020 fec0 8626 7801 8c99  4..Dy.. ...&x...
```

### Classic Sample (replay_100000.w3g, v10036)
```
00000000: 5761 7263 7261 6674 2049 4949 2072 6563  Warcraft III rec
00000010: 6f72 6465 6420 6761 6d65 1a00 4400 0000  orded game..D...
00000020: 6d0a 0600 0100 0000 f422 0d00 6a00 0000  m........"..j...
00000030: 5058 3357 3427 0000 e317 0080 2afe 1300  PX3W4'......*...
00000040: 8c93 0734 0b0f 0000 0020 0000 39f9 22f7  ...4..... ..9.".
00000050: 7801 d459 7958 53d7 12bf 492e 1570 634d  x..YyXS...I..pcM
```

---

## Conclusion

The header analysis phase is complete. Both format families are now well-understood at the header level. The next phase should focus on decompressed content analysis to understand game metadata and action data structures.

**Files Updated**:
- `/Users/felipeh/Development/w3g/FORMAT.md` - Complete format specification
- `/Users/felipeh/Development/w3g/thoughts/research/header-analysis.md` - This report
