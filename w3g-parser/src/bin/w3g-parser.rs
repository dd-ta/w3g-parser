//! Warcraft 3 replay (.w3g) parser CLI
//!
//! A command-line interface for parsing, validating, and analyzing W3G replay files.
//!
//! ## Commands
//!
//! - `info` - Display quick replay metadata
//! - `parse` - Parse replay with output format options
//! - `validate` - Validate replay format (exit codes for scripting)
//! - `batch` - Process multiple replays from a directory

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use w3g_parser::records::{ChatMessage, CHAT_MARKER};
use w3g_parser::{decompress, GameRecord, Header};

/// Warcraft 3 replay (.w3g) parser
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
        /// Output format: json, pretty
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
        /// Include chat messages
        #[arg(long)]
        chat: bool,
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
        /// Output directory for JSON files
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Output format
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

/// Output format options
#[derive(Clone, Debug, ValueEnum)]
enum OutputFormat {
    Json,
    Pretty,
}

// ============================================================================
// Serializable Output Structures
// ============================================================================

#[derive(Serialize)]
struct ParseOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    header: Option<HeaderInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    players: Option<Vec<PlayerInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chat: Option<Vec<ChatInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actions: Option<Vec<ActionInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    statistics: Option<Statistics>,
}

#[derive(Serialize)]
struct ChatInfo {
    flags: u8,
    message_id: u16,
    message: String,
}

#[derive(Serialize)]
struct HeaderInfo {
    format: String,
    file_size: usize,
    decompressed_size: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    build_version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    game_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<String>,
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
}

#[derive(Serialize, Default)]
struct Statistics {
    total_frames: usize,
    total_actions: usize,
    actions_by_type: HashMap<String, usize>,
    actions_by_player: HashMap<u8, usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    player_stats: Option<HashMap<u8, PlayerStats>>,
}

/// Per-player action statistics matching warcraft3.info categories.
#[derive(Serialize, Default, Clone)]
struct PlayerStats {
    /// Player name (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// Actions per minute.
    apm: f64,
    /// Total actions for this player.
    total: usize,
    /// Right-click (movement) actions.
    rightclick: usize,
    /// Basic commands (stop, hold position, patrol).
    basic: usize,
    /// Build/train commands.
    buildtrain: usize,
    /// Ability/spell usage.
    ability: usize,
    /// Item usage.
    item: usize,
    /// Unit selection actions.
    select: usize,
    /// Assign group (Ctrl+N).
    assigngroup: usize,
    /// Select hotkey (N to select group).
    selecthotkey: usize,
    /// ESC key presses.
    esc: usize,
    /// Unknown/other actions.
    other: usize,
}

#[derive(Serialize)]
struct BatchSummary {
    total_files: usize,
    successful: usize,
    failed: usize,
    total_actions: usize,
    format_distribution: HashMap<String, usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    average_duration_ms: Option<u32>,
}

// ============================================================================
// Validation Result Structure
// ============================================================================

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

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { file } => cmd_info(&file),
        Commands::Parse {
            file,
            output,
            actions,
            players,
            stats,
            chat,
        } => cmd_parse(&file, output, actions, players, stats, chat),
        Commands::Validate { file, verbose } => cmd_validate(&file, verbose),
        Commands::Batch {
            directory,
            output,
            format,
            summary,
            continue_on_error,
        } => cmd_batch(&directory, output, format, summary, continue_on_error),
    }
}

// ============================================================================
// Info Command Implementation
// ============================================================================

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
    print_info(&header, &game_record, data.len());

    ExitCode::SUCCESS
}

#[allow(clippy::cast_precision_loss)]
fn print_info(header: &Header, game_record: &GameRecord, file_size: usize) {
    println!("=== Replay Information ===\n");

    // File information
    println!("File:");
    println!(
        "  Size: {} bytes ({:.2} KB)",
        file_size,
        file_size as f64 / 1024.0
    );
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

// ============================================================================
// Parse Command Implementation
// ============================================================================

fn cmd_parse(
    file: &Path,
    output: OutputFormat,
    include_actions: bool,
    include_players: bool,
    include_stats: bool,
    include_chat: bool,
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
        include_chat,
    );

    // Format and print
    match output {
        OutputFormat::Json => print_json(&output_data),
        OutputFormat::Pretty => print_pretty(&output_data),
    }

    ExitCode::SUCCESS
}

fn parse_replay(data: &[u8]) -> Result<(Header, GameRecord, Vec<u8>), String> {
    let header = Header::parse(data).map_err(|e| format!("Header parsing failed: {}", e))?;
    let decompressed =
        decompress(data, &header).map_err(|e| format!("Decompression failed: {}", e))?;
    let game_record =
        GameRecord::parse(&decompressed).map_err(|e| format!("Record parsing failed: {}", e))?;
    Ok((header, game_record, decompressed))
}

fn build_output(
    header: &Header,
    game_record: &GameRecord,
    decompressed: &[u8],
    file_size: usize,
    include_actions: bool,
    include_players: bool,
    include_stats: bool,
    include_chat: bool,
) -> ParseOutput {
    // Always include header info (pass player count for game mode inference)
    let header_info = Some(build_header_info(header, file_size, game_record.player_count()));

    // Build player list if requested
    let players = if include_players {
        Some(build_player_info(game_record))
    } else {
        None
    };

    // Build chat messages if requested
    let chat = if include_chat {
        Some(collect_chat_messages(decompressed, game_record.timeframe_offset))
    } else {
        None
    };

    // Build actions and stats if requested
    let (actions, statistics) = if include_actions || include_stats {
        // Get duration for APM calculation
        let duration_ms = match header {
            Header::Classic(h) => Some(h.duration_ms),
            Header::Grbn(_) => None,
        };
        // Get player names for stats
        let player_names: HashMap<u8, String> = game_record
            .players
            .players()
            .map(|p| (p.slot_id(), p.player_name().to_string()))
            .collect();

        let (action_list, stats) =
            collect_actions(game_record, decompressed, duration_ms, &player_names);
        (
            if include_actions {
                Some(action_list)
            } else {
                None
            },
            if include_stats { Some(stats) } else { None },
        )
    } else {
        (None, None)
    };

    ParseOutput {
        header: header_info,
        players,
        chat,
        actions,
        statistics,
    }
}

fn build_header_info(header: &Header, file_size: usize, player_count: usize) -> HeaderInfo {
    match header {
        Header::Classic(h) => HeaderInfo {
            format: "Classic".to_string(),
            file_size,
            decompressed_size: h.decompressed_size,
            build_version: Some(h.build_version),
            version: Some(h.version_string()),
            game_mode: Some(infer_game_mode(player_count)),
            duration_ms: Some(h.duration_ms),
            duration: Some(h.duration_string()),
        },
        Header::Grbn(h) => HeaderInfo {
            format: "Grbn".to_string(),
            file_size,
            decompressed_size: h.decompressed_size,
            build_version: None,
            version: None,
            game_mode: Some(infer_game_mode(player_count)),
            duration_ms: None,
            duration: None,
        },
    }
}

/// Infers game mode from player count.
fn infer_game_mode(player_count: usize) -> String {
    match player_count {
        0 | 1 => "Solo".to_string(),
        2 => "1v1".to_string(),
        3 => "FFA".to_string(),
        4 => "2v2".to_string(),
        6 => "3v3".to_string(),
        8 => "4v4".to_string(),
        n => format!("{}p", n),
    }
}

fn build_player_info(game_record: &GameRecord) -> Vec<PlayerInfo> {
    let mut players = Vec::new();

    // Add players from the roster using the public API
    for record in game_record.players.players() {
        players.push(PlayerInfo {
            slot_id: record.slot_id(),
            name: record.player_name().to_string(),
        });
    }

    players
}

fn collect_actions(
    game_record: &GameRecord,
    decompressed: &[u8],
    duration_ms: Option<u32>,
    player_names: &HashMap<u8, String>,
) -> (Vec<ActionInfo>, Statistics) {
    use w3g_parser::actions::{ActionType, HotkeyOperation};

    let mut actions = Vec::new();
    let mut stats = Statistics::default();
    let mut player_stats: HashMap<u8, PlayerStats> = HashMap::new();

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
            *stats
                .actions_by_type
                .entry(action_type_str.clone())
                .or_insert(0) += 1;
            *stats
                .actions_by_player
                .entry(action.player_id)
                .or_insert(0) += 1;

            // Update per-player stats
            let ps = player_stats.entry(action.player_id).or_default();
            ps.total += 1;

            // Categorize action (matching warcraft3.info categories)
            match &action.action_type {
                ActionType::Movement(_) => ps.rightclick += 1,
                ActionType::BasicCommand { .. } => ps.basic += 1,
                ActionType::BuildTrain { .. } => ps.buildtrain += 1,
                ActionType::Ability(_)
                | ActionType::AbilityWithSelection(_)
                | ActionType::InstantAbility(_) => ps.ability += 1,
                ActionType::ItemAction { .. } => ps.item += 1,
                ActionType::Selection(_) => ps.select += 1,
                ActionType::Hotkey(hk) => match hk.operation {
                    HotkeyOperation::Assign | HotkeyOperation::AddToGroup => ps.assigngroup += 1,
                    HotkeyOperation::Select => ps.selecthotkey += 1,
                    HotkeyOperation::Unknown(_) => ps.other += 1,
                },
                ActionType::EscapeKey => ps.esc += 1,
                ActionType::Unknown { .. } => ps.other += 1,
            }

            actions.push(ActionInfo {
                player_id: action.player_id,
                timestamp_ms: frame.accumulated_time_ms,
                action_type: action_type_str,
            });
        }
    }

    // Calculate APM and add player names
    if let Some(duration) = duration_ms {
        let minutes = f64::from(duration) / 60000.0;
        for (player_id, ps) in &mut player_stats {
            ps.name = player_names.get(player_id).cloned();
            if minutes > 0.0 {
                ps.apm = (ps.total as f64) / minutes;
            }
        }
    } else {
        // No duration, just add names
        for (player_id, ps) in &mut player_stats {
            ps.name = player_names.get(player_id).cloned();
        }
    }

    stats.player_stats = Some(player_stats);

    (actions, stats)
}

/// Collects all chat messages from decompressed replay data.
///
/// Chat message format (reverse engineered):
/// - Offset 0: 0x20 (marker)
/// - Offset 1: flags (0x00-0x1F, typically 0x03 for system, 0x07 for player)
/// - Offset 2-3: message_id (u16 little-endian)
/// - Offset 4-8: padding signature (0x20 XX 0x00 0x00 0x00)
/// - Offset 9+: null-terminated message
fn collect_chat_messages(decompressed: &[u8], start_offset: usize) -> Vec<ChatInfo> {
    let mut messages = Vec::new();
    let mut offset = start_offset;

    while offset + 9 < decompressed.len() {
        if decompressed[offset] == CHAT_MARKER {
            // Validate chat message structure to filter false positives
            // Check: marker=0x20, flags in valid range, padding signature matches
            let flags = decompressed[offset + 1];

            // Flags should be small (0x00-0x1F range based on reverse engineering)
            // and padding byte at offset 4 should be 0x20
            if flags <= 0x1F
                && offset + 8 < decompressed.len()
                && decompressed[offset + 4] == 0x20
                && decompressed[offset + 6] == 0x00
                && decompressed[offset + 7] == 0x00
                && decompressed[offset + 8] == 0x00
            {
                // Try to parse as a chat message
                if let Ok(chat) = ChatMessage::parse(&decompressed[offset..]) {
                    // Only include meaningful messages:
                    // - At least 2 characters (filter out noise like single punctuation)
                    // - Printable ASCII content
                    if chat.message.len() >= 2
                        && chat.message.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
                    {
                        messages.push(ChatInfo {
                            flags: chat.flags,
                            message_id: chat.message_id,
                            message: chat.message,
                        });
                    }
                    offset += chat.byte_length;
                    continue;
                }
            }
        }
        offset += 1;
    }

    messages
}

fn print_json(output: &ParseOutput) {
    match serde_json::to_string_pretty(output) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to JSON: {}", e),
    }
}

fn print_pretty(output: &ParseOutput) {
    if let Some(header) = &output.header {
        println!("=== Header ===");
        println!("Format: {}", header.format);
        println!("File Size: {} bytes", header.file_size);
        println!("Decompressed Size: {} bytes", header.decompressed_size);
        if let Some(version) = &header.version {
            println!("Version: {}", version);
        }
        if let Some(build) = header.build_version {
            println!("Build: {}", build);
        }
        if let Some(mode) = &header.game_mode {
            println!("Mode: {}", mode);
        }
        if let Some(duration) = &header.duration {
            println!("Duration: {}", duration);
        }
        println!();
    }

    if let Some(players) = &output.players {
        println!("=== Players ({}) ===", players.len());
        for player in players {
            println!("  Slot {}: {}", player.slot_id, player.name);
        }
        println!();
    }

    if let Some(chat) = &output.chat {
        println!("=== Chat Messages ({}) ===", chat.len());
        for msg in chat {
            println!("  [{}] {}", msg.message_id, msg.message);
        }
        println!();
    }

    if let Some(stats) = &output.statistics {
        println!("=== Statistics ===");
        println!("Total Frames: {}", stats.total_frames);
        println!("Total Actions: {}", stats.total_actions);
        println!("\nActions by Type:");
        let mut types: Vec<_> = stats.actions_by_type.iter().collect();
        types.sort_by(|a, b| b.1.cmp(a.1));
        for (action_type, count) in types {
            println!("  {}: {}", action_type, count);
        }

        // Per-player detailed stats
        if let Some(player_stats) = &stats.player_stats {
            println!("\n=== Player Stats ===");
            let mut players: Vec<_> = player_stats.iter().collect();
            players.sort_by_key(|p| p.0);

            for (player_id, ps) in players {
                let name = ps.name.as_deref().unwrap_or("Unknown");
                println!("\nPlayer {} ({}):", player_id, name);
                println!("  APM: {:.1}", ps.apm);
                println!("  Total: {}", ps.total);
                println!("  rightclick: {}", ps.rightclick);
                println!("  basic: {}", ps.basic);
                println!("  buildtrain: {}", ps.buildtrain);
                println!("  ability: {}", ps.ability);
                println!("  item: {}", ps.item);
                println!("  select: {}", ps.select);
                println!("  assigngroup: {}", ps.assigngroup);
                println!("  selecthotkey: {}", ps.selecthotkey);
                println!("  esc: {}", ps.esc);
                if ps.other > 0 {
                    println!("  other: {}", ps.other);
                }
            }
        }
        println!();
    }

    if let Some(actions) = &output.actions {
        println!("=== Actions ({}) ===", actions.len());
        // Only show first 50 actions in pretty mode to avoid spam
        let display_count = std::cmp::min(actions.len(), 50);
        for action in &actions[..display_count] {
            println!(
                "  [{}ms] Player {}: {}",
                action.timestamp_ms, action.player_id, action.action_type
            );
        }
        if actions.len() > 50 {
            println!("  ... and {} more actions", actions.len() - 50);
        }
    }
}

// ============================================================================
// Validate Command Implementation
// ============================================================================

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
    #[allow(clippy::cast_possible_truncation)]
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
                result
                    .warnings
                    .push("No players found in replay".to_string());
            }
        }
        Err(e) => {
            result
                .errors
                .push(format!("Record parsing failed: {}", e));
        }
    }

    result
}

fn print_validation_summary(result: &ValidationResult, file: &Path) {
    let status = if result.is_valid() { "VALID" } else { "INVALID" };
    println!("{}: {}", file.display(), status);
}

fn print_validation_details(result: &ValidationResult, file: &Path) {
    println!("Validating: {}\n", file.display());

    println!("Checks:");
    println!("  Header parsing:    {}", status_icon(result.header_valid));
    println!(
        "  Decompression:     {}",
        status_icon(result.decompression_valid)
    );
    println!(
        "  Record parsing:    {}",
        status_icon(result.record_parsing_valid)
    );

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

    println!(
        "\nResult: {}",
        if result.is_valid() { "VALID" } else { "INVALID" }
    );
}

fn status_icon(valid: bool) -> &'static str {
    if valid {
        "[OK]"
    } else {
        "[FAIL]"
    }
}

// ============================================================================
// Batch Command Implementation
// ============================================================================

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

    // Create output directory if specified and doesn't exist
    if let Some(ref dir) = output_dir {
        if !dir.exists() {
            if let Err(e) = std::fs::create_dir_all(dir) {
                eprintln!("Failed to create output directory: {}", e);
                return ExitCode::FAILURE;
            }
        }
    }

    let mut success_count = 0;
    let mut error_count = 0;
    let mut results: Vec<(PathBuf, ParseOutput)> = Vec::new();
    let mut durations: Vec<u32> = Vec::new();
    let mut format_counts: HashMap<String, usize> = HashMap::new();

    for replay in &replays {
        eprint!(
            "Processing {}... ",
            replay.file_name().unwrap_or_default().to_string_lossy()
        );

        match process_replay(replay, &output_dir, &format) {
            Ok(output) => {
                eprintln!("OK");
                success_count += 1;

                // Collect stats for summary
                if let Some(ref header) = output.header {
                    *format_counts.entry(header.format.clone()).or_insert(0) += 1;
                    if let Some(duration) = header.duration_ms {
                        durations.push(duration);
                    }
                }

                results.push((replay.clone(), output));
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

    eprintln!(
        "\nProcessed: {} success, {} errors",
        success_count, error_count
    );

    if summary {
        generate_summary(&results, &output_dir, &format_counts, &durations);
    }

    if error_count > 0 && !continue_on_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn find_replays(directory: &Path) -> Vec<PathBuf> {
    let mut replays = Vec::new();

    if let Ok(entries) = std::fs::read_dir(directory) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "w3g") {
                replays.push(path);
            }
        }
    }

    replays.sort();
    replays
}

fn process_replay(
    replay: &Path,
    output_dir: &Option<PathBuf>,
    format: &OutputFormat,
) -> Result<ParseOutput, String> {
    // Parse replay
    let data = std::fs::read(replay).map_err(|e| e.to_string())?;
    let (header, game_record, decompressed) = parse_replay(&data)?;

    let output = build_output(
        &header,
        &game_record,
        &decompressed,
        data.len(),
        false, // Don't include actions in batch (too large)
        true,  // Include players
        true,  // Include stats
        false, // Don't include chat in batch (too large)
    );

    // Write to file if output directory specified
    if let Some(dir) = output_dir {
        let output_file = dir
            .join(replay.file_stem().unwrap_or_default())
            .with_extension("json");

        let content = match format {
            OutputFormat::Json => serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?,
            OutputFormat::Pretty => {
                // For pretty format in batch, still write JSON for machine readability
                serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?
            }
        };

        std::fs::write(&output_file, content).map_err(|e| e.to_string())?;
    }

    Ok(output)
}

fn generate_summary(
    results: &[(PathBuf, ParseOutput)],
    output_dir: &Option<PathBuf>,
    format_counts: &HashMap<String, usize>,
    durations: &[u32],
) {
    let total_actions: usize = results
        .iter()
        .filter_map(|(_, output)| output.statistics.as_ref())
        .map(|s| s.total_actions)
        .sum();

    #[allow(clippy::cast_possible_truncation)]
    let avg_duration = if durations.is_empty() {
        None
    } else {
        Some((durations.iter().map(|&d| u64::from(d)).sum::<u64>() / durations.len() as u64) as u32)
    };

    let summary = BatchSummary {
        total_files: results.len(),
        successful: results.len(),
        failed: 0,
        total_actions,
        format_distribution: format_counts.clone(),
        average_duration_ms: avg_duration,
    };

    println!("\n=== Batch Summary ===");
    println!("Files processed: {}", summary.total_files);
    println!("Successful: {}", summary.successful);
    println!("Total actions: {}", summary.total_actions);

    println!("\nFormat distribution:");
    for (format, count) in &summary.format_distribution {
        println!("  {}: {}", format, count);
    }

    if let Some(avg) = summary.average_duration_ms {
        let minutes = avg / 60000;
        let seconds = (avg % 60000) / 1000;
        println!("\nAverage duration: {:02}:{:02}", minutes, seconds);
    }

    if let Some(dir) = output_dir {
        let summary_file = dir.join("summary.json");
        if let Ok(json) = serde_json::to_string_pretty(&summary) {
            if std::fs::write(&summary_file, json).is_ok() {
                println!("\nSummary written to: {}", summary_file.display());
            }
        }
    }
}
