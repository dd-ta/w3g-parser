//! Debug tool to analyze chat message structures in W3G replays
//! This tool hex dumps chat messages to reverse engineer the player_id field

use std::env;
use std::fs;
use std::path::Path;
use w3g_parser::decompress::decompress;
use w3g_parser::header::Header;
use w3g_parser::records::{GameRecord, ChatMessage, CHAT_MARKER};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <replay.w3g>", args[0]);
        std::process::exit(1);
    }

    let replay_path = &args[1];
    let data = fs::read(replay_path).expect("Failed to read replay file");

    // Parse header
    let header = Header::parse(&data).expect("Failed to parse header");
    println!("Replay: {}", Path::new(replay_path).file_name().unwrap().to_string_lossy());
    println!();

    // Decompress
    let decompressed = decompress(&data, &header).expect("Failed to decompress");

    // Parse game record to get player roster
    let game_record = GameRecord::parse(&decompressed).expect("Failed to parse game record");

    println!("=== PLAYER ROSTER ===");
    for (i, player) in game_record.players.players().enumerate() {
        println!("  [{}] Slot {:2}: {}", i, player.slot_id(), player.player_name());
    }
    println!();

    // Find chat messages using proper parser
    println!("=== CHAT MESSAGES ===");
    let mut offset = game_record.timeframe_offset;
    let mut chat_count = 0;

    while offset < decompressed.len() {
        if decompressed[offset] == CHAT_MARKER {
            // Try to parse as chat message
            if let Ok(chat) = ChatMessage::parse(&decompressed[offset..]) {
                chat_count += 1;

                let msg_bytes = &decompressed[offset..offset + chat.byte_length];

                println!("Chat Message #{} at offset 0x{:X}:", chat_count, offset);

                // Hex dump
                let dump_len = msg_bytes.len().min(60);
                print!("  Raw: ");
                for i in 0..dump_len {
                    print!("{:02X} ", msg_bytes[i]);
                    if (i + 1) % 20 == 0 {
                        print!("\n       ");
                    }
                }
                println!();

                // Parse structure
                if msg_bytes.len() >= 9 {
                    let flags = msg_bytes[1];
                    let message_id = u16::from_le_bytes([msg_bytes[2], msg_bytes[3]]);
                    let padding_bytes = &msg_bytes[4..9];

                    println!("  Flags: 0x{:02X} {}", flags,
                        if flags == 0x03 { "(system)" }
                        else if flags >= 0x07 { "(player)" }
                        else { "(unknown)" });
                    println!("  Message ID: 0x{:04X} ({})", message_id, message_id);
                    print!("  Padding: ");
                    for (i, &b) in padding_bytes.iter().enumerate() {
                        print!("{:02X} ", b);
                        if i == 1 {
                            print!("<-- SUSPECTED PLAYER ID ");
                        }
                    }
                    println!();
                    println!("  Text: \"{}\"", chat.message);

                    // Highlight byte 5 (offset 5 from message start)
                    println!("  >>> BYTE at offset 5 (padding[1]): 0x{:02X} ({}) <<<",
                        msg_bytes[5], msg_bytes[5]);
                }

                println!();
                offset += chat.byte_length;
            } else {
                offset += 1;
            }
        } else {
            offset += 1;
        }
    }

    println!("Total chat messages found: {}", chat_count);
}
