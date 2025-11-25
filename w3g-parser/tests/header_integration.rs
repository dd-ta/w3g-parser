//! Integration tests for header parsing against real replay files.
//!
//! These tests verify that the header parsers correctly handle all 27 test
//! replay files from the `/replays/` directory.

use std::fs;
use std::path::Path;

use w3g_parser::format::{ClassicVersion, ReplayFormat};
use w3g_parser::header::{ClassicHeader, GrbnHeader, Header};

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
// GRBN Format Tests
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

#[test]
fn test_all_grbn_replays_parse() {
    for name in GRBN_REPLAYS {
        let data = read_replay(name);
        let header = Header::parse(&data).unwrap_or_else(|e| {
            panic!("Failed to parse {}: {}", name, e);
        });

        assert!(
            header.is_grbn(),
            "{} should be GRBN format, got {:?}",
            name,
            header.format()
        );
        assert_eq!(header.data_offset(), 0x80, "{} data offset", name);
        assert!(
            header.decompressed_size() > 0,
            "{} decompressed_size should be > 0",
            name
        );
    }
}

#[test]
fn test_grbn_header_replay_1() {
    // Detailed test for replay_1.w3g based on FORMAT.md
    let data = read_replay("replay_1.w3g");
    let header = Header::parse(&data).unwrap();

    assert!(matches!(header, Header::Grbn(_)));
    assert_eq!(header.data_offset(), 0x80);

    if let Header::Grbn(h) = header {
        assert_eq!(&h.magic, b"GRBN");
        assert_eq!(h.version, 2);
        assert_eq!(h.unknown_1, 11);
        assert_eq!(h.unknown_2, 51200);
        // Decompressed size from FORMAT.md: 0x0012D2A7 = 1,233,575 bytes
        assert_eq!(h.decompressed_size, 1_233_575);
    }
}

#[test]
fn test_grbn_header_replay_2() {
    let data = read_replay("replay_2.w3g");
    let header = Header::parse(&data).unwrap();

    if let Header::Grbn(h) = header {
        assert_eq!(h.version, 2);
        // Decompressed size from FORMAT.md: 0x00174F94 = 1,527,700 bytes
        assert_eq!(h.decompressed_size, 1_527_700);
    }
}

#[test]
fn test_grbn_header_replay_1000() {
    let data = read_replay("replay_1000.w3g");
    let header = Header::parse(&data).unwrap();

    if let Header::Grbn(h) = header {
        assert_eq!(h.version, 2);
        // From FORMAT.md: Unknown_3=2, Unknown_4=1
        assert_eq!(h.unknown_3, 2);
        assert_eq!(h.unknown_4, 1);
        // Decompressed size: 0x001155B2 = 1,136,050 bytes
        assert_eq!(h.decompressed_size, 1_136_050);
    }
}

#[test]
fn test_grbn_zlib_marker_present() {
    // Verify that zlib data starts at offset 0x80 with a valid marker
    for name in GRBN_REPLAYS {
        let data = read_replay(name);
        let header = Header::parse(&data).unwrap();
        let offset = header.data_offset();

        // Check for zlib magic (0x78 0x9C for default, 0x78 0x01 for fast)
        assert!(
            data.len() > offset + 2,
            "{} too short for zlib marker",
            name
        );

        let zlib_marker = &data[offset..offset + 2];
        assert!(
            (zlib_marker[0] == 0x78 && (zlib_marker[1] == 0x9C || zlib_marker[1] == 0x01)),
            "{} does not have valid zlib marker at 0x80: {:02X} {:02X}",
            name,
            zlib_marker[0],
            zlib_marker[1]
        );
    }
}

// ============================================================================
// Classic Format Tests
// ============================================================================

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

#[test]
fn test_all_classic_type_a_replays_parse() {
    for name in CLASSIC_TYPE_A_REPLAYS {
        let data = read_replay(name);
        let header = Header::parse(&data).unwrap_or_else(|e| {
            panic!("Failed to parse {}: {}", name, e);
        });

        assert!(
            header.is_classic(),
            "{} should be Classic format, got {:?}",
            name,
            header.format()
        );
        assert_eq!(header.data_offset(), 0x44, "{} data offset", name);
        assert_eq!(
            header.classic_version(),
            Some(ClassicVersion::TypeA),
            "{} should be Type A",
            name
        );

        if let Header::Classic(h) = &header {
            assert_eq!(h.build_version, 26, "{} build version", name);
            assert_eq!(h.block_header_size(), 8, "{} block header size", name);
        }
    }
}

#[test]
fn test_all_classic_type_b_replays_parse() {
    for name in CLASSIC_TYPE_B_REPLAYS {
        let data = read_replay(name);
        let header = Header::parse(&data).unwrap_or_else(|e| {
            panic!("Failed to parse {}: {}", name, e);
        });

        assert!(
            header.is_classic(),
            "{} should be Classic format, got {:?}",
            name,
            header.format()
        );
        assert_eq!(header.data_offset(), 0x44, "{} data offset", name);
        assert_eq!(
            header.classic_version(),
            Some(ClassicVersion::TypeB),
            "{} should be Type B",
            name
        );

        if let Header::Classic(h) = &header {
            assert!(
                h.build_version >= 10000,
                "{} build version {} should be >= 10000",
                name,
                h.build_version
            );
            assert_eq!(h.block_header_size(), 12, "{} block header size", name);
        }
    }
}

#[test]
fn test_classic_header_replay_5000() {
    // Detailed test based on FORMAT.md
    let data = read_replay("replay_5000.w3g");
    let file_size = data.len();

    let header = Header::parse(&data).unwrap();

    if let Header::Classic(h) = header {
        assert_eq!(&h.magic[..26], b"Warcraft III recorded game");
        assert_eq!(h.magic[26], 0x1A);
        assert_eq!(h.magic[27], 0x00);
        assert_eq!(h.header_size, 68);
        assert_eq!(
            h.file_size as usize, file_size,
            "File size field should match actual file size"
        );
        assert_eq!(h.header_version, 1);
        assert_eq!(&h.sub_header_magic, b"PX3W");
        assert_eq!(h.build_version, 26);
        // Duration from FORMAT.md: 0x0009ED68 = 650,600 ms (about 10.8 minutes)
        assert_eq!(h.duration_ms, 650_600);
        assert_eq!(h.duration_string(), "00:10:50");
    }
}

#[test]
fn test_classic_header_replay_5001() {
    let data = read_replay("replay_5001.w3g");
    let file_size = data.len();

    let header = Header::parse(&data).unwrap();

    if let Header::Classic(h) = header {
        assert_eq!(h.file_size as usize, file_size);
        assert_eq!(h.build_version, 26);
        // Duration: 0x00101530 = 1,054,000 ms (about 17.6 minutes)
        assert_eq!(h.duration_ms, 1_054_000);
    }
}

#[test]
fn test_classic_header_replay_100000() {
    // Type B replay
    let data = read_replay("replay_100000.w3g");
    let file_size = data.len();

    let header = Header::parse(&data).unwrap();

    if let Header::Classic(h) = header {
        assert_eq!(h.file_size as usize, file_size);
        assert_eq!(h.build_version, 10036);
        // Duration from FORMAT.md: 0x0013FE2A = 1,310,250 ms (about 21.8 minutes)
        assert_eq!(h.duration_ms, 1_310_250);
        assert!(h.is_type_b());
    }
}

#[test]
fn test_classic_file_size_matches_actual() {
    // Verify file_size field matches actual for all Classic replays
    let all_classic: Vec<&str> = CLASSIC_TYPE_A_REPLAYS
        .iter()
        .chain(CLASSIC_TYPE_B_REPLAYS.iter())
        .copied()
        .collect();

    for name in all_classic {
        let data = read_replay(name);
        let actual_size = data.len();

        let header = Header::parse(&data).unwrap();

        if let Header::Classic(h) = header {
            assert_eq!(
                h.file_size as usize, actual_size,
                "{}: file_size field ({}) should match actual size ({})",
                name, h.file_size, actual_size
            );
        }
    }
}

#[test]
fn test_classic_block_zlib_marker_present() {
    // Verify that the first block has a valid zlib marker
    let all_classic: Vec<&str> = CLASSIC_TYPE_A_REPLAYS
        .iter()
        .chain(CLASSIC_TYPE_B_REPLAYS.iter())
        .copied()
        .collect();

    for name in all_classic {
        let data = read_replay(name);
        let header = Header::parse(&data).unwrap();

        if let Header::Classic(h) = &header {
            let block_header_size = h.block_header_size();
            let first_block_data_offset = 0x44 + block_header_size;

            // Check for zlib magic
            assert!(
                data.len() > first_block_data_offset + 2,
                "{} too short",
                name
            );

            let zlib_marker = &data[first_block_data_offset..first_block_data_offset + 2];
            assert!(
                zlib_marker[0] == 0x78,
                "{} first block does not have zlib marker: {:02X} {:02X}",
                name,
                zlib_marker[0],
                zlib_marker[1]
            );
        }
    }
}

// ============================================================================
// Format Detection Tests
// ============================================================================

#[test]
fn test_format_detection_all_replays() {
    // Test that format detection works correctly for all files
    let all_replays: Vec<(&str, ReplayFormat)> = GRBN_REPLAYS
        .iter()
        .map(|&n| (n, ReplayFormat::Grbn))
        .chain(
            CLASSIC_TYPE_A_REPLAYS
                .iter()
                .map(|&n| (n, ReplayFormat::Classic)),
        )
        .chain(
            CLASSIC_TYPE_B_REPLAYS
                .iter()
                .map(|&n| (n, ReplayFormat::Classic)),
        )
        .collect();

    for (name, expected_format) in all_replays {
        let data = read_replay(name);
        let detected = w3g_parser::detect_format(&data).unwrap();

        assert_eq!(
            detected, expected_format,
            "{}: detected {:?}, expected {:?}",
            name, detected, expected_format
        );
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_invalid_magic_error() {
    let invalid_data = b"This is not a valid W3G replay file!";
    let result = Header::parse(invalid_data);

    assert!(
        matches!(result, Err(w3g_parser::ParserError::InvalidMagic { .. })),
        "Expected InvalidMagic error"
    );
}

#[test]
fn test_truncated_file_error() {
    // File too short for GRBN header (needs 128 bytes)
    let short_grbn = b"GRBN\x02\x00\x00\x00"; // Only 8 bytes
    let result = GrbnHeader::parse(short_grbn);

    assert!(
        matches!(result, Err(w3g_parser::ParserError::UnexpectedEof { .. })),
        "Expected UnexpectedEof error for truncated GRBN"
    );

    // File too short for Classic header (needs 68 bytes)
    let short_classic = b"Warcraft III recorded game\x1A\x00\x44\x00\x00\x00"; // Only 32 bytes
    let result = ClassicHeader::parse(short_classic);

    assert!(
        matches!(result, Err(w3g_parser::ParserError::UnexpectedEof { .. })),
        "Expected UnexpectedEof error for truncated Classic"
    );
}

#[test]
fn test_empty_file_error() {
    let empty: &[u8] = &[];
    let result = Header::parse(empty);

    assert!(
        matches!(result, Err(w3g_parser::ParserError::UnexpectedEof { .. })),
        "Expected UnexpectedEof error for empty file"
    );
}

// ============================================================================
// Summary Test
// ============================================================================

#[test]
fn test_all_27_replays_parse_successfully() {
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
        match Header::parse(&data) {
            Ok(_) => success_count += 1,
            Err(e) => failures.push(format!("{}: {}", name, e)),
        }
    }

    assert!(
        failures.is_empty(),
        "Failed to parse {} replays:\n{}",
        failures.len(),
        failures.join("\n")
    );

    assert_eq!(success_count, 27, "All 27 replays should parse successfully");
}
