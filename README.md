# w3g-parser

A comprehensive Warcraft III replay (.w3g) parser written in Rust.

## Features

- **Multi-format support**: Classic (RoC/TFT) and Reforged (1.32+) replay formats
- **Complete parsing pipeline**: Headers, decompression, records, and actions
- **Streaming action parsing**: Memory-efficient iteration over game actions
- **CLI tools**: Parse, analyze, and dump replay data
- **JSON output**: Structured data export for further processing

## Installation

```bash
cd w3g-parser
cargo build --release
```

## Usage

### CLI

```bash
# Parse a replay and output JSON
w3g-parser parse replay.w3g

# Show detailed info
w3g-parser info replay.w3g

# Analyze player actions and APM
w3g-parser analyze replay.w3g
```

### Library

```rust
use w3g_parser::{Header, decompress, Result};

fn parse_replay(data: &[u8]) -> Result<()> {
    // Parse header (auto-detects format)
    let header = Header::parse(data)?;

    println!("Format: {:?}", header.format());
    println!("Decompressed size: {} bytes", header.decompressed_size());

    // Decompress game data
    let decompressed = decompress(data)?;

    // Parse records and actions...
    Ok(())
}
```

## Supported Formats

| Format | Header | Compression | Status |
|--------|--------|-------------|--------|
| Classic (pre-1.32) | 68 bytes | Block-based zlib | Supported |
| Reforged (1.32+) | 128 bytes (GRBN) | Single zlib stream | Supported |

## Action Types

The parser handles common Warcraft III actions:
- Unit selection and control groups
- Movement and right-click commands
- Ability usage (items, spells, builds)
- Hotkey assignments

## Project Structure

```
w3g-parser/
├── src/
│   ├── lib.rs           # Library root
│   ├── header/          # Header parsing (Classic/GRBN)
│   ├── decompress/      # Zlib decompression
│   ├── records/         # Game records (players, timeframes)
│   ├── actions/         # Action parsing
│   └── bin/             # CLI tools
└── tests/               # Integration tests
```

## License

MIT
