//! Action dump tool for analyzing TimeFrame action data
//!
//! Usage: cargo run --bin action_dump <replay.w3g> [--frames N] [--hex]
//!
//! Options:
//!   --frames N  Number of frames to analyze (default: 50)
//!   --hex       Show hex dump of action data
//!   --small     Only show frames with 1-20 bytes of action data

use std::env;
use std::fs;
use w3g_parser::{decompress, GameRecord, Header};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <replay.w3g> [--frames N] [--hex] [--small]", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let mut max_frames: usize = 50;
    let mut show_hex = false;
    let mut small_only = false;

    // Parse arguments
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--frames" => {
                if i + 1 < args.len() {
                    max_frames = args[i + 1].parse().unwrap_or(50);
                    i += 1;
                }
            }
            "--hex" => show_hex = true,
            "--small" => small_only = true,
            _ => {}
        }
        i += 1;
    }

    eprintln!("Reading: {}", input_path);
    let data = fs::read(input_path).expect("Failed to read input file");

    let header = Header::parse(&data).expect("Failed to parse header");
    eprintln!("Format: {:?}", header.format());

    let decompressed = decompress(&data, &header).expect("Failed to decompress");
    eprintln!("Decompressed: {} bytes", decompressed.len());

    let game_record = GameRecord::parse(&decompressed).expect("Failed to parse game record");
    eprintln!("Host: {}", game_record.host_name());
    eprintln!("Players: {:?}", game_record.player_names());
    eprintln!("TimeFrame offset: 0x{:04X}", game_record.timeframe_offset);
    eprintln!();

    // Analyze TimeFrames
    let mut count = 0;
    let mut total_action_bytes = 0;
    let mut frames_with_actions = 0;

    println!("=== TimeFrame Action Data Analysis ===\n");

    for result in game_record.timeframes(&decompressed) {
        let frame = match result {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error parsing frame {}: {}", count, e);
                break;
            }
        };

        let action_len = frame.action_data.len();
        total_action_bytes += action_len;

        if action_len > 0 {
            frames_with_actions += 1;

            // Skip if small_only and frame is too big
            if small_only && action_len > 20 {
                count += 1;
                if count >= max_frames {
                    break;
                }
                continue;
            }

            println!(
                "Frame {:4}: time={:5}ms (total: {:7}ms), action_len={:3}",
                count, frame.time_delta_ms, frame.accumulated_time_ms, action_len
            );

            if show_hex || action_len <= 30 {
                print_hex_dump(&frame.action_data, "  ");
            }

            // Analyze action structure
            analyze_action_data(&frame.action_data);
            println!();
        }

        count += 1;
        if count >= max_frames {
            break;
        }
    }

    eprintln!("\n=== Summary ===");
    eprintln!("Frames analyzed: {}", count);
    eprintln!("Frames with actions: {}", frames_with_actions);
    eprintln!("Total action bytes: {}", total_action_bytes);
}

fn print_hex_dump(data: &[u8], prefix: &str) {
    for (i, chunk) in data.chunks(16).enumerate() {
        print!("{}{:04X}: ", prefix, i * 16);

        // Hex bytes
        for (j, &byte) in chunk.iter().enumerate() {
            if j == 8 {
                print!(" ");
            }
            print!("{:02X} ", byte);
        }

        // Padding for incomplete rows
        for j in chunk.len()..16 {
            if j == 8 {
                print!(" ");
            }
            print!("   ");
        }

        // ASCII representation
        print!(" |");
        for &byte in chunk {
            if byte >= 0x20 && byte < 0x7F {
                print!("{}", byte as char);
            } else {
                print!(".");
            }
        }
        println!("|");
    }
}

fn analyze_action_data(data: &[u8]) {
    if data.is_empty() {
        return;
    }

    let mut pos = 0;
    let mut findings = Vec::new();

    while pos < data.len() {
        let byte = data[pos];

        match byte {
            // Player ID byte (typically first byte)
            0x01..=0x0F if pos == 0 => {
                findings.push(format!("Player ID: {}", byte));
                pos += 1;
            }

            // Action command marker
            0x1A => {
                if pos + 1 < data.len() {
                    let subcommand = data[pos + 1];
                    if subcommand == 0x19 && pos + 6 <= data.len() {
                        // Ability use: 0x1A 0x19 [4-byte code]
                        let code = &data[pos + 2..pos + 6];
                        let code_str: String = code.iter()
                            .map(|&b| if b >= 0x20 && b < 0x7F { b as char } else { '.' })
                            .collect();
                        let code_rev: String = code.iter().rev()
                            .map(|&b| if b >= 0x20 && b < 0x7F { b as char } else { '.' })
                            .collect();
                        findings.push(format!("Ability: {} (rev: {})", code_str, code_rev));
                        pos += 6;
                    } else {
                        findings.push(format!("Action 0x1A sub=0x{:02X}", subcommand));
                        pos += 2;
                    }
                } else {
                    pos += 1;
                }
            }

            // Selection marker (within actions)
            0x16 => {
                if pos + 2 < data.len() {
                    let count = data[pos + 1];
                    findings.push(format!("Selection: {} units", count));
                    // Skip selection header + unit IDs
                    let skip = 4 + (count as usize * 8).min(data.len() - pos - 4);
                    pos += skip.max(2);
                } else {
                    pos += 1;
                }
            }

            // Potential FourCC detection (4 printable ASCII chars)
            0x41..=0x7A => {
                if pos + 4 <= data.len() {
                    let potential = &data[pos..pos + 4];
                    if potential.iter().all(|&b| b >= 0x20 && b < 0x7F) {
                        let code: String = potential.iter().map(|&b| b as char).collect();
                        let rev: String = potential.iter().rev().map(|&b| b as char).collect();
                        findings.push(format!("FourCC at 0x{:02X}: {} (rev: {})", pos, code, rev));
                    }
                }
                pos += 1;
            }

            _ => {
                pos += 1;
            }
        }
    }

    if !findings.is_empty() {
        println!("  Analysis: {}", findings.join(", "));
    }
}
