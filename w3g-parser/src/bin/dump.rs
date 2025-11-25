//! Dump tool for extracting decompressed replay data
//!
//! Usage: cargo run --bin dump <replay.w3g> [output.bin]

use std::env;
use std::fs;
use w3g_parser::{decompress, Header};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <replay.w3g> [output.bin]", args[0]);
        eprintln!("  If output.bin is not specified, writes to decompressed.bin");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = if args.len() > 2 {
        args[2].clone()
    } else {
        "decompressed.bin".to_string()
    };

    eprintln!("Reading: {}", input_path);
    let data = fs::read(input_path).expect("Failed to read input file");
    eprintln!("File size: {} bytes", data.len());

    let header = Header::parse(&data).expect("Failed to parse header");
    eprintln!("Format: {:?}", header.format());
    eprintln!("Header data offset: {}", header.data_offset());

    match &header {
        Header::Grbn(h) => {
            eprintln!("GRBN version: {}", h.version);
            eprintln!("Header decompressed_size: {}", h.decompressed_size);
        }
        Header::Classic(h) => {
            eprintln!("Build version: {}", h.build_version);
            eprintln!("Block count: {}", h.block_count);
            eprintln!("Header decompressed_size: {}", h.decompressed_size);
            eprintln!("Duration: {}", h.duration_string());
        }
    }

    eprintln!("Decompressing...");
    let decompressed = decompress(&data, &header).expect("Failed to decompress");

    eprintln!("Decompressed size: {} bytes", decompressed.len());

    fs::write(&output_path, &decompressed).expect("Failed to write output file");
    eprintln!("Wrote to: {}", output_path);
}
