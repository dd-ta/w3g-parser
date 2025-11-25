# Phase 2: Header Parsing - Implementation Notes

## Metadata
- **Agent**: Cody
- **Date**: 2025-11-25
- **Plan Reference**: `thoughts/plans/header-parsing.md` (Phase 2)
- **Status**: Complete

## Summary

Successfully implemented header parsing for both GRBN (Reforged) and Classic replay formats. All 27 test replays parse correctly.

## What Was Implemented

### Files Created

1. **`src/format.rs`** - Format detection and type definitions
   - `ReplayFormat` enum: `Grbn` and `Classic` variants
   - `ClassicVersion` enum: `TypeA` and `TypeB` variants
   - `detect_format()` function for automatic format detection
   - Magic byte constants: `GRBN_MAGIC`, `CLASSIC_MAGIC`
   - Threshold constant: `CLASSIC_TYPE_B_THRESHOLD = 10000`

2. **`src/header/mod.rs`** - Unified header interface
   - `Header` enum wrapping both format types
   - `Header::parse()` for automatic detection and parsing
   - Common interface methods: `data_offset()`, `decompressed_size()`, `format()`, etc.

3. **`src/header/grbn.rs`** - GRBN header parser (128 bytes)
   - `GrbnHeader` struct with all header fields
   - `GrbnHeader::parse()` with magic validation
   - Constants: `GRBN_HEADER_SIZE`, `GRBN_DATA_OFFSET`

4. **`src/header/classic.rs`** - Classic header parser (68 bytes)
   - `ClassicHeader` struct with all header fields
   - `ClassicHeader::parse()` with magic validation
   - `version_type()` to determine TypeA vs TypeB
   - `duration_parts()` and `duration_string()` utilities
   - Constants: `CLASSIC_HEADER_SIZE`, `CLASSIC_DATA_OFFSET`, `TFT_SUB_HEADER_MAGIC`

5. **`tests/header_integration.rs`** - Integration tests
   - Tests for all 15 GRBN replays
   - Tests for 6 Classic TypeA replays (build 26)
   - Tests for 6 Classic TypeB replays (build 10000+)
   - Error handling tests (invalid magic, truncated files, empty files)
   - File size field validation for all Classic replays
   - Zlib marker verification for all replays

### Files Modified

1. **`src/lib.rs`** - Added new module declarations and re-exports
   - Added `pub mod format` and `pub mod header`
   - Re-exported `Header`, `ReplayFormat`, `ClassicVersion`, `detect_format`
   - Updated documentation example

## Validation Criteria Status

| Criteria | Status | Notes |
|----------|--------|-------|
| GrbnHeader::parse reads all 15 GRBN replays | PASS | All GRBN replays parse successfully |
| ClassicHeader::parse reads all 12 Classic replays | PASS | All Classic replays parse successfully |
| Format detection routes correctly | PASS | All 27 files detected correctly |
| File size field matches actual (Classic) | PASS | Verified for all 12 Classic files |
| Build version identifies TypeA vs TypeB | PASS | v26 -> TypeA, v10000+ -> TypeB |
| Invalid magic returns ParserError::InvalidMagic | PASS | Tested with invalid data |
| Truncated files return ParserError::UnexpectedEof | PASS | Tested with short data |

## Test Results

```
running 75 tests (unit)      ... ok
running 17 tests (integration) ... ok
running 19 tests (doc)       ... ok

Total: 111 tests passed
```

Clippy: No warnings

## Deviations from Plan

### Minor Differences

1. **Duration value for replay_5001**: FORMAT.md listed 1,053,488 ms but actual file contains 1,054,000 ms. Test updated to use actual value.

2. **Additional utility methods**: Added several convenience methods not in original plan:
   - `ClassicHeader::is_type_a()`, `is_type_b()`
   - `ClassicHeader::duration_parts()`, `duration_string()`
   - `Header::is_grbn()`, `is_classic()`
   - `Header::as_grbn()`, `as_classic()`

3. **Constants exposed**: Added public constants for header sizes and data offsets to support Phase 3 decompression.

## Observations for Phase 3

### GRBN Format
- Zlib marker observed at offset 0x80: `0x78 0x9C` (default compression)
- Single continuous stream, no blocks
- `decompressed_size` field values range from ~1M to ~1.5M bytes

### Classic Format
- First block zlib marker at offset 0x44 + block_header_size
- Both `0x78 0x01` (fast) and `0x78 0x9C` (default) compression observed
- Block count field available for iteration
- TypeA: 8-byte block headers, TypeB: 12-byte block headers

### Block Header Access
- `ClassicHeader::block_header_size()` returns 8 or 12 based on version
- `ClassicVersion::block_header_size()` available directly from enum

## Files Ready for Phase 3

The following are accessible for the decompression phase:

```rust
// Parse header and get data offset
let header = Header::parse(&data)?;
let compressed_data_start = header.data_offset();
let expected_decompressed_size = header.decompressed_size();

// For Classic, get block info
if let Header::Classic(h) = &header {
    let block_count = h.block_count;
    let block_header_size = h.block_header_size();
}
```

## Code Quality

- No `unwrap()` in library code (only in tests)
- All public types and functions documented
- Comprehensive error handling with descriptive messages
- `#![deny(missing_docs)]` enforced
- `#![deny(unsafe_code)]` enforced
- All clippy warnings addressed
