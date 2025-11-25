//! Action parsing for W3G replay files.
//!
//! This module provides parsers for player actions within `TimeFrame` records,
//! including unit selection, ability use, movement commands, and control group hotkeys.
//!
//! # Overview
//!
//! Actions represent individual commands issued by players during gameplay.
//! Each action has:
//! - A player ID (1-15) identifying who issued the command
//! - An action type with parsed data specific to that action
//! - A timestamp inherited from the containing `TimeFrame`
//!
//! # Action Types
//!
//! | Type | Subcommand | Description |
//! |------|------------|-------------|
//! | 0x00 | 0x0D | Move/Attack command with coordinates |
//! | 0x0F | 0x00 | Instant ability |
//! | 0x16 | - | Unit selection |
//! | 0x17 | - | Control group hotkey |
//! | 0x1A | 0x00 | Ability with selection |
//! | 0x1A | 0x19 | Direct ability |
//!
//! # Example
//!
//! ```ignore
//! use w3g_parser::records::TimeFrame;
//! use w3g_parser::actions::{ActionIterator, ActionContext};
//!
//! fn process_frame(frame: &TimeFrame) {
//!     let ctx = ActionContext {
//!         timestamp_ms: frame.accumulated_time_ms,
//!         frame_number: 0,
//!     };
//!
//!     for result in ActionIterator::new(&frame.action_data, ctx) {
//!         match result {
//!             Ok(action) => {
//!                 println!("Player {} at {}ms: {:?}",
//!                     action.player_id,
//!                     action.timestamp_ms,
//!                     action.action_type);
//!             }
//!             Err(e) => eprintln!("Parse error: {}", e),
//!         }
//!     }
//! }
//! ```

mod ability;
mod hotkey;
mod movement;
mod parser;
mod selection;
mod types;

pub use ability::{AbilityAction, AbilityCode, AbilityWithSelectionAction, InstantAbilityAction, Race};
pub use hotkey::{HotkeyAction, HotkeyOperation};
pub use movement::{MovementAction, MovementType, Position};
pub use parser::{ActionContext, ActionIterator, ActionStatistics};
pub use selection::{SelectionAction, SelectionMode};
pub use types::{Action, ActionType};
