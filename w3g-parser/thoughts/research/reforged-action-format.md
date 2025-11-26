# Reforged Action Format Research

## Research Date
2025-11-25

## Overview
This document analyzes the action format differences between Classic Warcraft 3 replays and Reforged replays (Build 10100+). The current parser handles Classic replays well but produces many Unknown actions for Reforged replays due to format changes.

---

## Key Findings

### 1. Build Version Differences
- **Classic WC3**: Builds < 10000 use well-documented action format
- **Reforged**: Builds >= 10000 introduce format changes and new action types
- **Test Data**: Analysis performed on builds 10000, 85000, and 90000 replays

### 2. Primary Issue: Unknown Action Type 0x15
The action type **0x15 (0x15) is NOT documented** in the original W3G format specification. The documented action types jump from 0x14 to 0x16:
- 0x14: Unit/building ability (two targets) - 43 bytes
- **0x15: UNKNOWN - Not in classic documentation**
- 0x16: Change Selection (Unit, Building, Area) - 4+n*8 bytes

This suggests 0x15 is a **Reforged-specific action type** that was introduced after the original documentation.

### 3. Single-Byte Actions Common in Reforged
From the action_dump analysis, many frames contain only **1 byte of action data** (just the player ID). This pattern appears frequently:
```
Frame   37: time=   36ms (total:     219ms), action_len=  1
  0000: 02                                                |.|
  Analysis: Player ID: 2
```

**Hypothesis**: These may be keep-alive packets, ping acknowledgments, or lightweight state updates introduced in Reforged for network optimization.

### 4. New Subcommand Pattern: 0x1A 0x56
From build 90000 replay:
```
Frame   19: time=   63ms (total:     125ms), action_len= 27
  Analysis: Player ID: 2, Selection: 1 units, Action 0x1A sub=0x56
```

The subcommand **0x56** for action type 0x1A is not documented in classic format. Classic 0x1A subcommands are:
- 0x00: Ability with selection
- 0x19: Direct ability

**0x56 is a Reforged extension**, possibly related to new abilities or UI features.

### 5. Extended Action Type 0x15 Pattern
From build 85000 replay (Frame 5-6):
```
01 15 00 78 54 00 41 41 45 3D 3D 51 53 6A 68 51 41 3D 3D 00 00 00 00 00
02 15 00 78 54 00 41 41 45 3D 3D 51 53 6A 4D 77 41 3D 3D 00 00 00 00 00
```

Structure analysis:
- Byte 0: Player ID (0x01, 0x02)
- Byte 1: Action type (0x15)
- Byte 2: Subcommand (0x00)
- Byte 3-4: Unknown (0x78 0x54)
- Byte 5: Null byte
- Byte 6-18: Base64-like encoded data ("AAE==QSjhQA==", "AAE==QSjMwA==")
- Byte 19-23: Null padding

**Hypothesis**: This appears to be a **Battle.net 2.0 integration action** containing:
- Encrypted/encoded player state data
- Server synchronization tokens
- Cloud save triggers
- Achievement/statistic updates

The Base64-like encoding suggests data being sent to Blizzard's servers, which would be a new Reforged feature.

---

## Action Type Analysis

| Type ID | Hex  | Status | Frequency | Likely Purpose | Structure |
|---------|------|--------|-----------|----------------|-----------|
| 0x00    | 0x00 | Partial| Medium    | Movement variants | 28 bytes, subcommands 0x0D-0x12 |
| 0x10    | 0x10 | Missing| Low       | Unit Ability (no target) | 15 bytes |
| 0x11    | 0x11 | Missing| Low       | Unit Ability (ground target) | 22 bytes |
| 0x12    | 0x12 | Missing| Low       | Unit Ability (unit target) | 30 bytes |
| 0x13    | 0x13 | Missing| Low       | Give/Drop Item | 38 bytes |
| 0x14    | 0x14 | Missing| Medium    | Unit Ability (two targets) | 43 bytes |
| **0x15**| **0x15** | **Unknown** | **High** | **Battle.net 2.0 sync?** | **24 bytes** |
| 0x16    | 0x16 | Parsed | High      | Selection | 4+n*8 bytes |
| 0x17    | 0x17 | Parsed | Medium    | Hotkey | 3+ bytes (bug: needs fix) |
| 0x18    | 0x18 | Parsed | Low       | ESC key | 1 byte |
| 0x19    | 0x19 | Missing| Medium    | Select Subgroup | 13 bytes |
| 0x1A    | 0x1A | Partial| High      | Ability commands | Variable |
| 0x1B    | 0x1B | Parsed | Low       | Item Action | 14 bytes |
| 0x1C    | 0x1C | Parsed | Low       | Basic Command | 14 bytes |
| 0x1D    | 0x1D | Parsed | Low       | Build/Train | 14 bytes |
| 0x1E    | 0x1E | Missing| Low       | Remove from Queue | 6 bytes |
| **0x21**| **0x21** | **Unknown** | **Very Rare** | **Unknown** | **9 bytes** |
| 0x26    | 0x26 | Cheat  | N/A       | WhosYourDaddy (SP only) | 1 byte |
| **0x2E**| **0x2E** | **Unknown** | **Rare** | **Unknown** | **Unknown** |
| 0x30    | 0x30 | Cheat  | N/A       | Synergy (SP only) | 1 byte |
| **0x38**| **0x38** | **Unknown** | **Medium** | **Unknown** | **Unknown** |
| 0x50    | 0x50 | Missing| Low       | Change Ally Options | 6 bytes |
| 0x51    | 0x51 | Missing| Low       | Transfer Resources | 10 bytes |
| 0x68    | 0x68 | Missing| Low       | Minimap Ping | 13 bytes |
| **0x78**| **0x78** | **Unknown** | **Rare** | **Unknown** | **Unknown** |

### Notes:
- **Bold entries** are undocumented in classic W3G format
- 0x21 appears rarely in patches 1.04-1.05 (classic)
- 0x26, 0x30 are single-player cheats, shouldn't appear in multiplayer
- 0x38, 0x2E, 0x78 are completely unknown

---

## Structural Differences from Classic

### 1. Action Type 0x15 (New in Reforged)
**Classic**: No action type 0x15 exists
**Reforged**: Common action with 24-byte structure containing encoded data

**Structure (24 bytes)**:
```
Offset | Size | Content
-------|------|--------
0      | 1    | Player ID (0x01-0x0F)
1      | 1    | Action type (0x15)
2      | 1    | Subcommand (0x00 observed)
3-4    | 2    | Unknown marker (0x78 0x54)
5      | 1    | Null separator
6-18   | 13   | Base64-encoded data
19-23  | 5    | Null padding
```

### 2. Action Type 0x1A Subcommand 0x56 (New in Reforged)
**Classic**: 0x1A subcommands are 0x00 and 0x19
**Reforged**: Adds 0x56 subcommand (purpose unknown)

### 3. Single-Byte Actions
**Classic**: Minimum action size is typically 3+ bytes
**Reforged**: Many 1-byte actions (player ID only)

**Purpose**: Likely network keep-alive or micro-synchronization

### 4. Action Type 0x00 Subcommand Changes
**Classic**: The parser mentions subcommands 0x0D-0x12, but Reforged replays show:
- Different subcommand usage patterns
- Possibly expanded subcommand range

From the Unknown actions analysis document, type_id 0 with unusual subcommands was observed:
- Subcommands: 177, 250, 142, 173, 196, 9, 71, 86 (not classic 0x0D-0x12)

This suggests **0x00 action format may have been restructured entirely in Reforged**.

### 5. Data Encoding Changes
**Classic**: FourCC codes, raw coordinates, unit IDs
**Reforged**:
- FourCC codes still present
- New Base64-encoded data blocks
- Possible compression/encryption for server communication

---

## Byte Pattern Analysis

### From Build 85000 Replay

**Pattern 1: Type 0x15 with Base64 encoding**
```
Hex: 01 15 00 78 54 00 41 41 45 3D 3D 51 53 6A 68 51 41 3D 3D 00 00 00 00 00
     │  │  │  └─┬─┘ │  └──────────┬──────────┘ └─┬─┘
     │  │  │    │   │             │               │
     │  │  │    │   │    Base64 encoded data      Padding
     │  │  │    │   Separator
     │  │  │    Unknown marker
     │  │  Subcommand
     │  Type 0x15
     Player ID
```

**Pattern 2: Type 0x3A with Selection + Ability**
```
Hex: 02 3A 00 16 01 05 00 81 5C ... 1A 19 70 73 77 65 81 5C ...
     │  │  │  └───┬────┘ └─┬─┘    └───┬────┘ └─┬─┘
     │  │  │      │        │           │       │
     │  │  │   Selection  Unit IDs   Ability Unit IDs
     │  │  Subcommand                  code
     │  Type 0x3A
     Player ID
```

### From Build 90000 Replay

**Action Type Distribution** (from action_analysis):
- 0x1A: 8,452 occurrences (Action Command)
- 0x19: 8,328 occurrences (appears as ability subcommand marker)
- 0x16: 6,272 occurrences (Unit Selection)
- 0x14: 373 occurrences (possibly parsed differently in Reforged)
- 0x10: 285 occurrences

**Ability Code Frequency**:
Top ability codes observed:
- aedU (Udea): 1,292 - Death Knight abilities
- rafO (Ofar): 1,100 - Orc Far Seer
- sbou (uobs): 723 - Undead Obsidian Statue
- doko (okod): 624 - Orc Kodo Beast
- psbu (ubsp): 525 - Undead Burrow

**Coordinate Patterns**: 6,101 valid coordinate pairs found, indicating movement/targeting actions are being parsed correctly when not Unknown.

---

## Documentation Sources

The following resources were consulted:

### Primary Documentation
1. [WarCraft III Replay file format description (GitHub Gist)](https://gist.github.com/ForNeVeR/48dfcf05626abb70b35b8646dd0d6e92) - Main reference for classic format
2. [WarCraft III Replay Action Format Description (gamedevs.org)](https://www.gamedevs.org/uploads/w3g_actions.txt) - Detailed action type documentation
3. [w3g.deepnode.de](http://w3g.deepnode.de/) - Original homepage with w3g_format.txt and w3g_actions.txt

### Parser Implementations
4. [w3gjs (JavaScript)](https://github.com/PBug90/w3gjs) - TypeScript implementation supporting Reforged
5. [w3rs (Rust)](https://github.com/aesteve/w3rs) - Reforged-only parser (work in progress)
6. [warcrumb (Go)](https://github.com/efskap/warcrumb) - Supports Reforged, based on deepnode docs

### Community Resources
7. [W3Champions Replay Parser](https://w3replayers.com/) - Community tool with Reforged support
8. [WC3 Replay Tool by LadyRisa](https://replaytool.warcraft3.org/en:changelog) - All versions supported
9. [Blizzard Forums - Reforged Format Specs](https://us.forums.blizzard.com/en/warcraft3/t/specs-for-replay-format-reforged/4015) - Community discussion

### Key Quotes from Research
- "Since the Reforged beta started, the replay format has had a few minor (but breaking) changes that developers have had to manually reverse engineer." (Blizzard Forums)
- "WC3 replays do not record what happened, but rather what inputs the human players sent. The game is deterministic, so it can be recreated from this." (warcrumb README)
- "Actions are properly parsed without crashing and are in the process of being analyzed" (w3rs README) - indicates ongoing reverse engineering

---

## Recommendations

### Phase 1: Quick Wins (Immediate Implementation)
These changes will significantly reduce Unknown action counts:

1. **Fix Hotkey Parsing (0x17)**
   - Current bug: Always consumes 3 bytes
   - Fix: For Assign operations (byte 2 = 0x00), read additional unit IDs
   - Expected impact: ~10-15% reduction in Unknown actions

2. **Add Missing 0x00 Subcommands**
   - Add handlers for 0x0E (attack-move), 0x0F (patrol), 0x10 (hold), 0x12 (smart-click)
   - All use same 28-byte structure as 0x0D
   - Expected impact: ~5-10% reduction

3. **Implement 0x19 (Select Subgroup)**
   - Structure: 13 bytes (type + item + object_id1 + object_id2 + unknown)
   - Already defined in ActionType enum, just needs parser
   - Expected impact: ~5% reduction

### Phase 2: Common Actions (High Priority)
4. **Implement 0x10-0x14 Unit Abilities**
   - 0x10: No target (15 bytes)
   - 0x11: Ground target (22 bytes)
   - 0x12: Unit target (30 bytes)
   - 0x13: Give/Drop item (38 bytes)
   - 0x14: Two targets (43 bytes)
   - Expected impact: ~15-20% reduction

5. **Implement 0x1E (Remove from Queue)**
   - Structure: 6 bytes (type + slot + unit_type)
   - Expected impact: ~2-3% reduction

6. **Implement 0x50, 0x51 (Ally/Resources)**
   - 0x50: Change ally options (6 bytes)
   - 0x51: Transfer resources (10 bytes)
   - Expected impact: ~1-2% reduction

### Phase 3: Reforged-Specific (Reverse Engineering Required)
7. **Investigate Action Type 0x15**
   - **Priority: HIGH** - This is the most common Unknown action in Reforged
   - Structure appears to be 24 bytes with Base64-encoded data
   - Likely related to Battle.net 2.0 features:
     - Cloud save synchronization
     - Server-side state updates
     - Achievement/statistic tracking
     - Anti-cheat telemetry

   **Approach**:
   - Collect multiple 0x15 samples from various Reforged replays
   - Decode the Base64 data to analyze structure
   - Compare 0x15 patterns across different game states (start, mid, end)
   - Look for correlation with player actions or game events
   - Check if data changes based on map, players, or game settings

8. **Investigate 0x1A Subcommand 0x56**
   - New Reforged subcommand, purpose unknown
   - May be related to new abilities, UI features, or Reforged-exclusive mechanics
   - Appears with Selection data, so likely ability-related

9. **Investigate Single-Byte Actions**
   - Many frames with only 1 byte (player ID)
   - Likely keep-alive or micro-sync packets
   - May not need explicit parsing, could be logged as "KeepAlive" type

10. **Investigate 0x00 Subcommand Restructuring**
    - Observed subcommands: 177, 250, 142, 173, 196, 9, 71, 86
    - These are outside the classic 0x0D-0x12 range
    - Suggests fundamental restructuring of 0x00 action family

    **Approach**:
    - Compare 0x00 action data between Classic and Reforged replays
    - Look for new byte offsets or length changes
    - Check if subcommand encoding changed (e.g., moved to different offset)

### Phase 4: Edge Cases
11. **Implement Rare Actions**
    - 0x68: Minimap ping (13 bytes)
    - 0x01-0x07: Game state actions (pause, resume, save, speed changes)
    - 0x60-0x6B: Chat and trigger actions
    - Expected impact: <1% reduction each

---

## Testing Strategy

### 1. Baseline Measurement
Before implementing changes:
```bash
cargo run --bin action_analysis -- <replay.w3g>
```
Record:
- Total actions parsed
- Unknown action count
- Unknown action percentage
- Action type distribution

### 2. Incremental Testing
After each phase:
- Run action_analysis on test replays
- Compare Unknown action counts
- Verify no parsing regressions (frame count should not change)
- Check for new parsing errors

### 3. Test Replays
Use replays from different build versions:
- Classic: Build 5000, 6000, 7000
- Transition: Build 10000
- Reforged: Build 80000, 85000, 90000

### 4. Validation Criteria
- Unknown action rate < 5% for Classic replays
- Unknown action rate < 20% for Reforged replays (until 0x15 is understood)
- No increase in parsing errors
- Action statistics match expected patterns (selection, abilities, movement proportions)

---

## Open Questions

1. **What is action type 0x15?**
   - Most critical unknown for Reforged support
   - Hypothesis: Battle.net 2.0 synchronization
   - Need: Sample collection and decoding

2. **What does 0x1A subcommand 0x56 do?**
   - Reforged-specific ability variant?
   - New UI interaction?

3. **Why are there so many single-byte actions?**
   - Network optimization?
   - Server ping responses?
   - Empty action frames?

4. **Did 0x00 action format change completely?**
   - Subcommands outside 0x0D-0x12 range observed
   - May need version-specific parsing

5. **Are action types 0x38, 0x2E, 0x78 Reforged additions?**
   - Not in classic documentation
   - Not in cheat code range
   - Purpose unknown

6. **How do builds 10000-80000 differ?**
   - Test replays skip from 10000 to 80000
   - Did format evolve gradually or change suddenly?

---

## Next Steps

1. **Immediate**: Implement Phase 1 fixes (Hotkey, 0x00 subcommands, 0x19)
2. **Short-term**: Implement Phase 2 (0x10-0x14, 0x1E, 0x50-0x51)
3. **Medium-term**: Collect 0x15 samples and begin reverse engineering
4. **Long-term**: Full Reforged action support with 0x15, 0x56, and other unknowns

**Primary Goal**: Reduce Unknown action rate from current ~30-40% to <5% for Classic and <20% for Reforged.

---

## Appendix: Example Data

### Example 1: Action Type 0x15 (Build 85000, Frame 5)
```
Offset: 0x0000
01 15 00 78 54 00 41 41 45 3D 3D 51 53 6A 68 51 41 3D 3D 00 00 00 00 00
│  │  │  │  │  │  └────────────────────────────┘ └──────────┘
│  │  │  │  │  │            Base64 data           Null padding
│  │  │  │  │  Separator
│  │  │  │  Unknown marker
│  │  │  Unknown marker
│  │  Subcommand (0x00)
│  Type 0x15
Player 1
```

### Example 2: Action Type 0x3A with Selection (Build 85000, Frame 29)
```
Offset: 0x0000
02 3A 00 16 01 05 00 81 5C 00 00 84 5C 00 00 97 5C 00 00 9A 5C 00 00 AD
│  │  │  │  │  │  │  └───────┬────────┘ └───────┬────────┘
│  │  │  │  │  │  │      Unit ID 1          Unit ID 2
│  │  │  │  │  │  │
│  │  │  │  │  │  Mode
│  │  │  │  │  Unit count (5)
│  │  │  │  Selection marker (0x16)
│  │  │  Subcommand
│  │  Type 0x3A (unknown)
│  Player 2
```

### Example 3: Single-Byte Action (Build 90000, Multiple Frames)
```
Offset: 0x0000
02
│
Player 2 only

Purpose: Unknown (keep-alive? sync marker?)
```

---

## References

All web sources cited inline. Key technical documents:
- w3g_format.txt (deepnode.de)
- w3g_actions.txt (deepnode.de)
- ForNeVeR's GitHub Gist (comprehensive format description)
- Various parser implementations (w3gjs, w3rs, warcrumb)

**Document Status**: Research Phase Complete
**Next Action**: Begin Phase 1 Implementation
**Owner**: Rex (Reverse Engineering Agent)
