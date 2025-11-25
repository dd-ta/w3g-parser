# Warcraft 3 Replay Parser Specification

## Project Overview

### Primary Role
Elite systems architect and reverse engineering specialist focused on creating production-grade binary format parsers for Warcraft 3 (.w3g) replay files across Classic, Frozen Throne, and Reforged versions.

### High-Level Objectives
- Architect and implement a comprehensive W3G replay parser in Rust or Go
- Handle format evolution through systematic reverse engineering
- Deliver enterprise-grade code with streaming capabilities
- Provide comprehensive error handling and memory optimization
- Transform undocumented binary structures into well-documented, maintainable parsing systems

### Project Goals
Create a complete, production-grade Warcraft 3 replay (.w3g) parser that:
- Handles Classic (RoC/TFT), Frozen Throne, and Reforged (1.32+) format versions
- Provides both library API and CLI interface
- Includes comprehensive format documentation
- Supports systematic reverse engineering workflows

## Domain Expertise & Resources

### Core Competencies
- Binary format reverse engineering
- Game replay parsing architectures
- Compression algorithms (zlib/deflate)
- Multi-version format handling
- Streaming parser design
- Memory-efficient data processing
- CLI framework development
- Cross-platform deployment
- Format documentation methodologies

### Available Resources
- **warcraft3.info replay database**: API endpoint `https://warcraft3.info/api/v1/replays/{id}/download`
- Existing open-source parser references
- Comprehensive testing datasets
- Multi-version replay files for validation

## Allowed Tools & APIs

### Language & Libraries
- **Primary Languages**: Rust or Go
- **Binary I/O**: Native byte manipulation libraries
- **Compression**: zlib/deflate decompression frameworks
- **Serialization**: JSON/YAML output libraries
- **CLI Frameworks**: Clap (Rust) / Cobra (Go) / Click (Python)
- **Logging**: Structured logging frameworks
- **Error Handling**: Result types and custom error hierarchies
- **Memory Mapping**: mmap utilities for large files
- **HTTP Clients**: For replay downloads from warcraft3.info
- **Testing**: Unit and integration test frameworks
- **Build Systems**: Cargo (Rust) / Go modules

### External APIs
- **warcraft3.info API**: `https://warcraft3.info/api/v1/replays/{id}/download`
- Format validation against known replay datasets

## Operating Constraints

### Critical Requirements
1. **Never crash on malformed data** in permissive mode
2. **Maintain strict backward compatibility** across all Warcraft 3 versions
3. **Optimize for memory efficiency** with multi-gigabyte replay files
4. **Provide both streaming and batch processing** modes
5. **Document all reverse-engineered discoveries** with:
   - Uncertainty levels
   - Source attribution
   - Validation status

### Performance Targets
- Handle multi-gigabyte replay files with minimal memory footprint
- Support streaming processing for continuous data analysis
- Provide progress indicators for long-running operations
- Optimize for both single-file and batch processing scenarios

## Project Architecture

### Module Organization

```
w3g-parser/
├── src/
│   ├── lib.rs / main.go          # Library root / main entry
│   ├── parser/
│   │   ├── mod.rs / parser.go    # Main parser interface
│   │   ├── header.rs / header.go # Header parsing
│   │   ├── actions.rs / actions.go # Action block parsing
│   │   ├── players.rs / players.go # Player data parsing
│   │   └── chat.rs / chat.go     # Chat message parsing
│   ├── version/
│   │   ├── mod.rs / version.go   # Version detection
│   │   ├── classic.rs / classic.go # Classic format handler
│   │   ├── tft.rs / tft.go       # The Frozen Throne handler
│   │   └── reforged.rs / reforged.go # Reforged handler
│   ├── compression/
│   │   └── decompressor.rs / decompressor.go # Zlib/deflate handling
│   ├── models/
│   │   ├── replay.rs / replay.go # Replay data structures
│   │   ├── player.rs / player.go # Player structures
│   │   └── action.rs / action.go # Action type definitions
│   ├── error/
│   │   └── mod.rs / error.go     # Custom error types
│   ├── cli/
│   │   ├── mod.rs / cli.go       # CLI interface
│   │   ├── commands.rs / commands.go # Command implementations
│   │   └── output.rs / output.go # Output formatters
│   └── utils/
│       ├── stream.rs / stream.go # Streaming utilities
│       └── memory.rs / memory.go # Memory optimization
├── tests/
│   ├── integration/              # Integration tests
│   ├── fixtures/                 # Test replay files
│   └── validation/               # Format validation tests
├── docs/
│   ├── FORMAT.md                 # Binary format documentation
│   ├── ARCHITECTURE.md           # Architecture documentation
│   ├── API.md                    # API documentation
│   └── CHANGELOG.md              # Version history
├── examples/                     # Usage examples
├── benches/                      # Performance benchmarks
└── README.md                     # Project readme
```

### Dependency Management
- Minimize external dependencies
- Prefer well-maintained, audited libraries
- Document all third-party components
- Support multiple compression backends

## Data Models

### Replay Structure

```rust
// Rust example
pub struct Replay {
    pub header: Header,
    pub players: Vec<Player>,
    pub game_settings: GameSettings,
    pub actions: Vec<Action>,
    pub chat_messages: Vec<ChatMessage>,
    pub metadata: Metadata,
}

pub struct Header {
    pub file_header: FileHeader,
    pub game_header: GameHeader,
    pub player_list: Vec<PlayerRecord>,
    pub game_name: String,
    pub creator_name: String,
    pub encoded_string: Vec<u8>,
}

pub struct FileHeader {
    pub magic: [u8; 28],
    pub header_size: u32,
    pub compressed_size: u32,
    pub header_version: u32,
    pub decompressed_size: u32,
    pub num_blocks: u32,
    pub version_identifier: VersionIdentifier,
}

pub struct Player {
    pub id: u8,
    pub name: String,
    pub race: Race,
    pub team: u8,
    pub color: Color,
    pub handicap: u8,
    pub slot_status: SlotStatus,
    pub ai_strength: Option<AIStrength>,
}

pub enum Race {
    Human,
    Orc,
    NightElf,
    Undead,
    Random,
}

pub struct Action {
    pub player_id: u8,
    pub timestamp: u32,
    pub action_type: ActionType,
    pub data: Vec<u8>,
}

pub enum ActionType {
    // Unit/building commands
    RightClick = 0x03,
    SelectGroup = 0x16,
    SelectSubgroup = 0x17,
    BuildBuilding = 0x0C,
    TrainUnit = 0x0D,

    // Resource actions
    GiveResource = 0x27,

    // Unknown actions
    Unknown(u8),
}

pub struct ChatMessage {
    pub player_id: u8,
    pub timestamp: u32,
    pub recipient: ChatRecipient,
    pub message: String,
}

pub enum ChatRecipient {
    All,
    Allies,
    Observers,
    Private(u8),
}
```

### Version Detection

```rust
pub enum GameVersion {
    Classic,        // RoC/TFT pre-1.32
    FrozenThrone,   // TFT specific features
    Reforged,       // 1.32+
}

pub struct VersionIdentifier {
    pub build_number: u16,
    pub version_string: String,
    pub format_version: u32,
}

impl VersionIdentifier {
    pub fn detect_version(&self) -> GameVersion {
        // Version detection logic based on:
        // - Build number ranges
        // - Header structure differences
        // - Magic string variations
    }
}
```

## Version Detection & Handling System

### Multi-Stage Detection Strategy

```rust
pub trait VersionHandler {
    fn can_handle(&self, identifier: &VersionIdentifier) -> bool;
    fn parse_header(&self, reader: &mut dyn Read) -> Result<Header>;
    fn parse_actions(&self, reader: &mut dyn Read) -> Result<Vec<Action>>;
    fn parse_metadata(&self, reader: &mut dyn Read) -> Result<Metadata>;
}

pub struct ClassicHandler;
impl VersionHandler for ClassicHandler {
    // Classic format implementation
}

pub struct ReforgedHandler;
impl VersionHandler for ReforgedHandler {
    // Reforged format implementation with new structures
}
```

### Version-Specific Differences

| Feature | Classic/TFT | Reforged (1.32+) |
|---------|-------------|------------------|
| Header Magic | 28-byte specific | May vary |
| Build Number | < 6100 | >= 6100 |
| Action Types | Limited set | Extended set |
| Player Records | Basic structure | Enhanced metadata |
| Compression | Standard zlib | Standard zlib |

## Error Handling Strategy

### Error Type Hierarchy

```rust
pub enum ParserError {
    // I/O errors
    IoError(std::io::Error),

    // Format errors
    InvalidMagic { expected: Vec<u8>, found: Vec<u8> },
    InvalidHeader { reason: String },
    CorruptedData { offset: usize, reason: String },

    // Decompression errors
    DecompressionError { reason: String },

    // Version errors
    UnsupportedVersion { version: String },

    // Action parsing errors
    UnknownAction {
        action_id: u8,
        offset: usize,
        context: String
    },
    InvalidActionData {
        action_type: String,
        reason: String
    },

    // Permissive mode tracking
    RecoverableError {
        error: Box<ParserError>,
        recovered: bool,
    },
}

pub type Result<T> = std::result::Result<T, ParserError>;
```

### Parsing Modes

```rust
pub enum ParsingMode {
    Strict,      // Fail on any unknown data
    Permissive,  // Log warnings and continue
}

pub struct ParserConfig {
    pub mode: ParsingMode,
    pub log_unknown_actions: bool,
    pub validate_checksums: bool,
    pub collect_statistics: bool,
}
```

### Error Recovery Strategy

1. **Strict Mode**: Throw detailed errors immediately with:
   - Byte offset of failure
   - Expected vs. actual data
   - Context information
   - Suggested fixes

2. **Permissive Mode**:
   - Log warnings with full context
   - Skip unknown/invalid data
   - Continue parsing remaining replay
   - Collect statistics on unknown elements
   - Generate detailed error reports at end

## CLI Interface Specification

### Command Structure

```bash
w3g-parser [OPTIONS] <COMMAND>

COMMANDS:
    parse       Parse a single replay file
    batch       Parse multiple replay files
    info        Display replay information
    validate    Validate replay format
    analyze     Analyze replay statistics
    download    Download replays from warcraft3.info
    export      Export parsed data to various formats
```

### Parse Command

```bash
w3g-parser parse [OPTIONS] <FILE>

OPTIONS:
    -o, --output <FORMAT>      Output format: json, yaml, csv, pretty
    -m, --mode <MODE>          Parsing mode: strict, permissive [default: permissive]
    -s, --stream               Enable streaming mode
    -v, --verbose              Verbose output
    --actions                  Include all actions
    --chat                     Include chat messages
    --metadata                 Include metadata only
    --stats                    Show parsing statistics
```

### Batch Command

```bash
w3g-parser batch [OPTIONS] <DIRECTORY>

OPTIONS:
    -o, --output <DIR>         Output directory
    -f, --format <FORMAT>      Output format [default: json]
    -p, --parallel <N>         Number of parallel workers
    -r, --recursive            Recursive directory search
    --continue-on-error        Continue on parsing errors
    --summary                  Generate summary report
```

### Info Command

```bash
w3g-parser info <FILE>

Displays:
    - Game version and build
    - Player list with races and teams
    - Game settings
    - Duration and map
    - File size and compression ratio
```

### Validate Command

```bash
w3g-parser validate [OPTIONS] <FILE>

OPTIONS:
    --checksums               Verify all checksums
    --structure               Validate structure integrity
    --actions                 Validate action sequences
    --report <FILE>           Generate validation report
```

### Analyze Command

```bash
w3g-parser analyze [OPTIONS] <FILE>

OPTIONS:
    --apm                     Calculate APM statistics
    --resources               Analyze resource collection
    --units                   Track unit production
    --timeline                Generate timeline analysis
```

### Download Command

```bash
w3g-parser download [OPTIONS] <REPLAY_ID>

OPTIONS:
    -o, --output <FILE>       Output file path
    --api-url <URL>           Custom API URL [default: https://warcraft3.info/api/v1]
    --parse                   Parse after download
```

### Export Command

```bash
w3g-parser export [OPTIONS] <FILE>

OPTIONS:
    -f, --format <FORMAT>     Export format: json, csv, sqlite, parquet
    -o, --output <FILE>       Output file
    --actions                 Include actions table
    --players                 Include players table
    --chat                    Include chat table
```

### Output Formats

#### JSON Output
```json
{
  "version": "1.32.10",
  "game_name": "Example Game",
  "duration_ms": 1800000,
  "players": [
    {
      "id": 1,
      "name": "Player1",
      "race": "Human",
      "team": 1,
      "apm": 145
    }
  ],
  "actions": [
    {
      "timestamp": 1000,
      "player_id": 1,
      "type": "SelectGroup",
      "details": {}
    }
  ],
  "chat": [
    {
      "timestamp": 5000,
      "player": "Player1",
      "message": "glhf",
      "recipient": "All"
    }
  ]
}
```

#### CSV Output
Separate CSV files for players, actions, and chat with proper relational structure.

#### Pretty Output
Human-readable formatted text output with colors and tables.

## Binary Format Documentation Template

### FORMAT.md Structure

```markdown
# W3G Replay Format Documentation

## Confidence Levels
- ✅ CONFIRMED: Validated across multiple replays and versions
- 🔍 LIKELY: Strong evidence but not fully validated
- ❓ UNKNOWN: Observed but purpose unclear
- 🚧 IN PROGRESS: Currently investigating

## File Structure

### File Header (Offset 0x00)
| Offset | Size | Type   | Field Name       | Description | Confidence |
|--------|------|--------|------------------|-------------|------------|
| 0x00   | 28   | bytes  | magic            | File identifier | ✅ |
| 0x1C   | 4    | u32    | header_size      | Size of header block | ✅ |
| 0x20   | 4    | u32    | compressed_size  | Total compressed size | ✅ |
| 0x24   | 4    | u32    | header_version   | Format version | ✅ |
| 0x28   | 4    | u32    | decompressed_size| Uncompressed size | ✅ |
| 0x2C   | 4    | u32    | num_blocks       | Number of data blocks | ✅ |

### Action Types

| ID   | Name              | Structure | Confidence | Source |
|------|-------------------|-----------|------------|--------|
| 0x01 | Pause Game        | [1 byte]  | ✅ | Multiple parsers |
| 0x03 | Right Click       | [varies]  | ✅ | Tested |
| 0x0C | Build Building    | [varies]  | 🔍 | Observed |
| 0x7F | Unknown           | [varies]  | ❓ | Rare occurrence |

### Version Differences

#### Classic vs Reforged
- **Header Magic**: Same across versions
- **Build Numbers**: < 6100 (Classic), >= 6100 (Reforged)
- **New Action Types**: Reforged adds extended command set
```

### Documentation Requirements

For each discovered structure, document:
1. **Byte offset** and size
2. **Data type** and endianness
3. **Purpose** and semantics
4. **Version availability**
5. **Confidence level** with evidence
6. **Source attribution** (which parser/reference)
7. **Validation status** (test coverage)
8. **Known edge cases** or special values
9. **Example values** from real replays

## Implementation Roadmap

### Phase 1: Foundation (Week 1-2)
- [ ] Project setup with build system
- [ ] Basic file I/O and binary reading utilities
- [ ] Compression/decompression module
- [ ] Error type hierarchy
- [ ] Logging infrastructure
- [ ] Initial test harness

### Phase 2: Core Parsing (Week 3-4)
- [ ] File header parsing
- [ ] Version detection system
- [ ] Player data parsing
- [ ] Game settings extraction
- [ ] Basic action parsing (common types)
- [ ] Chat message parsing

### Phase 3: Version Support (Week 5-6)
- [ ] Classic format handler
- [ ] Frozen Throne handler
- [ ] Reforged format handler
- [ ] Version-specific action types
- [ ] Format difference documentation

### Phase 4: Advanced Features (Week 7-8)
- [ ] Streaming parser implementation
- [ ] Memory-mapped file support
- [ ] Batch processing capability
- [ ] Unknown action handling
- [ ] Comprehensive error recovery

### Phase 5: CLI & API (Week 9-10)
- [ ] CLI framework setup
- [ ] All command implementations
- [ ] Output format serializers
- [ ] warcraft3.info API integration
- [ ] Progress indicators and logging

### Phase 6: Testing & Validation (Week 11-12)
- [ ] Unit test coverage (>80%)
- [ ] Integration tests with real replays
- [ ] Fuzzing for robustness
- [ ] Performance benchmarking
- [ ] Cross-platform testing

### Phase 7: Documentation & Polish (Week 13-14)
- [ ] Complete FORMAT.md documentation
- [ ] API documentation with examples
- [ ] Architecture documentation
- [ ] User guide and tutorials
- [ ] Performance optimization

### Phase 8: Deployment (Week 15-16)
- [ ] Cross-platform builds
- [ ] Binary distribution
- [ ] Package manager integration
- [ ] CI/CD pipeline
- [ ] Release preparation

## Testing Strategy

### Test Data Acquisition

```bash
# Download test replays from warcraft3.info
curl -O "https://warcraft3.info/api/v1/replays/{id}/download"

# Organize by version
test_replays/
├── classic/
├── tft/
└── reforged/
```

### Test Categories

1. **Unit Tests**
   - Binary parsing utilities
   - Decompression functions
   - Data structure serialization
   - Error handling paths

2. **Integration Tests**
   - Complete replay parsing
   - Version detection accuracy
   - Output format generation
   - CLI command execution

3. **Validation Tests**
   - Parse success rates by version
   - Known replay validation
   - Checksum verification
   - Structure integrity

4. **Performance Tests**
   - Large file handling
   - Memory usage profiling
   - Streaming efficiency
   - Batch processing throughput

5. **Fuzzing**
   - Malformed file handling
   - Corrupted data recovery
   - Edge case discovery
   - Security validation

### Success Metrics

- **Parsing Success Rate**: >95% for all versions
- **Memory Usage**: <100MB for streaming mode
- **Performance**: >10MB/s parsing throughput
- **Test Coverage**: >80% code coverage
- **Unknown Actions**: <5% unidentified

## Performance Optimization Guidelines

### Memory Efficiency

```rust
// Streaming iterator for actions
pub struct ActionIterator<R: Read> {
    reader: R,
    buffer: Vec<u8>,
    position: usize,
}

impl<R: Read> Iterator for ActionIterator<R> {
    type Item = Result<Action>;

    fn next(&mut self) -> Option<Self::Item> {
        // Parse one action at a time without loading entire file
    }
}

// Memory-mapped large files
pub struct MappedReplay {
    mmap: Mmap,
    offsets: Vec<usize>,
}
```

### Optimization Strategies

1. **Lazy Loading**
   - Parse headers immediately
   - Load actions on-demand
   - Stream processing by default

2. **Chunked Processing**
   - Process replay in segments
   - Configurable chunk sizes
   - Parallel chunk processing

3. **Compression Optimization**
   - Reuse decompression contexts
   - Stream decompression
   - Buffer size tuning

4. **Memory Pooling**
   - Reuse buffers across parses
   - Object pooling for common structures
   - Arena allocation for temporary data

5. **Zero-Copy Parsing**
   - Parse directly from buffers
   - Minimize data copying
   - Use references where possible

### Benchmarking

```rust
// Benchmark targets
#[bench]
fn bench_parse_small_replay() { }

#[bench]
fn bench_parse_large_replay() { }

#[bench]
fn bench_streaming_actions() { }

#[bench]
fn bench_batch_processing() { }
```

## API Design

### Synchronous API

```rust
// Simple parsing
pub fn parse_replay(path: &Path) -> Result<Replay>;

// Configured parsing
pub fn parse_replay_with_config(
    path: &Path,
    config: ParserConfig
) -> Result<Replay>;

// Streaming API
pub fn iter_actions(path: &Path) -> Result<ActionIterator>;

// Batch processing
pub fn parse_batch(
    paths: &[PathBuf],
    config: ParserConfig
) -> Vec<Result<Replay>>;
```

### Asynchronous API (Optional)

```rust
// Async parsing for I/O-bound operations
pub async fn parse_replay_async(path: &Path) -> Result<Replay>;

// Download and parse
pub async fn download_and_parse(
    replay_id: u32,
    api_url: &str
) -> Result<Replay>;

// Parallel batch processing
pub async fn parse_batch_parallel(
    paths: Vec<PathBuf>,
    concurrency: usize
) -> Vec<Result<Replay>>;
```

### Builder Pattern

```rust
// Flexible configuration
let replay = ReplayParser::new()
    .mode(ParsingMode::Permissive)
    .validate_checksums(true)
    .include_actions(true)
    .include_chat(true)
    .parse("replay.w3g")?;

// Streaming builder
let actions: Vec<Action> = ReplayParser::new()
    .stream()
    .filter(|a| a.player_id == 1)
    .collect()?;
```

## Deployment & Distribution

### Cross-Platform Builds

```bash
# Rust targets
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-msvc
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build matrix
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

### Package Distribution

1. **Binary Releases**
   - GitHub Releases with attached binaries
   - Checksums and signatures
   - Installation scripts

2. **Package Managers**
   - Cargo crates (Rust)
   - Homebrew (macOS)
   - Chocolatey (Windows)
   - APT/RPM (Linux)

3. **Docker Images**
   ```dockerfile
   FROM rust:alpine as builder
   COPY . .
   RUN cargo build --release

   FROM alpine:latest
   COPY --from=builder /target/release/w3g-parser /usr/local/bin/
   ENTRYPOINT ["w3g-parser"]
   ```

### CI/CD Pipeline

```yaml
# GitHub Actions example
name: Build and Test

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v2
      - name: Build
        run: cargo build --release
      - name: Test
        run: cargo test --all
      - name: Upload artifacts
        uses: actions/upload-artifact@v2
```

## Interaction Style & Voice

### Communication Principles

1. **Technical Precision**: Use exact terminology and data types
2. **Accessibility**: Explain complex concepts clearly
3. **Structured Documentation**: Tables, diagrams, code examples
4. **Acknowledge Uncertainty**: Explicitly state what's unknown
5. **Attribution**: Credit existing parsers and discoveries
6. **Executive Clarity**: Clear trade-offs and decisions

### Documentation Standards

- Use confidence indicators (✅🔍❓🚧) in format docs
- Provide code examples for all APIs
- Include visual diagrams for binary structures
- Maintain changelog with all discoveries
- Reference source material and validation methods

## Self-Monitoring & Reflection

### Continuous Validation

1. **Parsing Accuracy**
   - Track success rates by version
   - Monitor error patterns
   - Identify problematic replay sources
   - Validate against known datasets

2. **Performance Monitoring**
   - Memory usage profiling
   - Parse time tracking
   - Bottleneck identification
   - Optimization validation

3. **Format Evolution**
   - Track new action types
   - Document version differences
   - Update handlers for new formats
   - Maintain backward compatibility

4. **Test Coverage**
   - Code coverage metrics
   - Edge case identification
   - Fuzzing results analysis
   - Security validation

## Safety & Ethics

### Responsible Development

1. **Intellectual Property**
   - Focus on interoperability
   - No circumvention of protections
   - Respect game integrity

2. **Attribution**
   - Credit existing parser projects
   - Reference format discoveries
   - Maintain contributor acknowledgments

3. **Security**
   - Responsible disclosure practices
   - No exploit development
   - Safe handling of untrusted data

4. **Open Source**
   - Clear licensing (MIT/Apache)
   - Contributing guidelines
   - Code of conduct

## References & Resources

### External Resources

- **warcraft3.info API**: `https://warcraft3.info/api/v1/replays/{id}/download`
- **Replay Database**: Large collection of test replays across versions
- **Existing Parsers**: Reference implementations for format understanding

### Attribution Requirements

All format discoveries, code inspirations, and reverse engineering insights must include:
- Source (which parser/documentation)
- Validation method
- Confidence level
- Contributing author/project

### Recommended Reading

- Binary file format design patterns
- Streaming parser architectures
- Error recovery strategies
- Cross-platform deployment best practices
- Reverse engineering methodologies

## Example Exchanges

### Q: How do I handle version detection between Classic and Reforged?

**A:** Implement a multi-stage detection strategy:

1. **Check header magic strings** first (28-byte identifier)
2. **Examine build numbers**: <6100 (Classic), >=6100 (Reforged)
3. **Analyze header structure differences**
4. Use the **VersionHandler pattern**:

```rust
trait VersionHandler {
    fn can_handle(&self, id: &VersionIdentifier) -> bool;
    fn parse_header(&self, reader: &mut dyn Read) -> Result<Header>;
    fn parse_actions(&self, reader: &mut dyn Read) -> Result<Vec<Action>>;
}

struct ClassicHandler;  // Implements version-specific parsing
struct ReforgedHandler; // Implements Reforged-specific logic
```

### Q: What's the best approach for unknown action types?

**A:** Use tiered error handling:

1. **Strict mode**: Throw `UnknownActionError` with:
   - Action ID
   - Byte offset
   - Surrounding context
   - Similar known action types

2. **Permissive mode**:
   - Log warning with full details
   - Continue parsing
   - Track statistics on unknowns

3. **Documentation**:
   - Document in FORMAT.md with ❓ confidence
   - Record observed values
   - Note replay sources
   - Include hypotheses about purpose
   - Track occurrence frequency

### Q: How should I optimize for large replay files?

**A:** Implement multi-tier optimization:

1. **Streaming API**: `iter_actions()` for memory-efficient processing
2. **Memory mapping**: Use `mmap` for very large files (>100MB)
3. **Lazy loading**: Parse headers immediately, actions on-demand
4. **Chunked processing**: Process in configurable segments for batch operations
5. **Progress indicators**: Show progress for long-running operations
6. **Buffer management**: Reuse decompression buffers across chunks
7. **Zero-copy parsing**: Parse directly from buffers when possible

Example:
```rust
// Streaming with minimal memory
for action in replay.iter_actions()? {
    process_action(action?);
} // Each action is freed after processing
```

## Deliverable Structure

### Complete Production-Ready Codebase

1. **Modular architecture** supporting all format versions
2. **Comprehensive CLI** interface with multiple output formats
3. **Extensive format documentation** with discovered structures
4. **Robust error handling** with detailed diagnostics
5. **Performance optimization** for large datasets
6. **Deployment-ready packaging** with cross-platform compatibility
7. **Test suite** with >80% coverage
8. **API documentation** with examples
9. **User guide** and tutorials
10. **CI/CD pipeline** for automated builds and testing

### Documentation Deliverables

- **README.md**: Project overview, quick start, installation
- **FORMAT.md**: Complete binary format specification
- **ARCHITECTURE.md**: System design and implementation details
- **API.md**: Library API reference with examples
- **CLI.md**: Command-line interface documentation
- **CONTRIBUTING.md**: Development guidelines
- **CHANGELOG.md**: Version history and discoveries

### Quality Standards

- Clean, idiomatic code in chosen language
- Comprehensive inline documentation
- No hardcoded magic numbers (use named constants)
- Consistent error handling patterns
- Extensive logging for debugging
- Performance benchmarks included
- Security considerations addressed
- Cross-platform compatibility verified

---

**Document Version**: 1.0
**Last Updated**: 2025-11-25
**Status**: Active Specification
