# Plan: CLI Interface (Phase 6)

## Metadata
- **Agent**: Archie
- **Date**: 2025-11-25
- **Research Used**:
  - `spec.md` (CLI specification)
  - `thoughts/validation/phase-5.md` (current capabilities)
  - Current library API (`src/lib.rs`)
- **Status**: Approved

## Overview

This plan implements the CLI interface for the W3G parser, providing user-friendly commands to parse, validate, and analyze Warcraft 3 replay files. The CLI builds on top of the completed library functionality from Phases 1-5.

The CLI will provide four main commands:
1. **info** - Quick replay information display
2. **parse** - Detailed parsing with output format options
3. **validate** - Validation with exit codes for scripting
4. **batch** - Directory-level batch processing

## Prerequisites

- [x] Phase 1: Project Foundation (error types, binary utilities)
- [x] Phase 2: Header Parsing (GRBN + Classic)
- [x] Phase 3: Decompression (block-based + single stream)
- [x] Phase 4: Record Parsing (GameRecord, Players, TimeFrames)
- [x] Phase 5: Action Parsing (Selection, Ability, Movement, Hotkey)

## Current Library API

The library now exposes:

```rust
// Re-exports at crate root
pub use actions::{
    AbilityAction, AbilityCode, Action, ActionContext, ActionIterator, ActionStatistics,
    ActionType, HotkeyAction, HotkeyOperation, MovementAction, Position, SelectionAction,
    SelectionMode,
};
pub use decompress::decompress;
pub use error::{ParserError, Result};
pub use format::{detect_format, ClassicVersion, ReplayFormat};
pub use header::Header;
pub use records::{GameRecord, GameRecordHeader, PlayerRoster, TimeFrame, TimeFrameIterator};
```

Key APIs:
- `Header::parse(&[u8])` - Parse file header
- `decompress(&[u8], &Header)` - Decompress replay data
- `GameRecord::parse(&[u8])` - Parse game records
- `TimeFrame::actions()` - Iterate actions within frames
- `ActionStatistics` - Collect action statistics

---

## Phase 6A: CLI Framework

### Goal
Set up the CLI framework with clap and establish the command structure.

### Files to Create/Modify
- `Cargo.toml` - Add clap dependency
- `src/bin/w3g-parser.rs` - Main CLI binary

### Implementation Steps

1. **Add dependencies to Cargo.toml**
   ```toml
   [dependencies]
   clap = { version = "4.4", features = ["derive"] }
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"

   [[bin]]
   name = "w3g-parser"
   path = "src/bin/w3g-parser.rs"
   ```

2. **Create CLI argument structure**
   ```rust
   use clap::{Parser, Subcommand};

   #[derive(Parser)]
   #[command(name = "w3g-parser")]
   #[command(about = "Warcraft 3 replay (.w3g) parser", long_about = None)]
   #[command(version)]
   struct Cli {
       #[command(subcommand)]
       command: Commands,
   }

   #[derive(Subcommand)]
   enum Commands {
       /// Display replay information
       Info {
           /// Path to the replay file
           file: PathBuf,
       },
       /// Parse a replay file
       Parse {
           /// Path to the replay file
           file: PathBuf,
           /// Output format: json, pretty [default: pretty]
           #[arg(short, long, default_value = "pretty")]
           output: OutputFormat,
           /// Include all actions in output
           #[arg(long)]
           actions: bool,
           /// Include player details
           #[arg(long)]
           players: bool,
           /// Show parsing statistics
           #[arg(long)]
           stats: bool,
       },
       /// Validate replay format
       Validate {
           /// Path to the replay file
           file: PathBuf,
           /// Verbose error reporting
           #[arg(short, long)]
           verbose: bool,
       },
       /// Parse multiple replay files
       Batch {
           /// Directory containing replay files
           directory: PathBuf,
           /// Output directory
           #[arg(short, long)]
           output: Option<PathBuf>,
           /// Output format [default: json]
           #[arg(short, long, default_value = "json")]
           format: OutputFormat,
           /// Generate summary report
           #[arg(long)]
           summary: bool,
           /// Continue on errors
           #[arg(long)]
           continue_on_error: bool,
       },
   }
   ```

3. **Define output format enum**
   ```rust
   #[derive(Clone, Debug, ValueEnum)]
   enum OutputFormat {
       Json,
       Pretty,
   }
   ```

4. **Implement main() with command dispatch**
   ```rust
   fn main() -> ExitCode {
       let cli = Cli::parse();

       match cli.command {
           Commands::Info { file } => cmd_info(&file),
           Commands::Parse { file, output, actions, players, stats } =>
               cmd_parse(&file, output, actions, players, stats),
           Commands::Validate { file, verbose } => cmd_validate(&file, verbose),
           Commands::Batch { directory, output, format, summary, continue_on_error } =>
               cmd_batch(&directory, output, format, summary, continue_on_error),
       }
   }
   ```

### Validation Criteria
- [ ] `cargo build --bin w3g-parser` succeeds
- [ ] `w3g-parser --help` shows all commands
- [ ] `w3g-parser info --help` shows info command options
- [ ] `w3g-parser --version` shows version

### Estimated Complexity
**Low** - Standard clap setup with derive macros.

---

## Phase 6B: Info Command

### Goal
Implement the `info` subcommand to display quick replay metadata.

### Files to Modify
- `src/bin/w3g-parser.rs` - Add info command implementation

### Implementation Steps

1. **Create info command handler**
   ```rust
   fn cmd_info(file: &Path) -> ExitCode {
       // Read file
       let data = match std::fs::read(file) {
           Ok(d) => d,
           Err(e) => {
               eprintln!("Error reading file: {}", e);
               return ExitCode::FAILURE;
           }
       };

       // Parse header
       let header = match Header::parse(&data) {
           Ok(h) => h,
           Err(e) => {
               eprintln!("Error parsing header: {}", e);
               return ExitCode::FAILURE;
           }
       };

       // Decompress and parse game record
       let decompressed = match decompress(&data, &header) {
           Ok(d) => d,
           Err(e) => {
               eprintln!("Error decompressing: {}", e);
               return ExitCode::FAILURE;
           }
       };

       let game_record = match GameRecord::parse(&decompressed) {
           Ok(r) => r,
           Err(e) => {
               eprintln!("Error parsing game record: {}", e);
               return ExitCode::FAILURE;
           }
       };

       // Display info
       print_info(&header, &game_record, &data.len());

       ExitCode::SUCCESS
   }
   ```

2. **Implement pretty info display**
   ```rust
   fn print_info(header: &Header, game_record: &GameRecord, file_size: &usize) {
       println!("=== Replay Information ===\n");

       // File information
       println!("File:");
       println!("  Size: {} bytes ({:.2} KB)", file_size, *file_size as f64 / 1024.0);
       println!("  Format: {:?}", header.format());

       // Version information
       match header {
           Header::Classic(h) => {
               println!("  Build Version: {}", h.build_version);
               println!("  Block Format: {:?}", h.version_type());
               println!("  Duration: {}", h.duration_string());
           }
           Header::Grbn(h) => {
               println!("  GRBN Version: {}", h.version);
           }
       }

       println!();

       // Players
       println!("Players:");
       println!("  Host: {}", game_record.host_name());
       for name in game_record.player_names() {
           println!("  - {}", name);
       }

       println!();

       // Technical details
       println!("Technical:");
       println!("  Header Size: {} bytes", header.header_size());
       println!("  Data Offset: 0x{:X}", header.data_offset());
       println!("  Decompressed Size: {} bytes", header.decompressed_size());
   }
   ```

### Output Example
```
=== Replay Information ===

File:
  Size: 51200 bytes (50.00 KB)
  Format: Classic
  Build Version: 10036
  Block Format: TypeB
  Duration: 00:15:32

Players:
  Host: Player1
  - Player1
  - Player2

Technical:
  Header Size: 68 bytes
  Data Offset: 0x44
  Decompressed Size: 245000 bytes
```

### Validation Criteria
- [ ] `w3g-parser info replay.w3g` displays formatted output
- [ ] Works on both GRBN and Classic format files
- [ ] Shows player list correctly
- [ ] Shows duration for Classic files
- [ ] Exit code 0 on success, 1 on failure

### Estimated Complexity
**Low** - Straightforward data extraction and formatting.

---

## Phase 6C: Parse Command

### Goal
Implement the `parse` subcommand with JSON and pretty output formats.

### Files to Modify
- `src/bin/w3g-parser.rs` - Add parse command implementation

### Implementation Steps

1. **Define serializable output structures**
   ```rust
   use serde::Serialize;

   #[derive(Serialize)]
   struct ParseOutput {
       #[serde(skip_serializing_if = "Option::is_none")]
       header: Option<HeaderInfo>,
       #[serde(skip_serializing_if = "Option::is_none")]
       players: Option<Vec<PlayerInfo>>,
       #[serde(skip_serializing_if = "Option::is_none")]
       actions: Option<Vec<ActionInfo>>,
       #[serde(skip_serializing_if = "Option::is_none")]
       statistics: Option<Statistics>,
   }

   #[derive(Serialize)]
   struct HeaderInfo {
       format: String,
       file_size: usize,
       decompressed_size: u32,
       #[serde(skip_serializing_if = "Option::is_none")]
       build_version: Option<u32>,
       #[serde(skip_serializing_if = "Option::is_none")]
       duration_ms: Option<u32>,
   }

   #[derive(Serialize)]
   struct PlayerInfo {
       slot_id: u8,
       name: String,
   }

   #[derive(Serialize)]
   struct ActionInfo {
       player_id: u8,
       timestamp_ms: u32,
       action_type: String,
       #[serde(skip_serializing_if = "Option::is_none")]
       details: Option<serde_json::Value>,
   }

   #[derive(Serialize)]
   struct Statistics {
       total_frames: usize,
       total_actions: usize,
       actions_by_type: std::collections::HashMap<String, usize>,
       actions_by_player: std::collections::HashMap<u8, usize>,
   }
   ```

2. **Implement parse command handler**
   ```rust
   fn cmd_parse(
       file: &Path,
       output: OutputFormat,
       include_actions: bool,
       include_players: bool,
       include_stats: bool,
   ) -> ExitCode {
       // Read and parse
       let data = match std::fs::read(file) {
           Ok(d) => d,
           Err(e) => {
               eprintln!("Error reading file: {}", e);
               return ExitCode::FAILURE;
           }
       };

       let (header, game_record, decompressed) = match parse_replay(&data) {
           Ok(r) => r,
           Err(e) => {
               eprintln!("Error: {}", e);
               return ExitCode::FAILURE;
           }
       };

       // Build output
       let output_data = build_output(
           &header,
           &game_record,
           &decompressed,
           data.len(),
           include_actions,
           include_players,
           include_stats,
       );

       // Format and print
       match output {
           OutputFormat::Json => print_json(&output_data),
           OutputFormat::Pretty => print_pretty(&output_data),
       }

       ExitCode::SUCCESS
   }
   ```

3. **Implement JSON output**
   ```rust
   fn print_json(output: &ParseOutput) {
       match serde_json::to_string_pretty(output) {
           Ok(json) => println!("{}", json),
           Err(e) => eprintln!("Error serializing to JSON: {}", e),
       }
   }
   ```

4. **Implement pretty output**
   ```rust
   fn print_pretty(output: &ParseOutput) {
       if let Some(header) = &output.header {
           println!("=== Header ===");
           println!("Format: {}", header.format);
           println!("File Size: {} bytes", header.file_size);
           // ... more fields
           println!();
       }

       if let Some(players) = &output.players {
           println!("=== Players ({}) ===", players.len());
           for player in players {
               println!("  Slot {}: {}", player.slot_id, player.name);
           }
           println!();
       }

       if let Some(stats) = &output.statistics {
           println!("=== Statistics ===");
           println!("Total Frames: {}", stats.total_frames);
           println!("Total Actions: {}", stats.total_actions);
           println!("\nActions by Type:");
           for (action_type, count) in &stats.actions_by_type {
               println!("  {}: {}", action_type, count);
           }
           println!();
       }
   }
   ```

5. **Implement action collection**
   ```rust
   fn collect_actions(
       game_record: &GameRecord,
       decompressed: &[u8],
   ) -> (Vec<ActionInfo>, Statistics) {
       let mut actions = Vec::new();
       let mut stats = Statistics {
           total_frames: 0,
           total_actions: 0,
           actions_by_type: HashMap::new(),
           actions_by_player: HashMap::new(),
       };

       for frame_result in game_record.timeframes(decompressed) {
           let frame = match frame_result {
               Ok(f) => f,
               Err(_) => continue,
           };

           stats.total_frames += 1;

           for action_result in frame.actions() {
               let action = match action_result {
                   Ok(a) => a,
                   Err(_) => continue,
               };

               stats.total_actions += 1;

               let action_type_str = format!("{:?}", action.action_type);
               *stats.actions_by_type.entry(action_type_str.clone()).or_insert(0) += 1;
               *stats.actions_by_player.entry(action.player_id).or_insert(0) += 1;

               actions.push(ActionInfo {
                   player_id: action.player_id,
                   timestamp_ms: frame.accumulated_time_ms,
                   action_type: action_type_str,
                   details: None, // Can be extended for specific action types
               });
           }
       }

       (actions, stats)
   }
   ```

### JSON Output Schema

```json
{
  "header": {
    "format": "Classic",
    "file_size": 51200,
    "decompressed_size": 245000,
    "build_version": 10036,
    "duration_ms": 932000
  },
  "players": [
    {"slot_id": 1, "name": "Player1"},
    {"slot_id": 2, "name": "Player2"}
  ],
  "actions": [
    {"player_id": 1, "timestamp_ms": 100, "action_type": "Selection"},
    {"player_id": 2, "timestamp_ms": 150, "action_type": "Movement"}
  ],
  "statistics": {
    "total_frames": 1500,
    "total_actions": 3200,
    "actions_by_type": {
      "Selection": 1200,
      "Movement": 800,
      "Ability": 600,
      "Hotkey": 600
    },
    "actions_by_player": {
      "1": 1600,
      "2": 1600
    }
  }
}
```

### Validation Criteria
- [ ] `w3g-parser parse replay.w3g` produces pretty output
- [ ] `w3g-parser parse -o json replay.w3g` produces valid JSON
- [ ] `--actions` flag includes action list
- [ ] `--players` flag includes player details
- [ ] `--stats` flag includes statistics
- [ ] JSON output can be piped to jq

### Estimated Complexity
**Medium** - Requires serde serialization and multiple output formats.

---

## Phase 6D: Validate Command

### Goal
Implement the `validate` subcommand for replay validation with exit codes.

### Files to Modify
- `src/bin/w3g-parser.rs` - Add validate command implementation

### Implementation Steps

1. **Define validation result structure**
   ```rust
   struct ValidationResult {
       header_valid: bool,
       decompression_valid: bool,
       record_parsing_valid: bool,
       errors: Vec<String>,
       warnings: Vec<String>,
   }

   impl ValidationResult {
       fn is_valid(&self) -> bool {
           self.header_valid && self.decompression_valid && self.record_parsing_valid
       }
   }
   ```

2. **Implement validate command handler**
   ```rust
   fn cmd_validate(file: &Path, verbose: bool) -> ExitCode {
       let result = validate_replay(file);

       if verbose {
           print_validation_details(&result, file);
       } else {
           print_validation_summary(&result, file);
       }

       if result.is_valid() {
           ExitCode::SUCCESS
       } else {
           ExitCode::FAILURE
       }
   }
   ```

3. **Implement validation logic**
   ```rust
   fn validate_replay(file: &Path) -> ValidationResult {
       let mut result = ValidationResult {
           header_valid: false,
           decompression_valid: false,
           record_parsing_valid: false,
           errors: Vec::new(),
           warnings: Vec::new(),
       };

       // Step 1: Read file
       let data = match std::fs::read(file) {
           Ok(d) => d,
           Err(e) => {
               result.errors.push(format!("Failed to read file: {}", e));
               return result;
           }
       };

       // Step 2: Validate header
       let header = match Header::parse(&data) {
           Ok(h) => {
               result.header_valid = true;
               h
           }
           Err(e) => {
               result.errors.push(format!("Header parsing failed: {}", e));
               return result;
           }
       };

       // Step 3: Validate decompression
       let decompressed = match decompress(&data, &header) {
           Ok(d) => {
               result.decompression_valid = true;
               d
           }
           Err(e) => {
               result.errors.push(format!("Decompression failed: {}", e));
               return result;
           }
       };

       // Verify decompressed size
       if decompressed.len() as u32 != header.decompressed_size() {
           result.warnings.push(format!(
               "Decompressed size mismatch: expected {}, got {}",
               header.decompressed_size(),
               decompressed.len()
           ));
       }

       // Step 4: Validate record parsing
       match GameRecord::parse(&decompressed) {
           Ok(record) => {
               result.record_parsing_valid = true;

               // Additional validation checks
               if record.player_count() == 0 {
                   result.warnings.push("No players found in replay".to_string());
               }
           }
           Err(e) => {
               result.errors.push(format!("Record parsing failed: {}", e));
           }
       }

       result
   }
   ```

4. **Implement output formatting**
   ```rust
   fn print_validation_summary(result: &ValidationResult, file: &Path) {
       let status = if result.is_valid() { "VALID" } else { "INVALID" };
       println!("{}: {}", file.display(), status);
   }

   fn print_validation_details(result: &ValidationResult, file: &Path) {
       println!("Validating: {}\n", file.display());

       println!("Checks:");
       println!("  Header parsing:    {}", status_icon(result.header_valid));
       println!("  Decompression:     {}", status_icon(result.decompression_valid));
       println!("  Record parsing:    {}", status_icon(result.record_parsing_valid));

       if !result.errors.is_empty() {
           println!("\nErrors:");
           for error in &result.errors {
               println!("  - {}", error);
           }
       }

       if !result.warnings.is_empty() {
           println!("\nWarnings:");
           for warning in &result.warnings {
               println!("  - {}", warning);
           }
       }

       println!("\nResult: {}", if result.is_valid() { "VALID" } else { "INVALID" });
   }

   fn status_icon(valid: bool) -> &'static str {
       if valid { "[OK]" } else { "[FAIL]" }
   }
   ```

### Output Examples

**Quiet mode:**
```
replay.w3g: VALID
```

**Verbose mode (success):**
```
Validating: replay.w3g

Checks:
  Header parsing:    [OK]
  Decompression:     [OK]
  Record parsing:    [OK]

Result: VALID
```

**Verbose mode (failure):**
```
Validating: corrupt.w3g

Checks:
  Header parsing:    [OK]
  Decompression:     [FAIL]
  Record parsing:    [FAIL]

Errors:
  - Decompression failed: zlib error: invalid block type

Result: INVALID
```

### Validation Criteria
- [ ] Exit code 0 for valid replays
- [ ] Exit code 1 for invalid replays
- [ ] `-v` shows detailed validation steps
- [ ] Works in shell scripts: `if w3g-parser validate file.w3g; then echo OK; fi`
- [ ] All 27 test replays pass validation

### Estimated Complexity
**Low** - Straightforward validation pipeline with exit codes.

---

## Phase 6E: Batch Command (Stretch Goal)

### Goal
Implement the `batch` subcommand for processing multiple replays.

### Files to Modify
- `src/bin/w3g-parser.rs` - Add batch command implementation

### Implementation Steps

1. **Implement directory traversal**
   ```rust
   fn find_replays(directory: &Path) -> Vec<PathBuf> {
       let mut replays = Vec::new();

       if let Ok(entries) = std::fs::read_dir(directory) {
           for entry in entries.flatten() {
               let path = entry.path();
               if path.extension().map_or(false, |e| e == "w3g") {
                   replays.push(path);
               }
           }
       }

       replays.sort();
       replays
   }
   ```

2. **Implement batch command handler**
   ```rust
   fn cmd_batch(
       directory: &Path,
       output_dir: Option<PathBuf>,
       format: OutputFormat,
       summary: bool,
       continue_on_error: bool,
   ) -> ExitCode {
       let replays = find_replays(directory);

       if replays.is_empty() {
           eprintln!("No .w3g files found in {}", directory.display());
           return ExitCode::FAILURE;
       }

       eprintln!("Found {} replay files", replays.len());

       let mut success_count = 0;
       let mut error_count = 0;
       let mut results = Vec::new();

       for replay in &replays {
           eprint!("Processing {}... ", replay.file_name().unwrap_or_default().to_string_lossy());

           match process_replay(replay, &output_dir, &format) {
               Ok(result) => {
                   eprintln!("OK");
                   success_count += 1;
                   results.push((replay.clone(), result));
               }
               Err(e) => {
                   eprintln!("ERROR: {}", e);
                   error_count += 1;
                   if !continue_on_error {
                       return ExitCode::FAILURE;
                   }
               }
           }
       }

       eprintln!("\nProcessed: {} success, {} errors", success_count, error_count);

       if summary {
           generate_summary(&results, &output_dir);
       }

       if error_count > 0 && !continue_on_error {
           ExitCode::FAILURE
       } else {
           ExitCode::SUCCESS
       }
   }
   ```

3. **Implement output file generation**
   ```rust
   fn process_replay(
       replay: &Path,
       output_dir: &Option<PathBuf>,
       format: &OutputFormat,
   ) -> std::result::Result<ParseOutput, String> {
       // Parse replay
       let data = std::fs::read(replay).map_err(|e| e.to_string())?;
       let (header, game_record, decompressed) = parse_replay(&data).map_err(|e| e.to_string())?;

       let output = build_output(&header, &game_record, &decompressed, data.len(), false, true, true);

       // Write to file if output directory specified
       if let Some(dir) = output_dir {
           let output_file = dir.join(
               replay.file_stem().unwrap_or_default()
           ).with_extension("json");

           let json = serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?;
           std::fs::write(&output_file, json).map_err(|e| e.to_string())?;
       }

       Ok(output)
   }
   ```

4. **Implement summary generation**
   ```rust
   #[derive(Serialize)]
   struct BatchSummary {
       total_files: usize,
       successful: usize,
       failed: usize,
       total_actions: usize,
       format_distribution: HashMap<String, usize>,
       average_duration_ms: Option<u32>,
   }

   fn generate_summary(results: &[(PathBuf, ParseOutput)], output_dir: &Option<PathBuf>) {
       let summary = BatchSummary {
           total_files: results.len(),
           successful: results.len(),
           failed: 0,
           // ... calculate other fields
       };

       println!("\n=== Batch Summary ===");
       println!("Files processed: {}", summary.total_files);
       println!("Successful: {}", summary.successful);

       if let Some(dir) = output_dir {
           let summary_file = dir.join("summary.json");
           if let Ok(json) = serde_json::to_string_pretty(&summary) {
               let _ = std::fs::write(&summary_file, json);
               println!("\nSummary written to: {}", summary_file.display());
           }
       }
   }
   ```

### Validation Criteria
- [ ] `w3g-parser batch replays/` processes all .w3g files
- [ ] `-o output/` writes individual JSON files
- [ ] `--summary` generates summary.json
- [ ] `--continue-on-error` doesn't stop on failures
- [ ] Progress shown during processing
- [ ] All 27 test replays process successfully

### Estimated Complexity
**Medium** - Directory traversal and file I/O coordination.

---

## Output Format Specifications

### JSON Schema

The JSON output follows this schema:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "header": {
      "type": "object",
      "properties": {
        "format": {"type": "string", "enum": ["Classic", "Grbn"]},
        "file_size": {"type": "integer"},
        "decompressed_size": {"type": "integer"},
        "build_version": {"type": "integer"},
        "duration_ms": {"type": "integer"}
      },
      "required": ["format", "file_size", "decompressed_size"]
    },
    "players": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "slot_id": {"type": "integer"},
          "name": {"type": "string"}
        },
        "required": ["slot_id", "name"]
      }
    },
    "actions": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "player_id": {"type": "integer"},
          "timestamp_ms": {"type": "integer"},
          "action_type": {"type": "string"}
        },
        "required": ["player_id", "timestamp_ms", "action_type"]
      }
    },
    "statistics": {
      "type": "object",
      "properties": {
        "total_frames": {"type": "integer"},
        "total_actions": {"type": "integer"},
        "actions_by_type": {"type": "object"},
        "actions_by_player": {"type": "object"}
      }
    }
  }
}
```

### Pretty Format

The pretty format uses ASCII formatting:
- Section headers: `=== Title ===`
- Indentation: 2 spaces
- Key-value pairs: `Key: Value`
- Lists: `  - Item`

---

## Edge Cases

### Invalid Files
- **Missing file**: Exit code 1, error message to stderr
- **Empty file**: Exit code 1, error message "Invalid header: file too short"
- **Truncated file**: Continue as far as possible, report what was parsed

### Large Files
- **Large replay (>100MB)**: Should handle with reasonable memory
- **Many actions**: Statistics still computed correctly

### Batch Processing
- **Empty directory**: Exit code 1, "No .w3g files found"
- **Mixed valid/invalid**: `--continue-on-error` processes all, reports summary
- **Output directory doesn't exist**: Create it

---

## Risks

### Risk 1: Action parsing errors
- **Impact**: Parse command may fail on some replays
- **Mitigation**: Use permissive parsing, report what we can

### Risk 2: Large JSON output
- **Impact**: `--actions` flag may produce multi-MB output
- **Mitigation**: Document the flag's behavior; suggest streaming for large files

### Risk 3: Platform compatibility
- **Impact**: Path handling differences on Windows
- **Mitigation**: Use PathBuf consistently, test on multiple platforms

---

## Success Criteria

1. **All commands work**: info, parse, validate, batch all function correctly
2. **Test replays**: All 27 test replays work with all commands
3. **Exit codes**: Proper exit codes for scripting (0 success, 1 failure)
4. **JSON validity**: JSON output parses correctly with jq
5. **Documentation**: `--help` provides useful information for each command
6. **Error messages**: Clear error messages to stderr
7. **Performance**: Processing 27 replays in batch takes < 30 seconds

---

## Implementation Order

1. **Phase 6A** (Framework) - Must be done first
2. **Phase 6B** (Info) - Simplest command, validates pipeline
3. **Phase 6D** (Validate) - Important for scripting, also simple
4. **Phase 6C** (Parse) - More complex with JSON output
5. **Phase 6E** (Batch) - Can be done last, optional

Total estimated time: 1-2 days

---

## Testing Strategy

### Unit Tests
- Argument parsing with various flag combinations
- Output formatting functions

### Integration Tests
- Run each command on test replays
- Verify JSON output parses correctly
- Verify exit codes

### Manual Testing
```bash
# Phase 6A
cargo build --bin w3g-parser
./target/debug/w3g-parser --help
./target/debug/w3g-parser --version

# Phase 6B
./target/debug/w3g-parser info ../replays/replay_1.w3g

# Phase 6D
./target/debug/w3g-parser validate ../replays/replay_1.w3g
echo $?

# Phase 6C
./target/debug/w3g-parser parse ../replays/replay_1.w3g
./target/debug/w3g-parser parse -o json ../replays/replay_1.w3g | jq .

# Phase 6E
./target/debug/w3g-parser batch ../replays/ --summary
```

### Validation Script
```bash
#!/bin/bash
# Validate all test replays
for f in ../replays/*.w3g; do
    if ./target/debug/w3g-parser validate "$f"; then
        echo "PASS: $f"
    else
        echo "FAIL: $f"
        exit 1
    fi
done
echo "All replays validated successfully"
```
