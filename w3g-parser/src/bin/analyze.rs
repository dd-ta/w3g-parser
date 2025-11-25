//! Binary to analyze decompressed replay data for debugging

use std::env;
use std::fs;
use w3g_parser::decompress::decompress;
use w3g_parser::header::Header;
use w3g_parser::records::{GameRecordHeader, PlayerRoster};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: analyze <replay.w3g>");
        std::process::exit(1);
    }

    let data = fs::read(&args[1]).unwrap();
    println!("File size: {} bytes", data.len());

    // Parse header
    let header = Header::parse(&data).unwrap();
    println!("\n=== Header ===");
    println!("Format: {:?}", header.format());
    println!("Data offset: {}", header.data_offset());
    println!("Decompressed size: {}", header.decompressed_size());

    if let Header::Classic(h) = &header {
        println!("Build version: {}", h.build_version);
    }

    // Decompress
    let decompressed = decompress(&data, &header).unwrap();
    println!("\n=== Decompressed ===");
    println!("Actual decompressed size: {} bytes", decompressed.len());

    // Hex dump first 512 bytes
    println!("\n=== First 512 bytes (hex) ===");
    hex_dump(&decompressed, 0, 512.min(decompressed.len()));

    // Parse game header
    println!("\n=== Game Record Header ===");
    match GameRecordHeader::parse(&decompressed) {
        Ok(header) => {
            println!("Host name: {}", header.host_name);
            println!("Host slot: {}", header.host_slot);
            println!("Additional data: {:?}", header.additional_data);
            println!("Encoded settings length: {} bytes", header.encoded_settings.len());
            println!("Byte length: {}", header.byte_length);

            // Look at what comes after game header
            let player_start = header.byte_length;
            println!("\n=== After Game Header (offset {}) ===", player_start);
            if player_start < decompressed.len() {
                let after_header = &decompressed[player_start..];
                println!("First 20 bytes: {:02X?}", &after_header[..20.min(after_header.len())]);
                println!("First byte marker: 0x{:02X}", after_header[0]);

                // Parse player roster
                println!("\n=== Player Roster ===");
                match PlayerRoster::parse(after_header) {
                    Ok(roster) => {
                        println!("Found {} players", roster.len());
                        println!("Roster byte length: {}", roster.byte_length);
                        for p in roster.players() {
                            println!("  Slot {}: {} ({} bytes)", p.slot_id(), p.player_name(), p.byte_length());
                        }

                        // Look at what comes after player roster
                        let timeframe_start = player_start + roster.byte_length;
                        println!("\n=== After Player Roster (offset {}) ===", timeframe_start);
                        if timeframe_start < decompressed.len() {
                            hex_dump(&decompressed, timeframe_start, 128.min(decompressed.len() - timeframe_start));
                        }
                    }
                    Err(e) => {
                        println!("Failed to parse player roster: {}", e);
                        // Try to manually scan for patterns
                        println!("\n=== Manual scan for markers ===");
                        scan_for_markers(after_header, player_start);
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to parse game header: {}", e);
        }
    }

    // Scan for record markers throughout the file
    println!("\n=== Record Marker Distribution ===");
    let mut marker_counts: std::collections::HashMap<u8, usize> = std::collections::HashMap::new();
    for &byte in &decompressed {
        match byte {
            0x16 | 0x19 | 0x1E | 0x1F | 0x20 | 0x22 | 0x17 => {
                *marker_counts.entry(byte).or_insert(0) += 1;
            }
            _ => {}
        }
    }
    for (marker, count) in &marker_counts {
        let name = match *marker {
            0x16 => "PlayerSlot",
            0x19 => "SlotRecord",
            0x1E => "TimeFrame1E",
            0x1F => "TimeFrame1F",
            0x20 => "Chat",
            0x22 => "Checksum",
            0x17 => "Leave",
            _ => "Unknown",
        };
        println!("  0x{:02X} ({}): {} occurrences", marker, name, count);
    }

    // Find first TimeFrame AFTER player roster
    println!("\n=== TimeFrame Search (after player roster) ===");
    let search_start = match GameRecordHeader::parse(&decompressed) {
        Ok(h) => {
            match PlayerRoster::parse(&decompressed[h.byte_length..]) {
                Ok(r) => h.byte_length + r.byte_length,
                Err(_) => h.byte_length,
            }
        }
        Err(_) => 0,
    };
    println!("Searching from offset {}", search_start);

    // Look for FIRST TimeFrame marker after player roster
    println!("\n=== First TimeFrame after player roster ===");
    let mut first_tf_offset = None;
    for i in search_start..decompressed.len() {
        let byte = decompressed[i];
        if byte == 0x1F || byte == 0x1E {
            // Verify this looks like a real TimeFrame
            if i + 5 <= decompressed.len() {
                let time_delta = u16::from_le_bytes([decompressed[i+1], decompressed[i+2]]);
                let length_hint = u16::from_le_bytes([decompressed[i+3], decompressed[i+4]]);
                // TimeFrames should have reasonable time deltas (< 5000ms typically)
                // and length hints (< 8000 bytes typically)
                if time_delta < 5000 && length_hint < 8000 {
                    first_tf_offset = Some(i);
                    println!("Found TimeFrame at offset {}", i);
                    println!("  Time delta: {}ms, Length hint: {}", time_delta, length_hint);
                    hex_dump(&decompressed, i, 64.min(decompressed.len() - i));
                    break;
                }
            }
        }
    }

    // Show what's between player roster and TimeFrame
    if let Some(tf_off) = first_tf_offset {
        println!("\n=== Data between player roster ({}) and TimeFrame ({}) ===", search_start, tf_off);
        println!("Length: {} bytes", tf_off - search_start);
        hex_dump(&decompressed, search_start, (tf_off - search_start).min(256));
    }

    // Test TimeFrameIterator with detailed debugging
    println!("\n=== Testing TimeFrameIterator ===");
    use w3g_parser::records::TimeFrameIterator;
    let mut iter = TimeFrameIterator::new(&decompressed, search_start);
    let mut frame_count = 0;
    let mut last_error: Option<String> = None;
    let mut last_offset = search_start;

    loop {
        let current_offset = iter.current_offset();
        match iter.next() {
            Some(Ok(frame)) => {
                frame_count += 1;
                if frame_count <= 5 || frame_count % 10000 == 0 {
                    println!("Frame {}: offset={}, delta={}ms, actions={} bytes",
                             frame_count, last_offset, frame.time_delta_ms, frame.action_data.len());
                }
                last_offset = current_offset;
            }
            Some(Err(e)) => {
                last_error = Some(format!("{}", e));
                println!("Error at offset {}: {}", current_offset, e);
                break;
            }
            None => {
                println!("Iterator finished at offset {}", current_offset);
                // Show bytes around stop point
                if current_offset < decompressed.len() {
                    println!("Bytes at stop point:");
                    hex_dump(&decompressed, current_offset, 32.min(decompressed.len() - current_offset));
                }
                break;
            }
        }
        if frame_count > 100000 {
            println!("... (stopped at 100000 frames)");
            break;
        }
    }
    println!("Total frames parsed: {}", frame_count);
    if let Some(e) = last_error {
        println!("Stopped due to error: {}", e);
    }
}

fn hex_dump(data: &[u8], offset: usize, len: usize) {
    let end = (offset + len).min(data.len());
    for row_start in (offset..end).step_by(16) {
        let row_end = (row_start + 16).min(end);
        let row = &data[row_start..row_end];
        print!("{:08X}: ", row_start);
        for (i, byte) in row.iter().enumerate() {
            print!("{:02X} ", byte);
            if i == 7 {
                print!(" ");
            }
        }
        // Padding for incomplete rows
        for i in row.len()..16 {
            print!("   ");
            if i == 7 {
                print!(" ");
            }
        }
        print!(" |");
        for byte in row {
            if *byte >= 0x20 && *byte < 0x7F {
                print!("{}", *byte as char);
            } else {
                print!(".");
            }
        }
        println!("|");
    }
}

fn scan_for_markers(data: &[u8], base_offset: usize) {
    println!("Scanning first 500 bytes after game header...");
    for (i, window) in data.windows(3).enumerate().take(500) {
        match window[0] {
            0x16 => {
                let slot = window[1];
                let next = window[2];
                if slot <= 24 && ((0x20..=0x7E).contains(&next) || next >= 0x80) {
                    println!("  Possible PlayerSlot at offset {} (slot {})", base_offset + i, slot);
                }
            }
            0x19 => {
                println!("  Possible SlotRecord at offset {}", base_offset + i);
            }
            0x1F | 0x1E => {
                println!("  TimeFrame marker at offset {}", base_offset + i);
            }
            0x20 => {
                println!("  Chat marker at offset {}", base_offset + i);
            }
            0x22 => {
                println!("  Checksum marker at offset {}", base_offset + i);
            }
            _ => {}
        }
    }
}
