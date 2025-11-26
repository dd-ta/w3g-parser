# w3g-parser

A comprehensive Warcraft III replay (.w3g) parser written in Rust.

## Features

- **Multi-format support**: Classic (RoC/TFT) and Reforged (1.32+) replay formats
- **Complete parsing pipeline**: Headers, decompression, records, and actions
- **Streaming action parsing**: Memory-efficient iteration over game actions
- **Chat message extraction**: Player and system messages with sender identification
- **Game metadata**: Lobby name, map path, host information
- **CLI tools**: Parse, analyze, validate, and batch process replays
- **JSON output**: Structured data export for further processing

## Installation

```bash
cd w3g-parser
cargo build --release
```

The binary will be available at `target/release/w3g-parser`.

## Usage

### CLI

```bash
# Basic replay info (header, players, duration)
w3g-parser info replay.w3g

# Full parse with all options
w3g-parser parse replay.w3g --players --stats --chat

# JSON output for scripting
w3g-parser parse replay.w3g --output json --chat

# Validate replay integrity
w3g-parser validate replay.w3g --verbose

# Batch process a directory
w3g-parser batch ./replays --summary
```

### Library

```rust
use w3g_parser::{Header, decompress, GameRecord, TimeFrameIterator, ChatMessage};

fn parse_replay(data: &[u8]) -> w3g_parser::Result<()> {
    // Parse header (auto-detects format)
    let header = Header::parse(data)?;
    println!("Format: {:?}", header.format());

    // Decompress game data
    let decompressed = decompress(data)?;

    // Parse game record (host, players, settings)
    let game_record = GameRecord::parse(&decompressed)?;
    println!("Host: {}", game_record.header.host_name);
    println!("Game: {}", game_record.header.game_name());

    // Get player roster
    let roster = game_record.player_roster();
    for player in &roster.players {
        println!("Player: {} (slot {})", player.name, player.slot_id);
    }

    // Iterate over timeframes (actions and chat)
    let iter = TimeFrameIterator::new(&decompressed, game_record.actions_offset);
    for timeframe in iter {
        // Process actions
        for action in &timeframe.actions {
            println!("{:?}", action.action_type);
        }

        // Process chat messages
        for chat in &timeframe.chat_messages {
            if chat.is_system_message() {
                println!("[SYSTEM] {}", chat.message);
            } else if let Some(slot) = chat.sender_slot {
                println!("[Slot {}] {}", slot, chat.message);
            }
        }
    }

    Ok(())
}
```

## Supported Formats

| Format | Header | Compression | Status |
|--------|--------|-------------|--------|
| Classic (pre-1.32) | 68 bytes | Block-based zlib | Supported |
| Reforged (1.32+) | 128 bytes (GRBN) | Single zlib stream | Supported |

## Action Types

The parser handles Warcraft III actions with >95% coverage:

- **Selection**: Unit selection, control groups, subgroup selection
- **Movement**: Right-click, attack-move, patrol, hold position
- **Abilities**: Spells, items, builds (with FourCC codes)
- **Hotkeys**: Assign and select control groups
- **Game**: Alliance changes, resource transfers, minimap pings
- **Reforged**: Wrapped abilities, BattleNet sync packets

## Chat Messages

Chat messages are extracted with sender identification:

```rust
pub struct ChatMessage {
    pub sender_slot: Option<u8>,  // Player slot ID (1-24) or None for system
    pub flags: u8,
    pub message_id: u16,
    pub message: String,
}

impl ChatMessage {
    pub fn is_system_message(&self) -> bool { ... }
}
```

## Game Metadata

Extract game information from encoded settings:

```rust
let game_record = GameRecord::parse(&decompressed)?;

// Lobby name (if available)
let name = game_record.header.game_name();

// Raw map path (obfuscated but recognizable)
if let Some(map) = game_record.header.map_path_raw() {
    println!("Map: {}", map);  // e.g., "Maps/W3C/..."
}
```

## Project Structure

```
w3g-parser/
├── src/
│   ├── lib.rs           # Library root and public API
│   ├── header/          # Header parsing (Classic/GRBN)
│   ├── decompress/      # Zlib decompression
│   ├── records/         # Game records (players, timeframes, chat)
│   ├── actions/         # Action parsing (50+ action types)
│   └── bin/             # CLI tool
└── tests/               # Integration tests (27 replays)
```

## Test Coverage

- 159 unit tests
- 53 integration tests
- Tested against replays from warcraft3.info API

## License

MIT
