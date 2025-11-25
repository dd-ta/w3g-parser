# W3G Parser Progress

**Last Updated**: 2025-11-25
**Current Phase**: PROJECT COMPLETE
**Overall Status**: All 6 phases complete - CLI Interface functional
**Philosophy**: Ground-up reverse engineering (see PHILOSOPHY.md)

## RPI Workflow Status

```
[x] Setup -> [x] Data Acquisition -> [x] Binary Analysis -> [x] Plan (P1-3) -> [x] Implement (P1-3) -> [x] Validate (P1-3)
                                                         -> [x] Plan (P4)   -> [x] Implement (P4)   -> [x] Validate (P4)
                                                         -> [x] Plan (P5)   -> [x] Implement (P5)   -> [x] Validate (P5)
                                                         -> [x] Plan (P6)   -> [x] Implement (P6)   -> [x] Validate (P6) -> COMPLETE!
```

**Project Status**: Complete - All phases implemented and validated

## Current Status

**Validation Reports**:
- `thoughts/validation/phases-1-3.md` - Header parsing and decompression
- `thoughts/validation/phase-4.md` - Decompressed data parsing
- `thoughts/validation/phase-5.md` - Action parsing
- `thoughts/validation/phase-6.md` - CLI Interface

All 6 phases have been completed and validated:
- Parse successfully (header extraction)
- Decompress successfully (zlib decompression)
- Game record headers extracted
- Player names parsed (readable UTF-8)
- TimeFrame iteration working
- Action parsing (Selection, Ability, Movement, Hotkey)
- CLI Interface (info, parse, validate, batch commands)

### Test Summary
```
Unit tests:         159 passed
Integration tests:   53 passed
Doc tests:           25 passed (8 ignored)
-----------------------------------
Total:              237 tests passing

Replay validation:  24/27 passed (88.9%)
```

## Major Discovery: GRBN Format Structure

During Phase 3 implementation, a significant discovery was made:

**GRBN files contain embedded Classic replays!**

```
+-------------------+
| GRBN Header       |  128 bytes (0x00-0x7F)
+-------------------+
| Metadata zlib     |  Small zlib stream at 0x80
|                   |  (Protobuf-encoded, ~2-3KB)
+-------------------+
| Zero padding      |  Variable size
+-------------------+
| Classic Header    |  68 bytes (variable offset)
+-------------------+
| Classic Blocks    |  Block-based zlib compression
+-------------------+
```

This means GRBN (Reforged) replays wrap Classic replay data with additional metadata.

## Completed Phases

### Phase 1: Project Setup and Foundation (2025-11-25)
- [x] Cargo project with `flate2` and `thiserror`
- [x] `ParserError` enum with 5 variants
- [x] Binary utilities: `read_u16_le`, `read_u32_le`, `read_bytes`, `read_string`
- [x] All lints enabled: `deny(missing_docs)`, `deny(unsafe_code)`, `warn(clippy::pedantic)`
- **Implementation Notes**: `thoughts/impl/phase-1-foundation.md`

### Phase 2: Header Parsing (2025-11-25)
- [x] Format detection via magic bytes (GRBN vs Classic)
- [x] `GrbnHeader` parser (128 bytes)
- [x] `ClassicHeader` parser (68 bytes)
- [x] Version type detection (TypeA v26 vs TypeB v10000+)
- [x] All 27 replays parse successfully
- **Implementation Notes**: `thoughts/impl/phase-2-headers.md`

### Phase 3: Decompression (2025-11-25)
- [x] GRBN decompression (metadata + embedded Classic)
- [x] Classic Type A decompression (8-byte block headers)
- [x] Classic Type B decompression (12-byte block headers)
- [x] All 27 replays decompress successfully
- **Implementation Notes**: `thoughts/impl/phase-3-decompression.md`
- **DISCOVERY**: GRBN format contains embedded Classic replays

### Validation Phases 1-3 (2025-11-25)
- [x] All Phase 1-3 criteria verified
- [x] 143 tests passing
- [x] No clippy warnings
- [x] No unwrap() in library code
- [x] Full documentation coverage
- **Validation Report**: `thoughts/validation/phases-1-3.md`

### Phase 4: Decompressed Data Parsing (2025-11-25)
- [x] Game record header parsing (0x10 0x01 0x00 0x00 magic)
- [x] Player slot records (0x16 marker)
- [x] Slot records (0x19 marker)
- [x] TimeFrame iterator (0x1F, 0x1E markers)
- [x] Checksum records (0x22)
- [x] Chat messages (0x20)
- [x] Leave records (0x17)
- [x] GRBN embedded Classic detection
- **Implementation Notes**: `thoughts/impl/phase-4-records.md`

### Validation Phase 4 (2025-11-25)
- [x] All Phase 4 criteria verified
- [x] 160 tests passing
- [x] Player names verified as readable UTF-8
- [x] TimeFrame iteration working for all replays
- [x] No unwrap() in library code
- **Validation Report**: `thoughts/validation/phase-4.md`

### Phase 5: Action Parsing (2025-11-25)
- [x] Action struct with player_id, action_type, timestamp
- [x] ActionType enum (Selection, Ability, Movement, Hotkey, Unknown)
- [x] ActionIterator parses TimeFrame data
- [x] 0x16 selection actions parsed
- [x] AbilityCode FourCC with reverse byte order
- [x] 0x1A 0x19 direct ability parsed (14-byte format)
- [x] 0x1A 0x00 ability with selection parsed (variable length)
- [x] 0x0F 0x00 instant ability parsed (18-byte format)
- [x] 0x00 0x0D movement parsed (28-byte format)
- [x] IEEE 754 coordinates extracted
- [x] 0x17 hotkey operations parsed
- **Implementation Notes**: `thoughts/impl/phase-5-actions.md`

### Validation Phase 5 (2025-11-25)
- [x] All Phase 5 criteria verified
- [x] 237 tests passing
- [x] No unwrap() in library code
- [x] Full documentation coverage
- **Validation Report**: `thoughts/validation/phase-5.md`

### Phase 6: CLI Interface (2025-11-25)
- [x] Added clap, serde, serde_json dependencies
- [x] Created `src/bin/w3g-parser.rs` binary
- [x] Implemented `info` subcommand (display replay metadata)
- [x] Implemented `parse` subcommand (JSON and pretty output)
- [x] Implemented `validate` subcommand (exit codes 0/1)
- [x] Implemented `batch` subcommand (directory processing)
- [x] `--help` and `--version` working
- [x] All subcommands have help text
- **Implementation Notes**: `thoughts/impl/phase-6-cli.md`

### Validation Phase 6 (2025-11-25)
- [x] All Phase 6 criteria verified
- [x] 24/27 replays validate successfully (88.9%)
- [x] All 4 commands functional
- [x] JSON output parses with jq
- [x] Exit codes correct for scripting
- **Validation Report**: `thoughts/validation/phase-6.md`

## Previous Completed Work

### Project Setup (2025-11-25)
- [x] spec.md, FORMAT.md, PHILOSOPHY.md, AGENTS.md, RPI.md
- [x] thoughts/ directory structure
- [x] Autonomous scripts

### Data Acquisition (2025-11-25)
- [x] 27 replay files from warcraft3.info
- [x] Documented in `thoughts/research/data-acquisition.md`

### Binary Analysis (2025-11-25)
- [x] GRBN header fully mapped
- [x] Classic header fully mapped
- [x] Both block formats documented
- [x] FORMAT.md complete
- [x] Documented in `thoughts/research/header-analysis.md`

### Planning (2025-11-25)
- [x] Implementation plan for Phases 1-3
- [x] Plan artifact: `thoughts/plans/header-parsing.md`
- [x] Implementation plan for Phase 4 (Decompressed Data Parsing)
- [x] Plan artifact: `thoughts/plans/decompressed-parsing.md`
- [x] Implementation plan for Phase 5 (Action Parsing)
- [x] Plan artifact: `thoughts/plans/action-parsing.md`
- [x] Implementation plan for Phase 6 (CLI Interface)
- [x] Plan artifact: `thoughts/plans/cli-interface.md`

## Blockers

*None*

## Decisions Log

### 2025-11-25: GRBN Embedded Classic Discovery
- **Context**: Original plan assumed GRBN was single zlib stream
- **Discovery**: GRBN actually contains metadata zlib + embedded Classic replay
- **Impact**: Modified decompression approach to handle two-phase structure
- **Evidence**: replay_1.w3g has Classic header at offset 0xC880

### 2025-11-25: Ground-Up Reverse Engineering
- **Decision**: Reset to ground-up reverse engineering
- **Rationale**: The goal is to learn RE, not copy answers

### 2025-11-25: Implementation Language
- **Decision**: Use Rust
- **Rationale**: Better binary handling, stronger type system

## Future Enhancements (Optional)

The core project is complete. These are optional future improvements:

- [ ] Lossy UTF-8 conversion for international player names (fixes 3 failing replays)
- [ ] Enhanced action analysis (APM calculation)
- [ ] Additional output formats (YAML, CSV)
- [ ] Performance benchmarking
- [ ] Package distribution (Homebrew, crates.io)

## Metrics

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| Test replays | 27 | 20+ | COMPLETE |
| FORMAT.md discoveries | Complete header + compression + records + actions | Full format | COMPLETE |
| Test coverage | 237 tests | High | COMPLETE |
| Code quality | No clippy errors | Production ready | COMPLETE |
| Phases complete | 6/6 | 6/6 | COMPLETE |
| Replay validation | 24/27 (88.9%) | 80%+ | COMPLETE |

## Artifact Inventory

### Test Data (`replays/`)
- 15 GRBN format replays (IDs 1-10, 1000-1004)
- 12 Classic format replays (IDs 5000-5002, 10000-10002, 50000-50002, 100000-100002)

### Research (`thoughts/research/`)
- `header-analysis.md` - Rex's complete header analysis report
- `data-acquisition.md` - Scout's replay acquisition report

### Plans (`thoughts/plans/`)
- `header-parsing.md` - Implementation plan for Phases 1-3
- `decompressed-parsing.md` - Implementation plan for Phase 4 (Decompressed Data Parsing)
- `action-parsing.md` - Implementation plan for Phase 5 (Action Parsing)
- `cli-interface.md` - Implementation plan for Phase 6 (CLI Interface)

### Implementation (`thoughts/impl/`)
- `phase-1-foundation.md` - Phase 1 implementation notes
- `phase-2-headers.md` - Phase 2 implementation notes
- `phase-3-decompression.md` - Phase 3 implementation notes
- `phase-4-records.md` - Phase 4 implementation notes
- `phase-5-actions.md` - Phase 5 implementation notes
- `phase-6-cli.md` - Phase 6 implementation notes

### Validation (`thoughts/validation/`)
- `phases-1-3.md` - Validation report for Phases 1-3
- `phase-4.md` - Validation report for Phase 4
- `phase-5.md` - Validation report for Phase 5
- `phase-6.md` - Validation report for Phase 6

### Source Code (`w3g-parser/`)
- `src/lib.rs` - Library root
- `src/error.rs` - Error types
- `src/binary.rs` - Binary utilities
- `src/format.rs` - Format detection
- `src/header/` - Header parsers (mod, grbn, classic)
- `src/decompress/` - Decompression (mod, grbn, classic)
- `src/records/` - Record parsers (game_header, player, timeframe)
- `src/actions/` - Action parsers (types, selection, ability, movement, hotkey, parser)
- `tests/` - Integration tests

## Quick Commands

```bash
# Run all tests
cd w3g-parser && cargo test

# Run clippy
cargo clippy

# Build documentation
cargo doc --open

# Build CLI binary
cargo build --release --bin w3g-parser

# CLI Commands
./target/release/w3g-parser info ../replays/replay_1.w3g
./target/release/w3g-parser validate ../replays/replay_1.w3g
./target/release/w3g-parser parse -o json --players ../replays/replay_1.w3g
./target/release/w3g-parser batch ../replays/ --summary

# Validate all replays
for f in ../replays/*.w3g; do
  ./target/release/w3g-parser validate "$f"
done
```

## Notes

- **Philosophy**: See PHILOSOPHY.md - we discover, we don't copy
- **Major Finding**: GRBN wraps Classic with Protobuf metadata
- **Post-discovery validation**: Can compare to external sources AFTER we discover
