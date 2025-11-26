# Reforged Parser Survey

**Research Date:** November 25, 2025
**Purpose:** Investigate Warcraft 3 Reforged replay format changes and existing parser implementations

## Executive Summary

The Warcraft 3 Reforged replay format maintains backward compatibility with the classic W3G format, but with some changes introduced in version 1.32+. **Blizzard has not published official specifications for Reforged format changes**, forcing the community to rely on reverse-engineering. Most modern parsers handle both classic and Reforged replays, though documentation on specific differences remains sparse.

---

## Existing Implementations

### 1. w3gjs (TypeScript/JavaScript)
**Repository:** https://github.com/PBug90/w3gjs
**Language:** TypeScript
**Status:** Active, widely used

**Key Features:**
- From-scratch asynchronous, fully typed implementation
- Supports both high-level and low-level APIs
- Includes test replays named "reforged1.w3g" (suggests Reforged support)
- Does NOT support replays from version ≤ 1.14

**Strengths:**
- Well-tested and documented
- Flexible architecture allows custom parsing
- Used by various community tools
- TypeScript types available

**Limitations:**
- No explicit documentation of Reforged-specific changes
- Version-specific differences not clearly documented

**Documentation:** https://pbug90.github.io/w3gjs

---

### 2. w3rs (Rust)
**Repository:** https://github.com/aesteve/w3rs
**Language:** Rust
**Status:** Educational project, work-in-progress

**Key Features:**
- **Reforged-only** parser (explicitly targets Reforged)
- Uses `nom` parser-combinator library
- Recognizes units and abilities in-game

**Supported Actions:**
- Unit training and building construction
- Ability usage (spells, hero abilities, item consumption)
- Chat messages
- Skill learning by heroes
- Unit transformations

**Strengths:**
- Clean Rust implementation
- Can recognize units and abilities
- Better functional completeness than flo-w3reply

**Limitations:**
- Explicitly stated as "weekend experiment" / learning project
- Author recommends w3gjs for production use
- Extensive use of unsafe `unwrap()` calls
- No error handling yet

**Testing:** Uses subset of Reforged replays fetched from w3gjs repository

---

### 3. flo (w3champions/flo) (Rust)
**Repository:** https://github.com/w3champions/flo
**Language:** Rust
**Status:** Production use by W3Champions

**Key Features:**
- Complete Warcraft III toolkit (not just replay parser)
- Libraries for W3GS protocol parsing, map parsing, replay parsing/generating
- LAN game creation support
- Includes server implementation and client applications

**Replay Capabilities:**
- Limited to generating replay files
- Very limited real-time action stream processing (APM calculation)
- Designed for "partial" files (network packets, streaming parser)

**Strengths:**
- Production-proven by W3Champions platform
- Comprehensive toolkit beyond just replay parsing
- Supports game hosting and networking

**Limitations:**
- Replay parsing is not the primary focus
- Less functionally complete for replay parsing than w3rs
- Minimal documentation on action types

---

### 4. warcrumb (Go)
**Repository:** https://github.com/efskap/warcrumb
**Language:** Go
**Status:** Work-in-progress

**Key Features:**
- Based on w3g_format.txt specification
- Additional research into Reforged format (Battle.net 2.0 integration)
- Supports all versions including Reforged

**Action Types Supported:**
- BasicAbility (with AbilityFlags and ItemId)
- TargetedAbility (with target coordinates)
- Training units
- Constructing buildings

**Important Note:**
- Replays record **player inputs**, not outcomes
- AI decisions, unit deaths, resource counts cannot be extracted definitively

**Strengths:**
- Claims support for all game versions
- Go implementation (fast, efficient)
- Explicitly mentions Reforged support

**Limitations:**
- Documentation is minimal
- Specific Reforged changes not detailed
- Project described as "WIP... much like Reforged" (humorous but indicates incomplete status)

---

### 5. Other Notable Parsers

#### w3g (Python)
**Repository:** https://github.com/scopatz/w3g
**Package:** Available via PyPI (`pip install w3g`)
**Status:** Stable, documentation oriented

**Features:**
- Python 2/3 compatibility
- Easy-to-use API
- Post-processed metrics (APM calculation)
- Single module package

#### Warcraft III Replay Parser (PHP)
**Website:** https://w3rep.sourceforge.net/
**Language:** PHP
**Features:**
- Comprehensive extraction: player accounts, races, colors, heroes, units, chat logs
- Mature implementation

#### WC3 Replay Tool (by LadyRisa)
**Website:** https://replaytool.warcraft3.org/
**Features:**
- GUI tool with converter
- Automatic game version switching
- W3Champions profile integration
- Converts .nwg to .w3g format

#### wc3v (JavaScript/Browser)
**Repository:** https://github.com/jblanchette/wc3v
**Features:**
- In-browser replay viewer
- Creep route detection
- "Birds eye view" simulation
- Best-effort action detection

---

## Format Changes Documented

### Official Documentation Status

**Critical Finding:** Blizzard has **NOT published official specifications** for the Reforged replay format changes.

From the official Blizzard forums:
> "Is it possible to publish specs for the Reforged replay format or announce changes?"
>
> No response from Blizzard. The community continues to rely on reverse-engineering.

### Known Format Structure

The W3G format consists of:
1. **Header** (0x30 bytes base + subheader)
   - Version 0: 0x40 bytes total
   - Version 1: 0x44 bytes total
2. **Compressed data blocks**
3. **Action blocks** (documented in separate w3g_actions.txt)

### Version 1.32 (Reforged Release) Changes

**Documented Change:**
> "How replays are stored and loaded has changed with the 1.32 Reforged release"

**Suspected Changes:**
- Support for reconnection events (not in classic format)
- Increased player limit (Reforged supports more players)
- Battle.net 2.0 integration data
- Possibly extended header information

**Technical Limit:**
- Original format uses single-byte player IDs (theoretical max: 256 players)
- Classic limit was 12 players (ladder/custom games)
- Reforged increased limits for custom games

### Version History - Action Format Evolution

#### Pre-Patch 1.07 (Pre-TFT)
- Actions 0x10-0x14: **8 bytes shorter**
- Action 0x62: **4 bytes shorter**
- Actions 0x66-0x6A shifted (were 0x65-0x69)
- One unknown action between 0x63 and 0x65

#### Pre-Patch 1.13
- AbilityFlags: **single byte** (not word)
- Actions 0x10-0x14: **1 byte shorter**
- No autocast flag (0x0100)

#### Pre-Patch 1.14b
- Action 0x19: Different purpose (was simple subgroup selection)
- Actions 0x1B-0x1E: Shifted down (were 0x1A-0x1D)
- No PreSubselection action (0x1A introduced in 1.14b)
- Select Subgroup was 2 bytes (not 13 bytes)

#### Patch 1.17+
- Replay version number now explicitly stored in header

#### Patch 1.32+ (Reforged)
- Storage/loading mechanism changed
- Specific action format changes: **NOT DOCUMENTED**

### Community Reverse-Engineering Status

From forum discussions:
> "In the past, some people reverse-engineered the original format with some 'white spots', but it was enough for parsing replays."

Current situation:
- Community continues reverse-engineering Reforged changes
- Most parsers "just work" with Reforged replays
- Specific differences not explicitly documented

---

## Key Action Types for Reforged

### Complete Action Type Reference (from w3g_actions.txt)

#### Game Control Actions (0x01-0x07)
| ID | Name | Size | APM | Notes |
|----|------|------|-----|-------|
| 0x01 | Pause game | 1 byte | No | Multiple pauses don't stack |
| 0x02 | Resume game | 1 byte | No | First unpause resumes regardless |
| 0x03 | Set game speed | 2 bytes | No | 0x00=slow, 0x01=normal, 0x02=fast |
| 0x04 | Increase game speed | 1 byte | No | Numpad + |
| 0x05 | Decrease game speed | 1 byte | No | Numpad - |
| 0x06 | Save game | n bytes | No | Null-terminated filename |
| 0x07 | Save game finished | 5 bytes | No | Contains dword 0x00000001 |

#### Unit/Building Abilities (0x10-0x14)
| ID | Name | Size* | APM | Parameters |
|----|------|-------|-----|------------|
| 0x10 | Ability (no params) | 15 bytes | Yes | AbilityFlags, ItemID, 2 unknown dwords |
| 0x11 | Ability + position | 22 bytes | Yes | + X/Y coordinates (floats) |
| 0x12 | Ability + pos + object | 30 bytes | Yes | + 2 ObjectID dwords |
| 0x13 | Give/drop item | 38 bytes | Yes | Position, 2 target ObjectIDs, 2 item ObjectIDs |
| 0x14 | Dual ability | 43 bytes | Yes | 2 ItemIDs, 2 positions, 9 unknown bytes |

*Pre-1.07: 8 bytes shorter; Pre-1.13: 1 byte shorter (AbilityFlags was byte)

**AbilityFlags (word, post-1.13):**
- Bits 0-4: Ability queuing status
- Bit 8 (0x0100): Autocast on/off (added in 1.13)
- Other bits: Ctrl key, area effect, etc.

**ItemID Encoding:**
- String format: 4-character code (e.g., "ewsp" = Elf Wisp)
- Numeric format: ?? ?? 0D 00

#### Selection & Hotkeys (0x16-0x1A)
| ID | Name | Size | APM | Notes |
|----|------|------|-----|-------|
| 0x16 | Change Selection | 4+n×8 | Special | Mode: 0x01=add, 0x02=remove; n units with dual ObjectIDs |
| 0x17 | Assign Group Hotkey | 4+n×8 | Yes | Group 0-9 (key '1'=0, '0'=9); complete group list |
| 0x18 | Select Group Hotkey | 3 bytes | Yes | Group 0-9 + unknown byte (0x03) |
| 0x19 | Select Subgroup | 13 bytes | Special | ItemID + 2 ObjectIDs (≥1.14b); 2 bytes (<1.14b) |
| 0x1A | Pre Subselection | 1 byte | No | Precedes 0x19; added in 1.14b |

**APM Special Rules:**
- Don't count 0x16 (select) immediately after deselect within same CommandData block
- Same rule for 0x19 (Select Subgroup)

#### Post-1.14b Actions (0x1B-0x1E)
| ID (≥1.14b) | ID (<1.14b) | Name | Size | APM |
|-------------|-------------|------|------|-----|
| 0x1B | 0x1A | Unknown | 10 bytes | No | Scenarios/triggers only |
| 0x1C | 0x1B | Select Ground Item | 10 bytes | Yes | Left-click ground item |
| 0x1D | 0x1C | Cancel Hero Revival | 9 bytes | Yes | 2 UnitIDs (LAN/multiplayer) |
| 0x1E | 0x1D | Remove Unit from Queue | 6 bytes | Yes | SlotNr + ItemID |

#### Single Player Cheats (0x20-0x32)
These are **only in single-player replays** (not multiplayer):

| ID | Cheat Code | Effect |
|----|------------|--------|
| 0x20 | TheDudeAbides | Fast cooldown |
| 0x22 | SomebodySetUpUsTheBomb | Instant defeat |
| 0x23 | WarpTen | Fast construction |
| 0x24 | IocainePowder | Fast Death/Decay |
| 0x25 | PointBreak | Remove food limit |
| 0x26 | WhosYourDaddy | God mode |
| 0x27 | KeyserSoze | +Gold (5 bytes: 0xFF + dword) |
| 0x28 | LeafitToMe | +Lumber (5 bytes: 0xFF + dword) |
| 0x29 | ThereIsNoSpoon | Unlimited mana (NOT in replays) |
| 0x2A | StrengthAndHonor | No defeat |
| 0x2B | itvexesme | Disable victory |
| 0x2C | WhoIsJohnGalt | Enable research |
| 0x2D | GreedIsGood | +Gold and Lumber (5 bytes: 0xFF + dword) |
| 0x2E | DayLightSavings | Set time of day (4 bytes: float) |
| 0x2F | ISeeDeadPeople | Remove fog |
| 0x30 | Synergy | Disable tech tree |
| 0x31 | SharpAndShiny | Research upgrades |
| 0x32 | AllYourBaseAreBelongToUs | Instant victory |

#### Multiplayer Actions (0x50-0x62)
| ID | Name | Size | APM | Details |
|----|------|------|-----|---------|
| 0x50 | Change Ally Options | 6 bytes | No | Player slot + flags (allied, vision, control, shared victory) |
| 0x51 | Transfer Resources | 10 bytes | No | Player slot + gold dword + lumber dword |
| 0x60 | Map Trigger Chat | n bytes | No | 2 dwords + null-terminated trigger string |
| 0x61 | ESC Pressed | 1 byte | Yes | Precedes cancel actions, dialogs |
| 0x62 | Scenario Trigger | 13 bytes | No | 2 dwords + counter (≥1.07); 9 bytes (<1.07) |

#### Interface Actions (0x66-0x6A)
| ID (≥1.07) | ID (≤1.06) | Name | Size | APM |
|------------|-----------|------|------|-----|
| 0x66 | 0x65 | Enter Hero Skill Menu | 1 byte | Yes |
| 0x67 | 0x66 | Enter Building Menu | 1 byte | Yes |
| 0x68 | 0x67 | Minimap Signal (Ping) | 13 bytes | No | X, Y, dword (0x00A04040) |
| 0x69 | 0x68 | Continue Game (BlockB) | 17 bytes | No | 4 dwords [C][D][A][B] |
| 0x6A | 0x69 | Continue Game (BlockA) | 17 bytes | No | 4 dwords [A][B][C][D]; replay saver only |

#### Other Actions
| ID | Name | Size | APM | Notes |
|----|------|------|-----|-------|
| 0x75 | Unknown | 2 bytes | No | Scenarios/triggers only |

### Reforged-Specific Actions: UNKNOWN

**Critical Gap:** No new action IDs documented for Reforged beyond the classic set.

**Possible Scenarios:**
1. Reforged reuses existing action IDs with extended formats
2. New actions use IDs beyond 0x75 (undocumented)
3. Reforged changes are primarily in header/metadata, not actions
4. New features (reconnection, etc.) stored in separate data structures

**WC3 Replay Tool Changelog mentions:**
> "New actions added: leaving game, canceling hero revival, canceling units/buildings, left clicking items, control sharing"

However, these appear to be **existing actions** (0x17/LeaveGame, 0x1D/Cancel Revival, 0x1E/Remove from Queue, 0x1C/Select Ground Item) that may have been previously undocumented in some tools, NOT new Reforged actions.

---

## Recommended Approach

### 1. Compatibility Strategy

**Support Multiple Versions:**
```
Classic (≤1.31)  →  Version-specific action sizes/IDs
Reforged (≥1.32) →  Extended format handling
Version 2.0+     →  Monitor for new features
```

**Implementation:**
1. Parse header version information
2. Switch action parsing logic based on version
3. Maintain separate action size tables for pre/post version milestones
4. Gracefully handle unknown actions

### 2. Parser Architecture

**Recommended Pattern (from existing parsers):**

```rust
// Example from community parsers
match version {
    v if v < Version::new(1, 7, 0) => parse_pre_tft_action(),
    v if v < Version::new(1, 13, 0) => parse_pre_1_13_action(),
    v if v < Version::new(1, 14, 11) => parse_pre_1_14b_action(),
    v if v >= Version::new(1, 32, 0) => parse_reforged_action(),
    _ => parse_classic_action(),
}
```

**Key Decisions:**
- Use enum for action types (type safety)
- Include version context in parser state
- Log/track unknown actions for future investigation
- Maintain backward compatibility with classic replays

### 3. Testing Strategy

**Test Coverage:**
1. **Classic Replays** (pre-1.32):
   - Various patch versions (1.07, 1.13, 1.14b, 1.27+)
   - Different game modes (melee, custom maps, campaigns)

2. **Reforged Replays** (1.32+):
   - Early Reforged (1.32.x)
   - Recent patches (1.33+, 2.0+)
   - W3Champions replays (competitive, validated)

3. **Edge Cases:**
   - Replays with reconnections
   - Large player counts (custom games)
   - Custom map triggers
   - Tournament replays

**Test Sources:**
- w3gjs test replays: https://github.com/PBug90/w3gjs (includes reforged1.w3g)
- W3Champions public replays: https://www.w3champions.com/
- Warcraft3.info archives: https://warcraft3.info/replays/
- W3Replayers: https://w3replayers.com/

### 4. Handling Unknown Actions

**Conservative Approach:**
```rust
pub enum ActionParseResult {
    Known(Action),
    Unknown { id: u8, data: Vec<u8>, version: GameVersion },
    Malformed { reason: String },
}
```

**Benefits:**
- Doesn't crash on unknown actions
- Collects data for future analysis
- Enables community contribution to documentation

**Logging Strategy:**
- Warn on unknown action IDs
- Include context (game version, player, timestamp)
- Optionally save unknown action samples for review

### 5. Reference Implementations to Study

**Priority Order:**

1. **w3gjs** (Production-ready, TypeScript)
   - Study: Action parsing logic
   - Extract: Version detection mechanism
   - Learn: How they handle Reforged replays

2. **flo** (Production by W3Champions, Rust)
   - Study: Network protocol integration
   - Extract: Streaming parser design
   - Learn: Real-world production considerations

3. **w3rs** (Educational, Rust)
   - Study: Rust-specific patterns with `nom`
   - Extract: Action semantic analysis approach
   - Learn: Unit/ability recognition techniques

4. **warcrumb** (Go, Reforged focus)
   - Study: Battle.net 2.0 integration details
   - Extract: BasicAbility vs TargetedAbility patterns
   - Learn: Version compatibility approach

### 6. Documentation Gaps to Monitor

**Active Monitoring:**
1. Watch Blizzard forums for official announcements
2. Monitor parser repositories for issues/PRs
3. Track W3Champions Discord/community discussions
4. Check HIVE Workshop modding community

**If You Encounter Unknown Actions:**
1. Document the context (game version, map, mode)
2. Save the replay file
3. Open GitHub issues in parser communities
4. Consider contributing findings back to w3g_actions.txt maintainers

### 7. Immediate Next Steps

**For This Parser:**
1. ✅ Document all known action types (done in this survey)
2. Implement version detection from replay header
3. Create action size lookup table with version switching
4. Test with classic replays (validate existing implementation)
5. Acquire Reforged test replays
6. Test with Reforged replays (identify gaps)
7. Implement unknown action logging
8. Build test suite with multiple version samples

**For Community Engagement:**
1. Open dialogue with w3gjs maintainers
2. Share findings if new Reforged actions discovered
3. Contribute test replays to community repositories
4. Consider maintaining compatibility matrix document

---

## Specific Questions Answered

### Q1: Did Blizzard change the action format in Reforged?

**Answer:** **Probably YES, but specifics are UNDOCUMENTED.**

**Evidence:**
- Official statement: "How replays are stored and loaded has changed with the 1.32 Reforged release"
- No official specifications published by Blizzard
- Community parsers (w3gjs, warcrumb, w3rs) claim Reforged support
- Suspected changes: reconnection support, increased player limits, Battle.net 2.0 data

**Uncertainty:**
- No explicit action ID changes documented
- Unknown if new action types added or existing ones extended
- Changes might be in header/metadata rather than action blocks

### Q2: What new action types were added?

**Answer:** **UNKNOWN - No documented new action types.**

**Documented Classic Actions:** 0x01-0x75 (with gaps)

**Reforged Actions:** No additional action IDs documented beyond classic set

**Possible Explanations:**
1. Reforged reuses existing action IDs with extended payloads
2. New actions exist but haven't been reverse-engineered
3. New features use out-of-band data structures (not in action blocks)
4. Changes are primarily in non-action replay data

**Community Tools Note:** Some tools mention "new actions" (leaving game, control sharing, etc.) but these appear to be existing actions that were previously undocumented, not genuinely new Reforged additions.

### Q3: Are there version-specific differences (1.32 vs 1.33 vs 2.0)?

**Answer:** **YES for patch history; UNDOCUMENTED for recent versions.**

**Well-Documented Historical Differences:**
- **Pre-1.07:** Actions 8 bytes shorter, different action numbering
- **Pre-1.13:** AbilityFlags single byte (not word)
- **Pre-1.14b:** Different action IDs for 0x19-0x1E
- **1.17+:** Version number in header

**Reforged Era (1.32+): UNDOCUMENTED**
- 1.32 initial release: Storage/loading changed
- 1.33 changes: AI/scripting changes (later rolled back in 1.36.2)
- 2.0 (Nov 2024): Major update, HD graphics toggle, UI changes

**No Technical Specifications Available for:**
- 1.32 vs 1.33 action format differences
- Version 2.0 replay format changes
- Incremental patch differences within Reforged

**Recommendation:** Parse version from header, handle gracefully, log anomalies.

### Q4: How do other parsers handle the differences?

**Answer:** **Version-based switching + graceful degradation.**

**Common Patterns:**

1. **Version Detection:**
   - Parse version from replay header
   - Maintain version comparison logic
   - Switch parsing strategy based on detected version

2. **Size Calculation:**
   - Lookup tables for action sizes by version
   - Dynamic size calculation based on version context
   - Pre-computed offsets for different patches

3. **Action ID Mapping:**
   - Version-specific action ID interpretation
   - Handle shifted action IDs (pre-1.14b vs post)
   - Graceful handling of unknown actions

4. **Graceful Degradation:**
   - Don't crash on unknown actions
   - Log warnings for unsupported versions
   - Extract what's possible, skip what's not

**Examples:**

**w3gjs (TypeScript):**
- High-level and low-level APIs
- Event-driven architecture (`gamedatablock` events)
- Extensible via custom parsers

**w3rs (Rust):**
- Uses `nom` parser combinators
- Semantic analysis layer (interpret actions)
- Explicit Reforged-only focus (skips legacy compatibility)

**warcrumb (Go):**
- BasicAbility vs TargetedAbility abstraction
- Claims "all versions" support (mechanism unclear)
- Based on w3g_format.txt + own research

**flo (Rust, W3Champions):**
- Streaming parser for partial files
- Focus on APM calculation (limited action parsing)
- Production-proven but minimal documentation

**Common Challenge:** All parsers rely on reverse-engineering due to lack of official specs.

---

## Key Resources & References

### Primary Documentation
- **w3g_format.txt**: http://w3g.deepnode.de/files/w3g_format.txt (Official community spec)
- **w3g_actions.txt**: https://www.gamedevs.org/uploads/w3g_actions.txt (Complete action reference)
- **GitHub Gist (Mirror)**: https://gist.github.com/ForNeVeR/48dfcf05626abb70b35b8646dd0d6e92

### Parser Repositories
- **w3gjs (TypeScript)**: https://github.com/PBug90/w3gjs
- **w3rs (Rust, Reforged)**: https://github.com/aesteve/w3rs
- **flo (Rust, W3Champions)**: https://github.com/w3champions/flo
- **warcrumb (Go)**: https://github.com/efskap/warcrumb
- **w3g (Python)**: https://github.com/scopatz/w3g
- **wc3v (Browser)**: https://github.com/jblanchette/wc3v
- **w3rep (PHP)**: https://w3rep.sourceforge.net/

### Community Tools
- **WC3 Replay Tool**: https://replaytool.warcraft3.org/
- **W3Champions**: https://www.w3champions.com/
- **Warcraft3.info**: https://warcraft3.info/replays/
- **W3Replayers**: https://w3replayers.com/
- **HIVE Workshop**: https://www.hiveworkshop.com/

### Official Forums & Discussions
- **Reforged Format Discussion**: https://us.forums.blizzard.com/en/warcraft3/t/specs-for-replay-format-reforged/4015
- **Warcraft III Forums**: https://us.forums.blizzard.com/en/warcraft3/
- **Patch Notes (2.0)**: https://us.forums.blizzard.com/en/warcraft3/t/warcraft-iii-reforged-patch-notes-version-201/33148

### Technical References
- **W3G Format Description (Gist)**: https://gist.github.com/ForNeVeR/48dfcf05626abb70b35b8646dd0d6e92
- **W3Champions flo Discussion**: https://github.com/w3champions/flo/discussions/17
- **Liquipedia Patch History**: https://liquipedia.net/warcraft/Patch_1.36.2

---

## Conclusion

**Key Takeaways:**

1. **No Official Specs:** Blizzard has not published Reforged format specifications. All knowledge is community-driven reverse-engineering.

2. **Backward Compatible:** Reforged maintains basic W3G structure. Most classic parsing logic still works.

3. **Documented Gaps:** While classic action types (0x01-0x75) are well-documented, Reforged-specific additions/changes are unknown.

4. **Version Matters:** Parsing logic MUST account for patch version (pre-1.07, pre-1.13, pre-1.14b, 1.32+) due to size/ID differences.

5. **Proven Parsers Exist:** Multiple production parsers (w3gjs, flo, etc.) successfully handle Reforged replays, proving it's feasible despite documentation gaps.

6. **Graceful Degradation:** Best practice is to parse what's known, log what's unknown, and avoid crashing on unexpected data.

7. **Active Community:** Parser developers are active and collaborative. Sharing findings benefits everyone.

**Recommended Path Forward:**
1. Implement classic action parsing with version awareness
2. Test with both classic and Reforged replays
3. Log unknown actions for investigation
4. Engage with parser communities to share findings
5. Monitor for official documentation (unlikely but hopeful)

**Status:** Community-driven reverse-engineering continues. Format is parseable but requires defensive programming and ongoing updates as new game versions release.
