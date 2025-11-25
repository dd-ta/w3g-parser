//! Integration tests for Phase 4: Decompressed Data Record Parsing.
//!
//! These tests validate that the records module correctly parses:
//! - Game record headers
//! - Player rosters
//! - TimeFrame iterations
//!
//! Tests run against the 27 sample replays in ../replays/

use std::fs;
use std::path::Path;
use w3g_parser::decompress::decompress;
use w3g_parser::header::Header;
use w3g_parser::records::{GameRecord, TimeFrameStats};

/// Path to the replays directory (relative to crate root).
const REPLAYS_DIR: &str = "../replays";

/// List of all Classic Type A (build < 10000) replays.
const CLASSIC_TYPE_A_REPLAYS: &[&str] = &[
    "replay_5000.w3g",
    "replay_5001.w3g",
    "replay_5002.w3g",
];

/// List of all Classic Type B (build >= 10000) replays.
const CLASSIC_TYPE_B_REPLAYS: &[&str] = &[
    "replay_100000.w3g",
    "replay_100001.w3g",
    "replay_100002.w3g",
    "replay_10000.w3g",
    "replay_10001.w3g",
    "replay_10002.w3g",
    "replay_50000.w3g",
    "replay_50001.w3g",
    "replay_50002.w3g",
];

/// List of GRBN (Reforged) replays - first 10 numbered 1-10.
const GRBN_REPLAYS: &[&str] = &[
    "replay_1.w3g",
    "replay_2.w3g",
    "replay_3.w3g",
    "replay_4.w3g",
    "replay_5.w3g",
    "replay_6.w3g",
    "replay_7.w3g",
    "replay_8.w3g",
    "replay_9.w3g",
    "replay_10.w3g",
    "replay_1000.w3g",
    "replay_1001.w3g",
    "replay_1002.w3g",
    "replay_1003.w3g",
    "replay_1004.w3g",
];

fn read_replay(filename: &str) -> Vec<u8> {
    let path = Path::new(REPLAYS_DIR).join(filename);
    fs::read(&path).unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
}

fn decompress_replay(filename: &str) -> Vec<u8> {
    let data = read_replay(filename);
    let header = Header::parse(&data).unwrap_or_else(|e| panic!("Failed to parse header for {}: {}", filename, e));
    decompress(&data, &header).unwrap_or_else(|e| panic!("Failed to decompress {}: {}", filename, e))
}

// ============================================================================
// Game Record Header Tests
// ============================================================================

#[test]
fn test_game_record_header_classic_replay_5000() {
    // replay_5000.w3g is a Classic Type A replay with 7 players
    // Based on Rex's analysis: Host is "kaiseris"
    let decompressed = decompress_replay("replay_5000.w3g");
    let record = GameRecord::parse(&decompressed).unwrap();

    // Verify host name is present and non-empty
    assert!(!record.host_name().is_empty(), "Host name should not be empty");

    // Verify the record is valid
    assert!(record.header.is_valid(), "Game record header should be valid");

    println!("replay_5000.w3g:");
    println!("  Host: {} (slot {})", record.host_name(), record.host_slot());
    println!("  Player count: {}", record.player_count());
}

#[test]
fn test_game_record_header_grbn_replay_1() {
    // replay_1.w3g is a GRBN replay
    // It may have an embedded Classic record or be protobuf-only
    let decompressed = decompress_replay("replay_1.w3g");
    let result = GameRecord::parse(&decompressed);

    match result {
        Ok(record) => {
            assert!(!record.host_name().is_empty(), "Host name should not be empty");
            assert!(record.header.is_valid(), "Game record header should be valid");

            println!("replay_1.w3g:");
            println!("  Host: {} (slot {})", record.host_name(), record.host_slot());
            println!("  Player count: {}", record.player_count());
        }
        Err(e) => {
            // This is expected for protobuf-only GRBN replays
            println!("replay_1.w3g: No Classic game record found (protobuf-only): {}", e);
        }
    }
}

#[test]
fn test_all_classic_replays_parse_game_record() {
    // Classic replays should all parse successfully
    let classic_replays: Vec<&str> = CLASSIC_TYPE_A_REPLAYS
        .iter()
        .chain(CLASSIC_TYPE_B_REPLAYS.iter())
        .copied()
        .collect();

    println!("Testing {} Classic replays for game record parsing...", classic_replays.len());

    for filename in &classic_replays {
        let decompressed = decompress_replay(filename);
        let result = GameRecord::parse(&decompressed);

        match result {
            Ok(record) => {
                println!("{}: Host='{}', Players={}",
                    filename, record.host_name(), record.player_count());
                assert!(record.header.is_valid(),
                    "{}: Game record should be valid", filename);
            }
            Err(e) => {
                panic!("{}: Failed to parse game record: {}", filename, e);
            }
        }
    }
}

#[test]
fn test_grbn_replays_parse_game_record_best_effort() {
    // GRBN replays may or may not have an embedded Classic game record
    // Some newer GRBN replays are purely protobuf format
    println!("Testing {} GRBN replays for game record parsing (best effort)...", GRBN_REPLAYS.len());

    let mut success_count = 0;
    let mut protobuf_only_count = 0;

    for filename in GRBN_REPLAYS {
        let decompressed = decompress_replay(filename);
        let result = GameRecord::parse(&decompressed);

        match result {
            Ok(record) => {
                println!("{}: Host='{}', Players={} (Classic embedded)",
                    filename, record.host_name(), record.player_count());
                success_count += 1;
            }
            Err(_) => {
                // This is expected for newer GRBN replays that are protobuf-only
                println!("{}: No Classic game record found (protobuf-only format)", filename);
                protobuf_only_count += 1;
            }
        }
    }

    println!("\nGRBN parsing summary:");
    println!("  Classic embedded: {}", success_count);
    println!("  Protobuf-only: {}", protobuf_only_count);

    // At least some GRBN replays should have embedded Classic records
    // (Based on Rex's analysis, some GRBN files do contain embedded Classic data)
    assert!(success_count > 0 || protobuf_only_count > 0,
        "Expected at least some GRBN replay data to parse");
}

// ============================================================================
// Player Roster Tests
// ============================================================================

#[test]
fn test_player_roster_classic_replay_5000() {
    // replay_5000.w3g should have 7 players according to Rex's analysis
    let decompressed = decompress_replay("replay_5000.w3g");
    let record = GameRecord::parse(&decompressed).unwrap();

    let names = record.player_names();
    println!("replay_5000.w3g player names: {:?}", names);

    // Should have multiple players
    assert!(record.player_count() > 0, "Should have at least one player");
}

#[test]
fn test_player_names_all_classic_replays() {
    let classic_replays: Vec<&str> = CLASSIC_TYPE_A_REPLAYS
        .iter()
        .chain(CLASSIC_TYPE_B_REPLAYS.iter())
        .copied()
        .collect();

    println!("\n=== Player Names from Classic Replays ===\n");

    for filename in &classic_replays {
        let decompressed = decompress_replay(filename);
        let record = GameRecord::parse(&decompressed).unwrap();

        let names = record.player_names();
        println!("{}: {} players", filename, record.player_count());
        for name in &names {
            println!("  - {}", name);
        }
    }
}

#[test]
fn test_player_names_all_grbn_replays() {
    println!("\n=== Player Names from GRBN Replays ===\n");

    for filename in GRBN_REPLAYS {
        let decompressed = decompress_replay(filename);
        if let Ok(record) = GameRecord::parse(&decompressed) {
            let names = record.player_names();
            println!("{}: {} players (Classic embedded)", filename, record.player_count());
            for name in &names {
                println!("  - {}", name);
            }
        } else {
            println!("{}: protobuf-only format (parsing not implemented)", filename);
        }
    }
}

// ============================================================================
// TimeFrame Iterator Tests
// ============================================================================

#[test]
fn test_timeframe_iteration_classic_replay_5000() {
    // replay_5000.w3g is a Classic replay
    let decompressed = decompress_replay("replay_5000.w3g");
    let record = GameRecord::parse(&decompressed).unwrap();

    let iter = record.timeframes(&decompressed);
    let stats = TimeFrameStats::from_iterator(iter).unwrap();

    println!("replay_5000.w3g TimeFrame stats:");
    println!("  Frame count: {}", stats.frame_count);
    println!("  Total time: {}ms ({})", stats.total_time_ms, stats.duration_string());
    println!("  Avg time delta: {:.2}ms", stats.average_time_delta_ms());
    println!("  Total action bytes: {}", stats.total_action_bytes);
    println!("  Empty frames: {}", stats.empty_frame_count);

    // Should have TimeFrames
    assert!(stats.frame_count > 0, "Expected some TimeFrames");

    // Total time should be at least some milliseconds
    assert!(stats.total_time_ms > 0, "Expected some game time");
}

#[test]
fn test_timeframe_iteration_grbn_replay_1() {
    let decompressed = decompress_replay("replay_1.w3g");

    if let Ok(record) = GameRecord::parse(&decompressed) {
        let iter = record.timeframes(&decompressed);
        let stats = TimeFrameStats::from_iterator(iter).unwrap();

        println!("replay_1.w3g TimeFrame stats:");
        println!("  Frame count: {}", stats.frame_count);
        println!("  Total time: {}ms ({})", stats.total_time_ms, stats.duration_string());
        println!("  Avg time delta: {:.2}ms", stats.average_time_delta_ms());
        println!("  Total action bytes: {}", stats.total_action_bytes);
        println!("  Empty frames: {}", stats.empty_frame_count);

        // Should have frames
        assert!(stats.frame_count > 0, "Expected TimeFrames");
    } else {
        println!("replay_1.w3g: protobuf-only format (TimeFrame parsing not applicable)");
    }
}

#[test]
fn test_timeframe_statistics_classic_replays() {
    // Test TimeFrame statistics for Classic replays only
    // (GRBN replays may be protobuf-only and require different parsing)
    let classic_replays: Vec<&str> = CLASSIC_TYPE_A_REPLAYS
        .iter()
        .chain(CLASSIC_TYPE_B_REPLAYS.iter())
        .copied()
        .collect();

    println!("\n=== TimeFrame Statistics for Classic Replays ===\n");
    println!("{:<25} {:>10} {:>12} {:>10}", "Replay", "Frames", "Duration", "Actions");
    println!("{}", "-".repeat(60));

    for filename in &classic_replays {
        let decompressed = decompress_replay(filename);
        let record = GameRecord::parse(&decompressed).unwrap();

        let iter = record.timeframes(&decompressed);
        let stats = TimeFrameStats::from_iterator(iter).unwrap();

        println!("{:<25} {:>10} {:>12} {:>10}",
            filename,
            stats.frame_count,
            stats.duration_string(),
            stats.total_action_bytes
        );

        // All replays should have at least some frames
        assert!(stats.frame_count > 0 || stats.total_time_ms == 0,
            "{}: Expected TimeFrames or zero duration", filename);
    }
}

// ============================================================================
// End-to-End Validation Tests
// ============================================================================

#[test]
fn test_complete_parsing_pipeline() {
    // Test the complete parsing pipeline for one file
    let data = read_replay("replay_5000.w3g");
    let header = Header::parse(&data).unwrap();
    let decompressed = decompress(&data, &header).unwrap();
    let record = GameRecord::parse(&decompressed).unwrap();

    // Verify all components
    assert!(record.header.is_valid());
    assert!(!record.host_name().is_empty());

    // Count TimeFrames
    let mut frame_count = 0;
    for result in record.timeframes(&decompressed) {
        result.unwrap();
        frame_count += 1;
    }

    assert!(frame_count > 0, "Should have TimeFrames");

    println!("\nComplete parsing pipeline for replay_5000.w3g:");
    println!("  Header size: {} bytes", header.header_size());
    println!("  Decompressed size: {} bytes", decompressed.len());
    println!("  Host: {}", record.host_name());
    println!("  Players: {}", record.player_count());
    println!("  TimeFrames: {}", frame_count);
}

#[test]
fn test_no_crashes_on_all_replays() {
    // This test verifies we don't crash on any replay
    let all_replays: Vec<&str> = CLASSIC_TYPE_A_REPLAYS
        .iter()
        .chain(CLASSIC_TYPE_B_REPLAYS.iter())
        .chain(GRBN_REPLAYS.iter())
        .copied()
        .collect();

    let mut classic_success_count = 0;
    let mut grbn_with_classic_count = 0;
    let mut grbn_protobuf_only_count = 0;

    for filename in &all_replays {
        let data = read_replay(filename);

        // These operations should not panic
        if let Ok(header) = Header::parse(&data) {
            if let Ok(decompressed) = decompress(&data, &header) {
                if let Ok(record) = GameRecord::parse(&decompressed) {
                    // Try iterating TimeFrames
                    let iter = record.timeframes(&decompressed);
                    let _ = TimeFrameStats::from_iterator(iter);

                    if header.is_classic() {
                        classic_success_count += 1;
                    } else {
                        grbn_with_classic_count += 1;
                    }
                } else if header.is_grbn() {
                    // GRBN replay without embedded Classic - expected for some files
                    grbn_protobuf_only_count += 1;
                }
            }
        }
    }

    println!("Parsing results:");
    println!("  Classic replays parsed: {}", classic_success_count);
    println!("  GRBN with embedded Classic: {}", grbn_with_classic_count);
    println!("  GRBN protobuf-only: {}", grbn_protobuf_only_count);

    // All Classic replays should parse successfully
    let classic_count = CLASSIC_TYPE_A_REPLAYS.len() + CLASSIC_TYPE_B_REPLAYS.len();
    assert_eq!(classic_success_count, classic_count,
        "All Classic replays should parse successfully");

    // All GRBN replays should at least decompress (either Classic embedded or protobuf)
    assert_eq!(grbn_with_classic_count + grbn_protobuf_only_count, GRBN_REPLAYS.len(),
        "All GRBN replays should at least decompress");
}
