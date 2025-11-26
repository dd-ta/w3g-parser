//! # W3G Parser
//!
//! A comprehensive Warcraft 3 replay (.w3g) parser library.
//!
//! This library provides tools for parsing Warcraft 3 replay files across
//! all format versions:
//! - **Classic** (RoC/TFT pre-1.32) with block-based compression
//! - **Reforged** (1.32+) with the GRBN header format
//!
//! ## Quick Start
//!
//! ```no_run
//! use w3g_parser::header::Header;
//! use w3g_parser::error::Result;
//!
//! fn parse_replay(data: &[u8]) -> Result<()> {
//!     // Automatically detect format and parse header
//!     let header = Header::parse(data)?;
//!
//!     println!("Format: {:?}", header.format());
//!     println!("Data offset: {}", header.data_offset());
//!     println!("Decompressed size: {} bytes", header.decompressed_size());
//!
//!     // Access format-specific fields
//!     match &header {
//!         w3g_parser::header::Header::Grbn(h) => {
//!             println!("GRBN version: {}", h.version);
//!         }
//!         w3g_parser::header::Header::Classic(h) => {
//!             println!("Build version: {}", h.build_version);
//!             println!("Game duration: {}", h.duration_string());
//!         }
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Module Overview
//!
//! - [`error`] - Error types and result alias for parser operations
//! - [`binary`] - Low-level binary reading utilities for little-endian data
//! - [`format`] - Format detection and type definitions
//! - [`header`] - Header parsing for GRBN and Classic formats
//! - [`decompress`] - Decompression for GRBN and Classic formats
//! - [`records`] - Decompressed data record parsing (game header, players, timeframes)
//!
//! ## Format Reference
//!
//! The W3G format documentation is maintained in `FORMAT.md` alongside this
//! library. Key characteristics:
//!
//! - **GRBN Format**: 128-byte header, single zlib stream
//! - **Classic Format**: 68-byte header, block-based zlib compression
//!   - Type A (build < 10000): 8-byte block headers
//!   - Type B (build >= 10000): 12-byte block headers
//!
//! All multi-byte integers are stored in little-endian byte order.

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod actions;
pub mod binary;
pub mod decompress;
pub mod error;
pub mod format;
pub mod header;
pub mod records;

// Re-export commonly used types at the crate root
pub use actions::{
    AbilityAction, AbilityCode, Action, ActionContext, ActionIterator, ActionStatistics,
    ActionType, HotkeyAction, HotkeyOperation, MovementAction, Position, SelectionAction,
    SelectionMode,
};
pub use decompress::decompress;
pub use error::{ParserError, Result};
pub use format::{detect_format, ClassicVersion, ReplayFormat};
pub use header::Header;
pub use records::{
    ChatMessage, GameRecord, GameRecordHeader, PlayerRoster, TimeFrame, TimeFrameIterator,
    CHAT_MARKER,
};
