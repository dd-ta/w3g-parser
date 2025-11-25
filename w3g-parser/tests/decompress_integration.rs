//! Integration tests for decompression against real replay files.
//!
//! These tests verify that the decompression functions correctly handle
//! all 27 test replay files from the `/replays/` directory.

use std::fs;
use std::path::Path;

use w3g_parser::decompress::decompress;
use w3g_parser::header::Header;

/// Path to the replays directory relative to the workspace root.
const REPLAYS_DIR: &str = "../replays";

/// Returns the path to a specific replay file.
fn replay_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(REPLAYS_DIR)
        .join(name)
}

/// Reads a replay file and returns its contents.
fn read_replay(name: &str) -> Vec<u8> {
    let path = replay_path(name);
    fs::read(&path).unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
}

// ============================================================================
// Replay Lists
// ============================================================================

/// All GRBN replay file names (15 files).
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

/// All Classic Type A replay file names (build version 26).
const CLASSIC_TYPE_A_REPLAYS: &[&str] = &[
    "replay_5000.w3g",
    "replay_5001.w3g",
    "replay_5002.w3g",
    "replay_10000.w3g",
    "replay_10001.w3g",
    "replay_10002.w3g",
];

/// All Classic Type B replay file names (build version 10000+).
const CLASSIC_TYPE_B_REPLAYS: &[&str] = &[
    "replay_50000.w3g",
    "replay_50001.w3g",
    "replay_50002.w3g",
    "replay_100000.w3g",
    "replay_100001.w3g",
    "replay_100002.w3g",
];

// ============================================================================
// GRBN Decompression Tests
// ============================================================================

#[test]
fn test_all_grbn_replays_decompress() {
    for name in GRBN_REPLAYS {
        let data = read_replay(name);
        let header = Header::parse(&data).unwrap_or_else(|e| {
            panic!("Failed to parse header for {}: {}", name, e);
        });

        let result = decompress(&data, &header);

        assert!(
            result.is_ok(),
            "Failed to decompress {}: {}",
            name,
            result.unwrap_err()
        );

        let decompressed = result.unwrap();
        assert!(
            !decompressed.is_empty(),
            "{} should decompress to non-empty data",
            name
        );
        assert!(
            decompressed.len() > 100_000,
            "{} decompressed size {} is too small",
            name,
            decompressed.len()
        );
    }
}

#[test]
fn test_grbn_replay_1_decompressed_size() {
    // GRBN files contain metadata + embedded Classic replay
    let data = read_replay("replay_1.w3g");
    let header = Header::parse(&data).unwrap();
    let decompressed = decompress(&data, &header).unwrap();

    // GRBN decompressed_size doesn't directly map to what we decompress
    // (it's an internal game engine value). We verify we got substantial data.
    assert!(
        decompressed.len() > 100_000,
        "replay_1 should decompress to substantial data, got {} bytes",
        decompressed.len()
    );
}

#[test]
fn test_grbn_replay_2_decompressed_size() {
    let data = read_replay("replay_2.w3g");
    let header = Header::parse(&data).unwrap();
    let decompressed = decompress(&data, &header).unwrap();

    // Verify we got substantial data from the embedded Classic replay
    assert!(
        decompressed.len() > 100_000,
        "replay_2 should decompress to substantial data, got {} bytes",
        decompressed.len()
    );
}

#[test]
fn test_grbn_replay_1000_decompressed_size() {
    let data = read_replay("replay_1000.w3g");
    let header = Header::parse(&data).unwrap();
    let decompressed = decompress(&data, &header).unwrap();

    // Verify we got substantial data
    assert!(
        decompressed.len() > 100_000,
        "replay_1000 should decompress to substantial data, got {} bytes",
        decompressed.len()
    );
}

// ============================================================================
// Classic Type A Decompression Tests
// ============================================================================

#[test]
fn test_all_classic_type_a_replays_decompress() {
    for name in CLASSIC_TYPE_A_REPLAYS {
        let data = read_replay(name);
        let header = Header::parse(&data).unwrap_or_else(|e| {
            panic!("Failed to parse header for {}: {}", name, e);
        });

        let result = decompress(&data, &header);

        assert!(
            result.is_ok(),
            "Failed to decompress {}: {}",
            name,
            result.unwrap_err()
        );

        let decompressed = result.unwrap();
        assert!(
            !decompressed.is_empty(),
            "{} should decompress to non-empty data",
            name
        );
    }
}

#[test]
fn test_classic_type_a_replay_5000() {
    let data = read_replay("replay_5000.w3g");
    let header = Header::parse(&data).unwrap();
    let decompressed = decompress(&data, &header).unwrap();

    assert!(!decompressed.is_empty());

    // Check that decompressed size is reasonable
    // The decompressed size should be approximately block_count * 8192
    // with the last block potentially being smaller
    if let Header::Classic(h) = &header {
        let expected_min = (h.block_count - 1) as usize * 8192;
        let expected_max = (h.block_count + 1) as usize * 8192;

        assert!(
            decompressed.len() >= expected_min && decompressed.len() <= expected_max,
            "Decompressed size {} should be reasonable for {} blocks in replay_5000.w3g",
            decompressed.len(),
            h.block_count
        );
    }
}

// ============================================================================
// Classic Type B Decompression Tests
// ============================================================================

#[test]
fn test_all_classic_type_b_replays_decompress() {
    for name in CLASSIC_TYPE_B_REPLAYS {
        let data = read_replay(name);
        let header = Header::parse(&data).unwrap_or_else(|e| {
            panic!("Failed to parse header for {}: {}", name, e);
        });

        let result = decompress(&data, &header);

        assert!(
            result.is_ok(),
            "Failed to decompress {}: {}",
            name,
            result.unwrap_err()
        );

        let decompressed = result.unwrap();
        assert!(
            !decompressed.is_empty(),
            "{} should decompress to non-empty data",
            name
        );
    }
}

#[test]
fn test_classic_type_b_replay_100000() {
    let data = read_replay("replay_100000.w3g");
    let header = Header::parse(&data).unwrap();
    let decompressed = decompress(&data, &header).unwrap();

    assert!(!decompressed.is_empty());

    // Verify build version indicates Type B
    if let Header::Classic(h) = &header {
        assert_eq!(h.build_version, 10036);
        assert!(h.is_type_b());
    }
}

// ============================================================================
// All 27 Replays Test
// ============================================================================

#[test]
fn test_all_27_replays_decompress_successfully() {
    let all_replays: Vec<&str> = GRBN_REPLAYS
        .iter()
        .chain(CLASSIC_TYPE_A_REPLAYS.iter())
        .chain(CLASSIC_TYPE_B_REPLAYS.iter())
        .copied()
        .collect();

    assert_eq!(
        all_replays.len(),
        27,
        "Should have exactly 27 test replays"
    );

    let mut success_count = 0;
    let mut failures = Vec::new();

    for name in &all_replays {
        let data = read_replay(name);
        let header = match Header::parse(&data) {
            Ok(h) => h,
            Err(e) => {
                failures.push(format!("{}: header parse failed: {}", name, e));
                continue;
            }
        };

        match decompress(&data, &header) {
            Ok(decompressed) => {
                if decompressed.is_empty() {
                    failures.push(format!("{}: decompressed to empty", name));
                } else {
                    success_count += 1;
                }
            }
            Err(e) => {
                failures.push(format!("{}: decompression failed: {}", name, e));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Failed to decompress {} replays:\n{}",
        failures.len(),
        failures.join("\n")
    );

    assert_eq!(
        success_count, 27,
        "All 27 replays should decompress successfully"
    );
}

// ============================================================================
// Decompression Statistics Test
// ============================================================================

#[test]
fn test_decompression_statistics() {
    let all_replays: Vec<&str> = GRBN_REPLAYS
        .iter()
        .chain(CLASSIC_TYPE_A_REPLAYS.iter())
        .chain(CLASSIC_TYPE_B_REPLAYS.iter())
        .copied()
        .collect();

    let mut total_compressed: u64 = 0;
    let mut total_decompressed: u64 = 0;

    for name in &all_replays {
        let data = read_replay(name);
        let header = Header::parse(&data).unwrap();
        let decompressed = decompress(&data, &header).unwrap();

        total_compressed += data.len() as u64;
        total_decompressed += decompressed.len() as u64;
    }

    // GRBN files have overhead (embedded headers, metadata), so ratio varies
    let ratio = total_decompressed as f64 / total_compressed as f64;

    // Ratio should be positive (decompression produces data)
    assert!(
        ratio > 1.0,
        "Decompression should produce more data than compressed size, got ratio {}",
        ratio
    );
}

// ============================================================================
// Block Count Verification (Classic only)
// ============================================================================

#[test]
fn test_classic_block_iteration_complete() {
    // Verify that block iteration processes all blocks
    let all_classic: Vec<&str> = CLASSIC_TYPE_A_REPLAYS
        .iter()
        .chain(CLASSIC_TYPE_B_REPLAYS.iter())
        .copied()
        .collect();

    for name in all_classic {
        let data = read_replay(name);
        let header = Header::parse(&data).unwrap();

        if let Header::Classic(h) = &header {
            let decompressed = decompress(&data, &header).unwrap();

            // Each block decompresses to approximately 8192 bytes
            // Total should be close to block_count * 8192
            let expected_min = (h.block_count - 1) as usize * 8192;
            let expected_max = (h.block_count + 1) as usize * 8192;

            assert!(
                decompressed.len() >= expected_min && decompressed.len() <= expected_max,
                "{}: decompressed {} bytes, expected between {} and {} for {} blocks",
                name,
                decompressed.len(),
                expected_min,
                expected_max,
                h.block_count
            );
        }
    }
}
