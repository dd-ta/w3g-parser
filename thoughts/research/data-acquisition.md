# Data Acquisition Report

## Source
- **API**: warcraft3.info
- **Endpoint**: `https://warcraft3.info/api/v1/replays/{id}/download`
- **Date**: 2025-11-25

## Replays Acquired

**Total: 27 replay files**

### ID Ranges Tested

| Range | IDs Tried | Success | Notes |
|-------|-----------|---------|-------|
| 1-10 | 1-10 | 10/10 | All successful |
| 100-150 | 100 | 0/1 | 404 Not Found |
| 1000-1004 | 1000-1004 | 5/5 | All successful |
| 5000-5002 | 5000-5002 | 3/3 | All successful |
| 10000-10002 | 10000-10002 | 3/3 | All successful |
| 50000-50002 | 50000-50002 | 3/3 | All successful |
| 100000-100002 | 100000-100002 | 3/3 | All successful |

### File Details

| File | Size (bytes) | Format Magic |
|------|--------------|--------------|
| replay_1.w3g | 312,597 | GRBN |
| replay_2.w3g | 334,677 | GRBN |
| replay_3.w3g | 295,819 | GRBN |
| replay_4.w3g | 310,026 | GRBN |
| replay_5.w3g | 298,096 | GRBN |
| replay_6.w3g | 325,758 | GRBN |
| replay_7.w3g | 358,224 | GRBN |
| replay_8.w3g | 303,907 | GRBN |
| replay_9.w3g | 355,887 | GRBN |
| replay_10.w3g | 447,077 | GRBN |
| replay_1000.w3g | 266,577 | GRBN |
| replay_1001.w3g | 229,776 | GRBN |
| replay_1002.w3g | 178,245 | GRBN |
| replay_1003.w3g | 354,858 | GRBN |
| replay_1004.w3g | 200,410 | GRBN |
| replay_5000.w3g | 100,646 | Warc (classic) |
| replay_5001.w3g | 161,754 | Warc (classic) |
| replay_5002.w3g | 120,129 | Warc (classic) |
| replay_10000.w3g | 53,886 | Warc (classic) |
| replay_10001.w3g | 102,014 | Warc (classic) |
| replay_10002.w3g | 36,656 | Warc (classic) |
| replay_50000.w3g | 209,116 | Warc (classic) |
| replay_50001.w3g | 538,892 | Warc (classic) |
| replay_50002.w3g | 373,287 | Warc (classic) |
| replay_100000.w3g | 395,885 | Warc (classic) |
| replay_100001.w3g | 154,121 | Warc (classic) |
| replay_100002.w3g | 200,258 | Warc (classic) |

## Key Observations

### Two Distinct File Formats Found

1. **GRBN Format** (IDs 1-1004 range)
   - Magic bytes: `GRBN` (0x4752424E)
   - Appears to be newer/Reforged format
   - 15 files acquired

2. **Classic W3G Format** (IDs 5000+)
   - Magic bytes: `Warcraft III recorded game` + 0x1A00
   - Classic/TFT format
   - 12 files acquired

### File Size Distribution

- **Smallest**: 36,656 bytes (replay_10002.w3g)
- **Largest**: 538,892 bytes (replay_50001.w3g)
- **Average**: ~230,000 bytes

### Issues Encountered

1. **Some IDs return 404**: ID 100 returned "Not Found" - not all IDs have replays
2. **File extension mismatch**: API serves files with `.nwg` extension in Content-Disposition header, but files are valid W3G/GRBN format

## Diversity Achieved

- [x] Multiple file sizes (36KB - 538KB range)
- [x] Two distinct binary formats (GRBN vs classic W3G)
- [x] Different ID ranges (early vs late uploads)
- [x] 27 total files exceeds 15+ goal

## Next Steps for Rex (Binary Analyzer)

The data is ready for reverse engineering. Two format variants exist:
1. Start with classic format (Warc magic) - likely more documented historically
2. Then analyze GRBN format - appears to be newer variant

Files located at: `/Users/felipeh/Development/w3g/replays/`
