# Chat Message Parsing Implementation

## Date: 2025-11-25

## Summary

Implemented chat message parsing for the W3G CLI with reverse-engineered format support.

## Problem

The initial implementation had many false positives because the chat marker `0x20` is also the ASCII space character. Every space in binary data was being detected as a potential chat message.

## Solution

Reverse engineered the actual chat message format by:
1. Dumping decompressed replay data
2. Finding known chat strings (e.g., "left the game voluntarily")
3. Analyzing bytes around those strings to identify the structure

### Chat Message Format

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0 | 1 | marker | Always 0x20 |
| 1 | 1 | flags | Message type (0x03=system, 0x07+=player) |
| 2 | 2 | message_id | Message identifier (u16 LE) |
| 4 | 5 | padding | 0x20 XX 0x00 0x00 0x00 |
| 9 | var | message | Null-terminated string |

### Validation Heuristics

To filter false positives, we check:
1. `flags` byte is in range 0x00-0x1F
2. Padding signature at offset 4-8 matches expected pattern
3. Message is at least 2 characters
4. Message contains only printable ASCII

## Files Modified

- `src/records/timeframe.rs` - Fixed `ChatMessage::parse()` to use offset 9 for message start
- `src/bin/w3g-parser.rs` - Added `collect_chat_messages()` with validation heuristics

## Testing

Verified with `replay_5000.w3g`:
- Found 13 valid chat messages
- Player chat: "bad engagment", "from chief", "yeah", "gg"
- System messages: player leave notifications, desync warnings

## Status

COMPLETE - Chat parsing working correctly with proper false positive filtering.
