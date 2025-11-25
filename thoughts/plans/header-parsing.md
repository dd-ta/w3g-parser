# Plan: W3G Header Parsing and Decompression

## Metadata
- **Agent**: Archie
- **Date**: 2025-11-25
- **Research Used**:
  - `FORMAT.md` - Complete format specification
  - `thoughts/research/header-analysis.md` - Rex's binary analysis report
  - `thoughts/research/data-acquisition.md` - Test replay inventory
- **Status**: Draft

## Overview

This plan implements the foundational W3G replay parser in Rust, covering:
1. Project setup and binary reading utilities
2. Header parsing for both GRBN (Reforged) and Classic formats
3. Decompression of replay data (single-stream and block-based)

Based on Rex's research, we handle **two format families** with **three sub-variants**:
- **GRBN Format**: 128-byte header + single zlib stream
- **Classic Type A**: 68-byte header + 8-byte block headers (version 26)
- **Classic Type B**: 68-byte header + 12-byte block headers (version 10000+)

## Prerequisites

- Rust toolchain installed (rustup)
- 27 test replays available in `/replays/`
- FORMAT.md verified by Rex's analysis

---

## Phase 1: Project Setup and Foundation

### Goal
Establish the Cargo project structure with core binary reading utilities and error types.

### Files to Create/Modify

- `Cargo.toml` - Project manifest with dependencies
- `src/lib.rs` - Library root with module declarations
- `src/error.rs` - Custom error types following spec.md error hierarchy
- `src/binary.rs` - Binary reading utilities (little-endian readers)

### Implementation Steps

1. **Initialize Cargo project**
   ```bash
   cargo init --lib w3g-parser
   ```

2. **Configure Cargo.toml**
   - Add `flate2` crate for zlib decompression
   - Add `thiserror` for error derive macros
   - Set edition to 2021, appropriate metadata

3. **Create error module** (`src/error.rs`)
   - Define `ParserError` enum with variants:
     - `IoError(std::io::Error)` - File I/O failures
     - `InvalidMagic { expected: Vec<u8>, found: Vec<u8> }` - Wrong magic bytes
     - `InvalidHeader { reason: String }` - Malformed header
     - `DecompressionError { reason: String }` - Zlib failures
     - `UnexpectedEof { expected: usize, available: usize }` - Truncated data
   - Implement `std::error::Error` and `From<std::io::Error>`
   - Define `type Result<T> = std::result::Result<T, ParserError>`

4. **Create binary utilities** (`src/binary.rs`)
   - `read_u16_le(bytes: &[u8], offset: usize) -> Result<u16>`
   - `read_u32_le(bytes: &[u8], offset: usize) -> Result<u32>`
   - `read_bytes(bytes: &[u8], offset: usize, len: usize) -> Result<&[u8]>`
   - `read_string(bytes: &[u8], offset: usize, max_len: usize) -> Result<String>`
   - All functions return `ParserError::UnexpectedEof` on bounds failure

5. **Set up lib.rs**
   - Declare modules: `error`, `binary`
   - Re-export public types

### Validation Criteria

- [ ] `cargo build` succeeds with no warnings
- [ ] `cargo test` passes (basic unit tests for binary readers)
- [ ] Can read u16/u32 values from byte arrays with correct endianness
- [ ] Error types implement `std::error::Error` and `Display`
- [ ] Binary readers return errors on out-of-bounds access

### Estimated Complexity
**Low** - Standard Rust project setup with simple utility functions.

---

## Phase 2: Header Parsing

### Goal
Parse headers for both GRBN and Classic formats, detecting format type automatically from magic bytes.

### Files to Create/Modify

- `src/format.rs` - Format detection and routing
- `src/header/mod.rs` - Header module root
- `src/header/grbn.rs` - GRBN header parser (128 bytes)
- `src/header/classic.rs` - Classic header parser (68 bytes)
- `src/lib.rs` - Add header module

### Implementation Steps

1. **Define format types** (`src/format.rs`)
   ```rust
   pub enum ReplayFormat {
       Grbn,      // "GRBN" magic, Reforged
       Classic,   // "Warcraft III recorded game" magic
   }

   pub enum ClassicVersion {
       TypeA,     // Version 26, 8-byte block headers
       TypeB,     // Version 10000+, 12-byte block headers
   }
   ```

2. **Implement format detection**
   - Check first 4 bytes for "GRBN" (0x47 0x52 0x42 0x4E)
   - Check first 28 bytes for "Warcraft III recorded game\x1A\x00"
   - Return `ParserError::InvalidMagic` if neither matches

3. **Create GRBN header struct** (`src/header/grbn.rs`)
   Based on FORMAT.md offsets:
   ```rust
   pub struct GrbnHeader {
       pub magic: [u8; 4],           // 0x00: "GRBN"
       pub version: u32,             // 0x04: Always 2
       pub unknown_1: u32,           // 0x08: Always 11
       pub unknown_2: u32,           // 0x0C: Always 51200
       pub reserved_1: [u8; 8],      // 0x10: Zeros
       pub unknown_3: u32,           // 0x18: 0-6
       pub unknown_4: u32,           // 0x1C: 0 or 1
       pub reserved_2: [u8; 4],      // 0x20: Zeros
       pub decompressed_size: u32,   // 0x24: Size after decompression
       pub reserved_3: [u8; 88],     // 0x28-0x7F: Zeros
   }
   // Total: 128 bytes
   // Data starts at offset 0x80
   ```

4. **Create Classic header struct** (`src/header/classic.rs`)
   Based on FORMAT.md offsets:
   ```rust
   pub struct ClassicHeader {
       pub magic: [u8; 28],          // 0x00: "Warcraft III recorded game\x1A\x00"
       pub header_size: u32,         // 0x1C: Always 68 (0x44)
       pub file_size: u32,           // 0x20: Total file size
       pub header_version: u32,      // 0x24: Always 1
       pub decompressed_size: u32,   // 0x28: Size after decompression
       pub block_count: u32,         // 0x2C: Number of data blocks
       pub sub_header_magic: [u8; 4],// 0x30: "PX3W" (W3XP reversed)
       pub build_version: u32,       // 0x34: 26 or 10032-10036
       pub flags: u32,               // 0x38: Unknown, high bit set
       pub duration_ms: u32,         // 0x3C: Game duration in milliseconds
       pub checksum: [u8; 4],        // 0x40: Unknown checksum
   }
   // Total: 68 bytes
   // Data blocks start at offset 0x44
   ```

5. **Implement header parsing functions**
   - `GrbnHeader::parse(bytes: &[u8]) -> Result<GrbnHeader>`
   - `ClassicHeader::parse(bytes: &[u8]) -> Result<ClassicHeader>`
   - Both validate magic bytes and return structured data
   - `ClassicHeader::version_type(&self) -> ClassicVersion` - Returns TypeA if build_version < 10000

6. **Create unified header enum** (`src/header/mod.rs`)
   ```rust
   pub enum Header {
       Grbn(GrbnHeader),
       Classic(ClassicHeader),
   }

   impl Header {
       pub fn parse(bytes: &[u8]) -> Result<Header>;
       pub fn data_offset(&self) -> usize;  // 0x80 for GRBN, 0x44 for Classic
       pub fn decompressed_size(&self) -> u32;
   }
   ```

### Validation Criteria

- [ ] `GrbnHeader::parse` correctly reads all 15 GRBN test replays
- [ ] `ClassicHeader::parse` correctly reads all 12 Classic test replays
- [ ] Format detection routes to correct parser automatically
- [ ] File size field (Classic 0x20) matches actual file size for all test files
- [ ] Build version correctly identifies TypeA (v26) vs TypeB (v10000+)
- [ ] Invalid magic bytes return `ParserError::InvalidMagic`
- [ ] Truncated files return `ParserError::UnexpectedEof`

### Test Cases
```rust
#[test]
fn test_grbn_header() {
    let data = std::fs::read("replays/replay_1.w3g").unwrap();
    let header = Header::parse(&data).unwrap();
    assert!(matches!(header, Header::Grbn(_)));
    assert_eq!(header.data_offset(), 0x80);
}

#[test]
fn test_classic_header_type_a() {
    let data = std::fs::read("replays/replay_5000.w3g").unwrap();
    let header = Header::parse(&data).unwrap();
    if let Header::Classic(h) = header {
        assert_eq!(h.build_version, 26);
        assert_eq!(h.file_size, 100_646);
    }
}

#[test]
fn test_classic_header_type_b() {
    let data = std::fs::read("replays/replay_100000.w3g").unwrap();
    let header = Header::parse(&data).unwrap();
    if let Header::Classic(h) = header {
        assert_eq!(h.build_version, 10036);
        assert_eq!(h.file_size, 395_885);
    }
}
```

### Estimated Complexity
**Medium** - Multiple structures to implement with careful offset alignment.

---

## Phase 3: Decompression

### Goal
Decompress replay data for both GRBN (single zlib stream) and Classic (block-based zlib) formats.

### Files to Create/Modify

- `src/decompress/mod.rs` - Decompression module root
- `src/decompress/grbn.rs` - Single-stream zlib decompression
- `src/decompress/classic.rs` - Block-based decompression with Type A/B support
- `src/lib.rs` - Add decompress module

### Implementation Steps

1. **GRBN decompression** (`src/decompress/grbn.rs`)
   - Simple: single zlib stream starting at offset 0x80
   ```rust
   pub fn decompress_grbn(data: &[u8], header: &GrbnHeader) -> Result<Vec<u8>> {
       use flate2::read::ZlibDecoder;
       let compressed = &data[0x80..];
       let mut decoder = ZlibDecoder::new(compressed);
       let mut decompressed = Vec::with_capacity(header.decompressed_size as usize);
       decoder.read_to_end(&mut decompressed)?;
       Ok(decompressed)
   }
   ```

2. **Define block header structures** (`src/decompress/classic.rs`)
   ```rust
   // Type A: 8-byte block header (version 26)
   pub struct BlockHeaderA {
       pub compressed_size: u16,    // Offset 0
       pub decompressed_size: u16,  // Offset 2, always 0x2000
       pub checksum: [u8; 4],       // Offset 4
   }
   // Compressed data follows immediately (8 + compressed_size bytes per block)

   // Type B: 12-byte block header (version 10000+)
   pub struct BlockHeaderB {
       pub compressed_size: u16,    // Offset 0
       pub padding_1: u16,          // Offset 2, zeros
       pub decompressed_size: u16,  // Offset 4, always 0x2000
       pub padding_2: u16,          // Offset 6, zeros
       pub checksum: [u8; 4],       // Offset 8
   }
   // Compressed data follows immediately (12 + compressed_size bytes per block)
   ```

3. **Implement block iteration**
   ```rust
   pub struct BlockIterator<'a> {
       data: &'a [u8],
       offset: usize,
       blocks_remaining: u32,
       version: ClassicVersion,
   }

   impl<'a> Iterator for BlockIterator<'a> {
       type Item = Result<&'a [u8]>;  // Returns compressed data slice
   }
   ```

4. **Implement Classic decompression**
   ```rust
   pub fn decompress_classic(data: &[u8], header: &ClassicHeader) -> Result<Vec<u8>> {
       let version = header.version_type();
       let block_header_size = match version {
           ClassicVersion::TypeA => 8,
           ClassicVersion::TypeB => 12,
       };

       let mut result = Vec::with_capacity(header.decompressed_size as usize);
       let mut offset = 0x44;  // Data starts after 68-byte header

       for _ in 0..header.block_count {
           let compressed_size = read_u16_le(data, offset)?;
           let compressed_data = &data[offset + block_header_size..
                                        offset + block_header_size + compressed_size as usize];

           let mut decoder = ZlibDecoder::new(compressed_data);
           decoder.read_to_end(&mut result)?;

           offset += block_header_size + compressed_size as usize;
       }

       Ok(result)
   }
   ```

5. **Create unified decompression interface**
   ```rust
   pub fn decompress(data: &[u8], header: &Header) -> Result<Vec<u8>> {
       match header {
           Header::Grbn(h) => decompress_grbn(data, h),
           Header::Classic(h) => decompress_classic(data, h),
       }
   }
   ```

6. **Handle decompression errors gracefully**
   - Wrap `flate2` errors in `ParserError::DecompressionError`
   - Validate decompressed size matches header expectation (warning if mismatch)
   - Handle truncated compressed data

### Validation Criteria

- [ ] GRBN decompression produces non-empty output for all 15 GRBN replays
- [ ] Classic Type A decompression works for replays with build version 26
- [ ] Classic Type B decompression works for replays with build version 10000+
- [ ] Decompressed size approximately matches header's decompressed_size field
- [ ] Block iteration correctly chains all blocks (no gaps, no overlaps)
- [ ] First 4 bytes of GRBN data show zlib marker (0x78 0x9C or 0x78 0x01)
- [ ] Each Classic block shows zlib marker at expected offset

### Test Cases
```rust
#[test]
fn test_grbn_decompress() {
    let data = std::fs::read("replays/replay_1.w3g").unwrap();
    let header = Header::parse(&data).unwrap();
    let decompressed = decompress(&data, &header).unwrap();

    // replay_1 decompressed size from FORMAT.md: 1,233,575 bytes
    assert!(decompressed.len() > 1_000_000);
    assert!(decompressed.len() < 1_500_000);
}

#[test]
fn test_classic_type_a_decompress() {
    let data = std::fs::read("replays/replay_5000.w3g").unwrap();
    let header = Header::parse(&data).unwrap();
    let decompressed = decompress(&data, &header).unwrap();

    // Should decompress successfully
    assert!(!decompressed.is_empty());
}

#[test]
fn test_classic_type_b_decompress() {
    let data = std::fs::read("replays/replay_100000.w3g").unwrap();
    let header = Header::parse(&data).unwrap();
    let decompressed = decompress(&data, &header).unwrap();

    // Should decompress successfully
    assert!(!decompressed.is_empty());
}

#[test]
fn test_all_replays_decompress() {
    for entry in std::fs::read_dir("replays").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map(|e| e == "w3g").unwrap_or(false) {
            let data = std::fs::read(&path).unwrap();
            let header = Header::parse(&data).unwrap();
            let result = decompress(&data, &header);
            assert!(result.is_ok(), "Failed to decompress {:?}", path);
        }
    }
}
```

### Estimated Complexity
**Medium** - Requires handling two different block structures and zlib decompression.

---

## Edge Cases

### Invalid/Corrupted Files
- **Empty file**: Return `InvalidHeader` with descriptive message
- **Truncated header**: Return `UnexpectedEof` with expected vs available bytes
- **Unknown magic**: Return `InvalidMagic` with hex dump of found bytes
- **Corrupted zlib**: Return `DecompressionError` with zlib error message

### Version Edge Cases
- **Build version exactly 10000**: Treat as Type B (threshold is < 10000 for Type A)
- **Unknown build versions**: Parse as Type B if >= 10000, Type A otherwise
- **Future GRBN versions**: Parse header, warn if version field != 2

### Size Mismatches
- **Decompressed size mismatch**: Log warning, continue (don't fail)
- **File size mismatch (Classic)**: Log warning for debugging
- **Block count exceeds file**: Return `UnexpectedEof`

## Risks

### Risk 1: Undiscovered format variants
- **Mitigation**: Test against all 27 replays before marking complete
- **Fallback**: Add `Unknown` variant to format enum for future discovery

### Risk 2: Zlib decompression failures
- **Mitigation**: Use robust `flate2` library, test with all replays
- **Fallback**: Add permissive mode that skips bad blocks

### Risk 3: Block boundary calculation errors
- **Mitigation**: Verify block chain with hex dumps during implementation
- **Fallback**: Add integrity checks comparing expected vs actual offsets

## Success Criteria

The implementation plan succeeds when:

1. **All 27 test replays parse without errors**
   - 15 GRBN replays decompress successfully
   - 12 Classic replays decompress successfully (6 Type A, 6 Type B)

2. **Header data matches FORMAT.md predictions**
   - File sizes match (Classic)
   - Build versions detected correctly
   - Decompressed sizes approximately correct

3. **Code quality meets spec.md standards**
   - No `unwrap()` in library code (proper error handling)
   - All public types documented
   - Tests cover happy path and error cases

4. **Foundation ready for Phase 4 (Decompressed Data)**
   - Decompressed bytes available for next analysis phase
   - Header metadata accessible for game info extraction

---

## Future Work (Phase 4+)

This plan covers the foundation. Future phases will address:

1. **Decompressed Data Analysis** (Phase 4)
   - Analyze structure of decompressed content
   - Find player names and game metadata
   - Identify action blocks

2. **Action Parsing** (Phase 5)
   - Parse individual game actions
   - Build action type registry
   - Handle unknown actions gracefully

3. **CLI Interface** (Phase 6)
   - Implement commands from spec.md
   - JSON/YAML output formatters
   - Batch processing support

---

## Appendix: Key Offsets Reference

### GRBN Format
| Offset | Size | Field |
|--------|------|-------|
| 0x00 | 4 | Magic "GRBN" |
| 0x04 | 4 | Version (2) |
| 0x24 | 4 | Decompressed size |
| 0x80 | - | Zlib data start |

### Classic Format
| Offset | Size | Field |
|--------|------|-------|
| 0x00 | 28 | Magic string |
| 0x1C | 4 | Header size (68) |
| 0x20 | 4 | File size |
| 0x28 | 4 | Decompressed size |
| 0x2C | 4 | Block count |
| 0x34 | 4 | Build version |
| 0x3C | 4 | Duration (ms) |
| 0x44 | - | First block |

### Block Headers
| Type | Size | Compressed Size Offset | Decompressed Size Offset |
|------|------|------------------------|--------------------------|
| A (v26) | 8 | 0 | 2 |
| B (v10000+) | 12 | 0 | 4 |
