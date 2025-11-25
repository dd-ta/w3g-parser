//! Deep action data analysis tool
//!
//! This tool performs detailed analysis of action data structure within TimeFrames

use std::collections::HashMap;
use std::env;
use std::fs;
use w3g_parser::{decompress, GameRecord, Header};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <replay.w3g>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    eprintln!("Reading: {}", input_path);

    let data = fs::read(input_path).expect("Failed to read input file");
    let header = Header::parse(&data).expect("Failed to parse header");
    let decompressed = decompress(&data, &header).expect("Failed to decompress");
    let game_record = GameRecord::parse(&decompressed).expect("Failed to parse game record");

    eprintln!("Host: {}", game_record.host_name());
    eprintln!("Players: {:?}", game_record.player_names());
    eprintln!();

    let mut all_action_bytes: Vec<u8> = Vec::new();
    let mut ability_codes: HashMap<String, u32> = HashMap::new();
    let mut action_types: HashMap<u8, u32> = HashMap::new();
    let mut player_actions: HashMap<u8, u32> = HashMap::new();
    let mut action_block_sizes: Vec<usize> = Vec::new();
    let mut frame_count = 0;
    let mut frames_with_actions = 0;

    // Collect all action data
    for result in game_record.timeframes(&decompressed) {
        let frame = match result {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error at frame {}: {}", frame_count, e);
                break;
            }
        };

        frame_count += 1;

        if frame.action_data.is_empty() {
            continue;
        }

        frames_with_actions += 1;
        action_block_sizes.push(frame.action_data.len());
        all_action_bytes.extend_from_slice(&frame.action_data);

        // Analyze this frame
        analyze_frame_actions(&frame.action_data, &mut ability_codes, &mut action_types, &mut player_actions);
    }

    println!("=== Action Data Analysis Report ===\n");

    println!("## Summary Statistics\n");
    println!("Total frames: {}", frame_count);
    println!("Frames with actions: {}", frames_with_actions);
    println!("Total action bytes: {}", all_action_bytes.len());
    if !action_block_sizes.is_empty() {
        let avg_size: f64 = action_block_sizes.iter().sum::<usize>() as f64 / action_block_sizes.len() as f64;
        println!("Average action block size: {:.1} bytes", avg_size);
        println!("Max action block size: {} bytes", action_block_sizes.iter().max().unwrap_or(&0));
        println!("Min action block size (>0): {} bytes", action_block_sizes.iter().filter(|&&x| x > 0).min().unwrap_or(&0));
    }
    println!();

    // Player action distribution
    println!("## Player Action Distribution\n");
    let mut player_vec: Vec<_> = player_actions.iter().collect();
    player_vec.sort_by_key(|&(id, _)| *id);
    for (player_id, count) in player_vec {
        println!("Player {:2}: {:5} action blocks", player_id, count);
    }
    println!();

    // Action type distribution
    println!("## Action Type Distribution\n");
    let mut type_vec: Vec<_> = action_types.iter().collect();
    type_vec.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));
    for (action_type, count) in type_vec.iter().take(20) {
        let desc = describe_action_type(**action_type);
        println!("0x{:02X}: {:5} occurrences - {}", action_type, count, desc);
    }
    println!();

    // Ability codes
    println!("## Ability Codes (FourCC)\n");
    let mut ability_vec: Vec<_> = ability_codes.iter().collect();
    ability_vec.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));
    println!("| Stored | Reversed | Count | Likely Meaning |");
    println!("|--------|----------|-------|----------------|");
    for (code, count) in ability_vec.iter().take(30) {
        let reversed: String = code.chars().rev().collect();
        let meaning = guess_ability_meaning(code);
        println!("| {} | {} | {:5} | {} |", code, reversed, count, meaning);
    }
    println!();

    // Byte frequency analysis
    println!("## First Byte Distribution (Action Blocks)\n");
    let mut first_bytes: HashMap<u8, u32> = HashMap::new();
    for frame_result in game_record.timeframes(&decompressed) {
        if let Ok(frame) = frame_result {
            if !frame.action_data.is_empty() {
                *first_bytes.entry(frame.action_data[0]).or_insert(0) += 1;
            }
        }
    }
    let mut fb_vec: Vec<_> = first_bytes.iter().collect();
    fb_vec.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));
    for (byte, count) in fb_vec.iter().take(15) {
        println!("0x{:02X}: {:5} ({:.1}%)", byte, count, (**count as f64 / frames_with_actions as f64) * 100.0);
    }
    println!();

    // Look for coordinate patterns (IEEE 754 floats)
    println!("## Coordinate Patterns\n");
    find_coordinates(&all_action_bytes);
}

fn analyze_frame_actions(
    data: &[u8],
    ability_codes: &mut HashMap<String, u32>,
    action_types: &mut HashMap<u8, u32>,
    player_actions: &mut HashMap<u8, u32>,
) {
    if data.is_empty() {
        return;
    }

    // First byte is often player ID
    let first_byte = data[0];
    if first_byte >= 0x01 && first_byte <= 0x0F {
        *player_actions.entry(first_byte).or_insert(0) += 1;
    }

    // Scan for patterns
    let mut pos = 0;
    while pos < data.len() {
        let byte = data[pos];

        // Track all bytes that could be action type markers
        if byte == 0x1A || byte == 0x16 || byte == 0x17 || byte == 0x19 || byte == 0x10 || byte == 0x14 {
            *action_types.entry(byte).or_insert(0) += 1;
        }

        // Look for ability codes: 0x1A 0x19 [4-byte FourCC]
        if byte == 0x1A && pos + 1 < data.len() && data[pos + 1] == 0x19 && pos + 6 <= data.len() {
            let code = &data[pos + 2..pos + 6];
            if code.iter().all(|&b| b >= 0x20 && b < 0x7F) {
                let code_str: String = code.iter().map(|&b| b as char).collect();
                *ability_codes.entry(code_str).or_insert(0) += 1;
            }
        }

        // Also look for standalone FourCC codes that might be ability references
        if pos + 4 <= data.len() {
            let potential = &data[pos..pos + 4];
            // Look for typical ability code patterns (lowercase + uppercase mix)
            if is_likely_ability_code(potential) {
                let code_str: String = potential.iter().map(|&b| b as char).collect();
                // Only count if not already counted via 0x1A 0x19 pattern
                if pos < 2 || data[pos - 2] != 0x1A || data[pos - 1] != 0x19 {
                    // Don't double count
                }
            }
        }

        pos += 1;
    }
}

fn is_likely_ability_code(bytes: &[u8]) -> bool {
    if bytes.len() != 4 {
        return false;
    }

    // Must be printable ASCII
    if !bytes.iter().all(|&b| b >= 0x20 && b < 0x7F) {
        return false;
    }

    // Typical ability codes have patterns like:
    // - Start with uppercase (A-Z) or lowercase (a-z)
    // - Often have specific patterns like "Axxx" (hero abilities) or "xxxx" (unit abilities)
    let first = bytes[0];
    let has_uppercase = bytes.iter().any(|&b| b >= b'A' && b <= b'Z');
    let has_lowercase = bytes.iter().any(|&b| b >= b'a' && b <= b'z');

    (first >= b'A' && first <= b'Z') || (first >= b'a' && first <= b'z' && has_uppercase)
}

fn describe_action_type(action: u8) -> &'static str {
    match action {
        0x10 => "Pause/Resume?",
        0x14 => "Unknown (0x14)",
        0x16 => "Unit Selection",
        0x17 => "Group Assignment?",
        0x19 => "Ability Subcommand",
        0x1A => "Action Command",
        0x1B => "Unknown (0x1B)",
        0x1C => "Unknown (0x1C)",
        _ => "Unknown",
    }
}

fn guess_ability_meaning(code: &str) -> &'static str {
    let reversed: String = code.chars().rev().collect();

    // Match on reversed code (which is likely the "real" code)
    match reversed.as_str() {
        // Human abilities
        "AHbz" => "Blizzard (Archmage)",
        "AHwe" => "Water Elemental (Archmage)",
        "AHab" => "Brilliance Aura (Archmage)",
        "AHmt" => "Mass Teleport (Archmage)",
        "AHtb" => "Thunderbolt (Mountain King)",
        "AHtc" => "Thunder Clap (Mountain King)",
        "AHbh" => "Bash (Mountain King)",
        "AHav" => "Avatar (Mountain King)",
        "AHhb" => "Holy Light (Paladin)",
        "AHds" => "Divine Shield (Paladin)",
        "AHad" => "Devotion Aura (Paladin)",
        "AHre" => "Resurrection (Paladin)",

        // Orc abilities
        "AOcl" => "Chain Lightning (Far Seer)",
        "AOww" => "Feral Spirit (Far Seer)",
        "AOsf" => "Far Sight (Far Seer)",
        "AOeq" => "Earthquake (Far Seer)",
        "AOmi" => "Mirror Image (Blademaster)",
        "AOwk" => "Wind Walk (Blademaster)",
        "AOcr" => "Critical Strike (Blademaster)",
        "AObf" => "Bladestorm (Blademaster)",
        "AOsh" => "Shockwave (Tauren Chieftain)",
        "AOae" => "Endurance Aura (Tauren Chieftain)",
        "AOws" => "War Stomp (Tauren Chieftain)",
        "AOre" => "Reincarnation (Tauren Chieftain)",

        // Undead abilities
        "AUdc" => "Death Coil (Death Knight)",
        "AUdp" => "Death Pact (Death Knight)",
        "AUau" => "Unholy Aura (Death Knight)",
        "AUan" => "Animate Dead (Death Knight)",
        "AUcs" => "Carrion Swarm (Dreadlord)",
        "AUsl" => "Sleep (Dreadlord)",
        "AUav" => "Vampiric Aura (Dreadlord)",
        "AUin" => "Inferno (Dreadlord)",
        "AUfn" => "Frost Nova (Lich)",
        "AUfa" => "Frost Armor (Lich)",
        "AUdr" => "Dark Ritual (Lich)",
        "AUdd" => "Death and Decay (Lich)",

        // Night Elf abilities
        "AEer" => "Entangling Roots (Keeper)",
        "AEfn" => "Force of Nature (Keeper)",
        "AEah" => "Thorns Aura (Keeper)",
        "AEtq" => "Tranquility (Keeper)",
        "AEst" => "Scout (Priestess)",
        "AEsn" => "Searing Arrows (Priestess)",
        "AEtr" => "Trueshot Aura (Priestess)",
        "AEsf" => "Starfall (Priestess)",
        "AEmb" => "Mana Burn (Demon Hunter)",
        "AEim" => "Immolation (Demon Hunter)",
        "AEev" => "Evasion (Demon Hunter)",
        "AEme" => "Metamorphosis (Demon Hunter)",
        "AEbl" => "Blink (Warden)",
        "AEsh" => "Shadow Strike (Warden)",
        "AEfk" => "Fan of Knives (Warden)",
        "AEsv" => "Spirit of Vengeance (Warden)",

        // Common unit abilities
        "etol" => "Entangle Tree (Ancient)",
        "ewsp" => "Wisp Detonate",
        "eaom" => "Eat Tree (Ancient)",
        "uaco" => "Acolyte Sacrifice",
        "aeph" => "Phase Shift (Faerie Dragon)",

        // Buildings/Training
        "htow" => "Town Hall",
        "hkee" => "Keep",
        "hcas" => "Castle",
        "halt" => "Altar of Kings",
        "hbar" => "Barracks",
        "hbla" => "Blacksmith",

        _ => "",
    }
}

fn find_coordinates(data: &[u8]) {
    // Look for IEEE 754 single-precision floats that look like map coordinates
    // Typical WC3 map coordinates range from -10000 to 10000

    let mut coord_candidates = 0;

    for i in 0..data.len().saturating_sub(4) {
        let bytes = [data[i], data[i+1], data[i+2], data[i+3]];
        let value = f32::from_le_bytes(bytes);

        // Check if it's a reasonable map coordinate
        if value.is_finite() && value.abs() >= 100.0 && value.abs() <= 20000.0 {
            // Check if next 4 bytes are also a coordinate
            if i + 8 <= data.len() {
                let next_bytes = [data[i+4], data[i+5], data[i+6], data[i+7]];
                let next_value = f32::from_le_bytes(next_bytes);

                if next_value.is_finite() && next_value.abs() >= 100.0 && next_value.abs() <= 20000.0 {
                    coord_candidates += 1;

                    if coord_candidates <= 10 {
                        println!("Coordinate pair at 0x{:04X}: ({:.1}, {:.1})", i, value, next_value);
                    }
                }
            }
        }
    }

    println!("\nTotal coordinate pairs found: {}", coord_candidates);
}
