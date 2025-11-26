# Reforged Compatibility Implementation Plan

## Date: November 25, 2025
## Author: Archie (Planning Agent)
## Based on: Rex's research findings & current parser analysis

---

## Executive Summary

The w3g-parser currently handles classic Warcraft 3 replays well but struggles with Reforged replays (build 10000+), producing Unknown actions at ~30-40% rates. The root causes are:

1. **Reforged-specific action types** (0x15 - 24 bytes with Base64 data)
2. **Version-aware parsing needed** for action size differences across patches
3. **Restructured 0x00 subcommands** (177, 250, 142, etc. instead of classic 0x0D-0x12)
4. **New 0x1A subcommand 0x56** (unknown purpose)
5. **Single-byte keep-alive packets** (just player ID, no action type)

This plan breaks Reforged support into 4 manageable phases with measurable impact at each stage. Backward compatibility with classic replays is maintained throughout.

---

## Phase 1: Quick Wins - Handle Common Reforged Patterns (1-2 weeks)

**Goal:** Reduce Unknown actions by 20-30% with minimal complexity
**Impact:** Address high-frequency, well-understood action types

### 1.1 Implement Single-Byte Actions (Keep-Alive Packets)

**Current Problem:**
- Many Reforged frames have 1-byte actions (just player ID)
- Parser currently treats these as Unknown
- From research: Build 90000 shows this pattern frequently

**Implementation:**
```
Location: src/actions/parser.rs - parse_action_type()

Add new ActionType variant:
  pub enum ActionType {
      KeepAlive,  // Single byte action, just player ID
      // ... existing variants
  }

Detection logic:
  if data.len() == 1 && is_reforged_version {
      return Ok((ActionType::KeepAlive, 1))
  }

Alternative (backward compatible):
  Could treat as "EmptyAction" or log as debug info
  Don't count toward Unknown actions
```

**Expected Impact:** 5-10% Unknown reduction
**Testing:** Run action_analysis on Reforged replays, verify fewer 1-byte Unknowns

---

### 1.2 Add Missing 0x19 (Select Subgroup) Parser

**Current Problem:**
- 0x19 action type is already defined in ActionType enum
- Parser has structure comment but no complete implementation
- Classic format: 13 bytes (type + item + 2×object_id + 3×unknown)

**Implementation:**
```
Location: src/actions/parser.rs - parse_action_type()

Current code at line 234-253 is incomplete.
Complete implementation:

(0x19, _) => {
    // SelectSubgroup: 13 bytes
    let consumed = 13.min(data.len());
    if data.len() >= 13 {
        let item = data[1];
        let obj_id1 = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
        let obj_id2 = u32::from_le_bytes([data[6], data[7], data[8], data[9]]);
        let unknown = [data[10], data[11], data[12]];
        Ok((
            ActionType::SelectSubgroup {
                item,
                object_id1: obj_id1,
                object_id2: obj_id2,
                unknown: unknown,
            },
            consumed,
        ))
    } else {
        Ok(Self::parse_unknown_action(action_type, subcommand, data))
    }
}
```

**Expected Impact:** 3-5% Unknown reduction
**Testing:** Verify SelectSubgroup actions are parsed, check byte consumption

---

### 1.3 Add 0x00 Subcommand Handlers (0x0D-0x12)

**Current Problem:**
- Only 5 of 6 movement subcommands implemented (0x0D, 0x0E, 0x0F, 0x10, 0x12)
- Missing 0x11 (likely Hold Position variant)
- All use same 28-byte structure

**Implementation:**
```
Location: src/actions/parser.rs - parse_action_type()

Extend the movement subcommand match:

(0x00, Some(sub @ (0x0D | 0x0E | 0x0F | 0x10 | 0x11 | 0x12))) => {
    let (mov, consumed) = MovementAction::parse_with_subcommand(data, sub)?;
    Ok((ActionType::Movement(mov), consumed))
}

All subcommands parse to same 28-byte structure:
  - 0x0D: Move
  - 0x0E: Attack-Move
  - 0x0F: Patrol
  - 0x10: Hold Position
  - 0x11: Smart Click
  - 0x12: (Need to verify from data)
```

**Expected Impact:** 2-4% Unknown reduction
**Testing:** Verify movement subcommands 0x11-0x12 are parsed correctly

---

### 1.4 Add 0x1A Subcommand 0x56 Handler (Reforged Extension)

**Current Problem:**
- 0x1A subcommand 0x56 appears in Reforged replays
- Purpose unknown (likely new ability variant or UI feature)
- Classic 0x1A only has 0x00 and 0x19

**Implementation:**
```
Location: src/actions/parser.rs - parse_action_type()

Add new match arm:

(0x1A, Some(0x56)) => {
    // Reforged-specific subcommand, unknown purpose
    // Similar structure to 0x1A 0x00 but may differ
    // For now: Parse as AbilityWithSelection but mark version
    let (ab, consumed) = AbilityWithSelectionAction::parse(data)?;
    Ok((ActionType::AbilityWithSelection(ab), consumed))
}

Alternatively, create new variant:
  ReforgedAbility { ability_code, ... }

This needs verification from actual Reforged replay data.
```

**Expected Impact:** 1-2% Unknown reduction
**Testing:** Find Reforged replays with 0x1A 0x56, verify parsing structure

---

### Phase 1 Summary
- **Total effort:** 40-60 hours (mostly data verification)
- **Expected Unknown reduction:** 12-25%
- **Risk level:** Low (all patterns well-understood from research)
- **Validation:** Run action_analysis baseline → implement changes → measure improvement

---

## Phase 2: Version-Aware Parsing Infrastructure (2-3 weeks)

**Goal:** Build foundation for handling version-specific differences
**Impact:** Enable proper handling of classic vs Reforged without code branching

### 2.1 Add Version Detection to Replay Header

**Current Problem:**
- Parser reads header but doesn't extract/use build number
- No context about Reforged vs classic when parsing actions

**Implementation:**
```
Location: src/header.rs (or new src/version.rs)

Add GameVersion struct:
  pub struct GameVersion {
      major: u8,
      minor: u8,
      revision: u8,
      build_number: u32,  // From replay header
  }

  impl GameVersion {
      pub fn is_reforged(&self) -> bool {
          self.build_number >= 10000
      }
      pub fn version_class(&self) -> VersionClass {
          match (self.major, self.minor) {
              (1, v) if v < 7 => VersionClass::PreTFT,
              (1, v) if v < 13 => VersionClass::TFT,
              (1, v) if v < 14 || (v == 14 && self.revision < 11) => VersionClass::Pre1_14b,
              _ if self.build_number >= 10000 => VersionClass::Reforged,
              _ => VersionClass::Classic,
          }
      }
  }

  pub enum VersionClass {
      PreTFT,      // Pre-1.07: 0x10-0x14 are 8 bytes shorter
      TFT,         // Pre-1.13: AbilityFlags is single byte
      Pre1_14b,    // Pre-1.14b: Different action IDs for 0x19-0x1E
      Classic,     // 1.14b-1.31: Standard format
      Reforged,    // 1.32+: New action types & formats
  }

Extract from header during parsing:
  pub fn read_version(&mut self) -> Result<GameVersion> {
      let major = read_u8()?;
      let minor = read_u8()?;
      let revision = read_u8()?;
      let build_number = read_u32_le()?;
      Ok(GameVersion { major, minor, revision, build_number })
  }
```

**Expected Impact:** Infrastructure for all future version-specific logic
**Testing:** Parse various replay versions, verify version detection accuracy

---

### 2.2 Create Action Size Lookup Tables

**Current Problem:**
- Hard-coded sizes assume classic format
- Reforged and pre-1.07/1.13/1.14b have different sizes
- No way to calculate correct action length dynamically

**Implementation:**
```
Location: src/actions/sizes.rs (new file)

pub struct ActionSizeTable {
    version: VersionClass,
}

impl ActionSizeTable {
    pub fn get_size(&self, action_type: u8, subcommand: Option<u8>) -> Option<usize> {
        match (self.version, action_type, subcommand) {
            // Classic sizes (1.14b+)
            (VersionClass::Classic, 0x10, _) => Some(15),
            (VersionClass::Classic, 0x11, _) => Some(22),
            (VersionClass::Classic, 0x12, _) => Some(30),
            (VersionClass::Classic, 0x13, _) => Some(38),
            (VersionClass::Classic, 0x14, _) => Some(43),
            (VersionClass::Classic, 0x19, _) => Some(13),

            // Pre-1.07 sizes (8 bytes shorter for 0x10-0x14)
            (VersionClass::PreTFT, 0x10, _) => Some(7),   // 15 - 8
            (VersionClass::PreTFT, 0x11, _) => Some(14),  // 22 - 8
            (VersionClass::PreTFT, 0x12, _) => Some(22),  // 30 - 8
            (VersionClass::PreTFT, 0x13, _) => Some(30),  // 38 - 8
            (VersionClass::PreTFT, 0x14, _) => Some(35),  // 43 - 8

            // Pre-1.13 sizes (AbilityFlags is 1 byte, not 2)
            (VersionClass::TFT, 0x10, _) => Some(14),  // 15 - 1
            (VersionClass::TFT, 0x11, _) => Some(21),  // 22 - 1
            (VersionClass::TFT, 0x12, _) => Some(29),  // 30 - 1
            (VersionClass::TFT, 0x13, _) => Some(37),  // 38 - 1
            (VersionClass::TFT, 0x14, _) => Some(42),  // 43 - 1

            // Reforged sizes (may have new actions)
            (VersionClass::Reforged, 0x15, _) => Some(24),  // Battle.net sync
            (VersionClass::Reforged, _, _) => self.get_classic_size(action_type, subcommand),

            _ => None,
        }
    }

    fn get_classic_size(&self, action_type: u8, _subcommand: Option<u8>) -> Option<usize> {
        match action_type {
            0x00 => Some(28),  // Movement commands
            0x0F => Some(19),  // Instant ability
            0x16 => None,      // Variable size (4 + n*8)
            0x17 => None,      // Variable size (4 + n*8)
            0x18 => Some(1),   // ESC key
            0x19 => Some(13),  // Select subgroup
            0x1A => None,      // Variable size
            0x1B => Some(14),  // Item action
            0x1C => Some(14),  // Basic command
            0x1D => Some(14),  // Build/train
            0x1E => Some(6),   // Remove from queue
            0x50 => Some(6),   // Change ally options
            0x51 => Some(10),  // Transfer resources
            0x68 => Some(13),  // Minimap ping
            _ => None,
        }
    }
}
```

**Expected Impact:** Foundation for version-specific parsing
**Testing:** Verify sizes against documentation, test with multi-version replays

---

### 2.3 Pass Version Context Through Parser

**Current Problem:**
- ActionIterator has no version information
- Can't make version-aware parsing decisions

**Implementation:**
```
Location: src/actions/parser.rs

Modify ActionIterator:
  pub struct ActionIterator<'a> {
      data: &'a [u8],
      offset: usize,
      context: ActionContext,
      version: GameVersion,      // NEW
      size_table: ActionSizeTable, // NEW
      finished: bool,
  }

  impl<'a> ActionIterator<'a> {
      pub fn new(data: &'a [u8], context: ActionContext, version: GameVersion) -> Self {
          let size_table = ActionSizeTable::new(version.version_class());
          Self {
              data,
              offset: 0,
              context,
              version,
              size_table,
              finished: false,
          }
      }
  }

Update parse_action_type to use version:
  fn parse_action_type(
      action_type: u8,
      subcommand: Option<u8>,
      data: &[u8],
      version: &GameVersion,
      size_table: &ActionSizeTable,
  ) -> Result<(ActionType, usize)> {
      // Now can make version-aware decisions
  }
```

**Expected Impact:** Enable all future version-specific handling
**Testing:** Verify version propagates through parser correctly

---

### Phase 2 Summary
- **Total effort:** 60-80 hours (mostly refactoring & testing)
- **Expected Unknown reduction:** 0% (foundation phase, not directly fixing actions)
- **Risk level:** Medium (touches core parser, needs careful refactoring)
- **Validation:** All existing tests still pass + multi-version test suite

---

## Phase 3: Reforged-Specific Actions (3-4 weeks)

**Goal:** Handle the high-frequency Reforged-specific patterns
**Impact:** Reduce Reforged Unknown actions by 50-70%

### 3.1 Implement Action Type 0x15 (Battle.net Sync)

**Current Problem:**
- 0x15 appears frequently in Reforged replays (high priority)
- Not in classic documentation
- 24-byte structure: Player(1) + Type(1) + Sub(1) + Unknown(2) + Null(1) + Base64Data(13) + Padding(5)

**Implementation:**
```
Location: src/actions/types.rs

Add new variant:
  pub enum ActionType {
      BattleNetSync {
          subcommand: u8,
          unknown_marker: u16,
          encoded_data: [u8; 13],  // Base64-like data
          padding: [u8; 5],
      },
      // ... existing variants
  }

Location: src/actions/parser.rs

(0x15, Some(sub)) => {
    // Reforged Battle.net 2.0 sync action
    let consumed = 24.min(data.len());
    if data.len() >= 24 {
        let unknown_marker = u16::from_le_bytes([data[2], data[3]]);
        let mut encoded = [0u8; 13];
        encoded.copy_from_slice(&data[6..19]);
        let mut padding = [0u8; 5];
        padding.copy_from_slice(&data[19..24]);

        Ok((
            ActionType::BattleNetSync {
                subcommand: sub,
                unknown_marker,
                encoded_data: encoded,
                padding,
            },
            consumed,
        ))
    } else {
        Ok(Self::parse_unknown_action(action_type, subcommand, data))
    }
}

Update is_known_action_type():
  fn is_known_action_type(byte: u8) -> bool {
      matches!(
          byte,
          0x00 | 0x0F | 0x15 | 0x16 | 0x17 | 0x18 | 0x19 | 0x1A | 0x1B | 0x1C | 0x1D | 0x1E | 0x50 | 0x51 | 0x68
      )
  }
```

**Analysis Strategy:**
The Base64 data (bytes 6-18) likely contains:
- Encrypted player state hash
- Server synchronization token
- Achievement/statistic updates
- Anti-cheat telemetry

For now, treat as opaque data. Future investigation could:
1. Collect samples from multiple replays
2. Decode Base64 to binary
3. Correlate with game events (win/loss, duration, etc.)
4. Look for patterns (same data for identical games?)

**Expected Impact:** 15-25% Unknown reduction for Reforged replays
**Testing:** Verify 0x15 actions are properly sized and boundary detection works

---

### 3.2 Implement 0x00 Subcommand Restructuring (Reforged)

**Current Problem:**
- Research found subcommands: 177, 250, 142, 173, 196, 9, 71, 86
- These are way outside classic 0x0D-0x12 range
- Suggests 0x00 structure may have changed fundamentally

**Implementation Strategy:**
```
Phase 3a: Investigation
  1. Collect actual Reforged replays with 0x00 actions
  2. Extract hex data for these actions
  3. Try to identify patterns (byte counts, structure)
  4. Compare with classic 0x00 actions

Phase 3b: Hypothesis Development
  If classic subcommand in offset 2 maps to different range:
    - Could be offset shift (now at offset 3?)
    - Could be different encoding (16-bit instead of 8-bit?)
    - Could be separate data structure

Phase 3c: Implementation
  Add version-specific parsing for 0x00:

  (0x00, Some(sub)) if version.is_reforged() => {
      // Try Reforged 0x00 parsing
      parse_reforged_movement(data, sub)
  }

  (0x00, Some(sub)) => {
      // Classic 0x00 parsing (existing)
      parse_classic_movement(data, sub)
  }
```

**Risk:** This is speculative - may not materialize
**Mitigation:** Collect actual replay data before implementing
**Expected Impact:** 5-15% if confirmed, 0% if false hypothesis

---

### 3.3 Implement Missing Unit Ability Actions (0x10-0x14)

**Current Problem:**
- These are documented but complex
- Different sizes per version (pre-1.07, pre-1.13, classic)
- Currently cause Unknown actions in actual games

**Implementation:**
```
Location: src/actions/types.rs

Add variants:
  pub enum ActionType {
      UnitAbilityNoTarget { ability_code: [u8; 4], flags: u16 },
      UnitAbilityGroundTarget { ability_code: [u8; 4], x: f32, y: f32 },
      UnitAbilityUnitTarget { ability_code: [u8; 4], target1: u32, target2: u32 },
      GiveDropItem { item_code: [u8; 4], source: u32, target1: u32, target2: u32 },
      UnitAbilityTwoTargets { ability_code: [u8; 4], target1: [u32; 2], target2: [u32; 2] },
  }

Location: src/actions/parser.rs

(0x10, _) => {
    let size = size_table.get_size(0x10, None).unwrap_or(15);
    // Parse unit ability with no target
    let consumed = size.min(data.len());
    if data.len() >= size {
        let ability_code = [data[1], data[2], data[3], data[4]];
        let flags = u16::from_le_bytes([data[5], data[6]]);
        Ok((
            ActionType::UnitAbilityNoTarget { ability_code, flags },
            consumed,
        ))
    } else {
        Ok(Self::parse_unknown_action(action_type, subcommand, data))
    }
}

// Similar for 0x11-0x14 with appropriate structures
```

**Expected Impact:** 5-10% Unknown reduction (these are relatively rare)
**Testing:** Find replays with unit abilities, verify parsing

---

### 3.4 Investigate Single-Byte Action Edge Cases

**Current Problem:**
- Phase 1 handles simple 1-byte keep-alives
- But Reforged may use 1-byte actions for other purposes
- Need to distinguish between different 1-byte action types

**Implementation:**
```
Research approach:
  1. Look for context around single-byte actions
  2. Check if they always appear at frame boundaries
  3. Verify they're truly just player ID (no action type byte)
  4. Correlate with game events if possible

Possible variants:
  - Ping/acknowledgment packets
  - Sync markers
  - Network state updates
  - Latency compensation

For now: Accept as KeepAlive, log frequency
Future: May need sub-typing based on context
```

**Expected Impact:** Already covered in Phase 1
**Testing:** Monitor keep-alive frequency, look for patterns

---

### Phase 3 Summary
- **Total effort:** 80-120 hours (includes investigation + implementation)
- **Expected Unknown reduction:** 40-60% for Reforged replays
- **Risk level:** Medium (some unknowns in 0x00 restructuring)
- **Validation:** Measure Unknown reduction, verify frame count unchanged

---

## Phase 4: Restructured Actions & Edge Cases (2-3 weeks)

**Goal:** Handle remaining edge cases and minor action types
**Impact:** Reduce Unknown rate to <10% for Reforged replays

### 4.1 Implement 0x00 0x1A Subcommand (If Different in Reforged)

Some evidence suggests 0x1A subcommand byte might be offset differently or have new meanings in Reforged.

### 4.2 Add Rare Unit Ability Actions (0x1E, 0x68 variants)

- 0x1E: Remove from queue (already partially done)
- 0x68: Minimap ping (already partially done)
- Edge cases with incomplete data

### 4.3 Implement Game State Actions

If appearing in replays:
- 0x01-0x07: Game control (pause, resume, speed)
- May need version-specific handling

### 4.4 Handle Unknown Reforged Action Types

Create robust logging for any remaining Unknown actions:
```
struct UnknownActionLog {
    game_version: GameVersion,
    action_type: u8,
    subcommand: Option<u8>,
    data_length: usize,
    sample_bytes: Vec<u8>,
    frequency: u32,
}

Enable collection mode:
  ./w3g-parser --collect-unknowns replay.w3g > unknown_actions.json

Analyze patterns:
  Identify if new action types are consistently formatted
```

### 4.5 Add Graceful Degradation

Even if action can't be parsed:
- Don't crash
- Log warning with context
- Extract what data is available
- Continue to next action

### Phase 4 Summary
- **Total effort:** 40-60 hours
- **Expected Unknown reduction:** Final 10-20%
- **Risk level:** Low (mostly edge cases & logging)
- **Validation:** Achieve <10% Unknown rate for Reforged, <5% for Classic

---

## Implementation Details

### Code Organization

```
src/
├── actions/
│   ├── mod.rs          (module root)
│   ├── parser.rs       (ActionIterator - MODIFY for phases 1-3)
│   ├── types.rs        (ActionType enum - EXTEND in each phase)
│   ├── sizes.rs        (NEW: Version-aware size tables, Phase 2)
│   ├── movement.rs     (existing)
│   ├── ability.rs      (existing)
│   ├── hotkey.rs       (existing)
│   ├── selection.rs    (existing)
│   └── reforged.rs     (NEW: Reforged-specific parsing, Phase 3)
│
├── version.rs          (NEW: GameVersion struct, Phase 2)
│
└── header.rs           (MODIFY: Extract version info, Phase 2)
```

### Phase Implementation Order

**Week 1-2: Phase 1**
- Add 1-byte keep-alive support
- Complete 0x19 implementation
- Add missing 0x00 subcommands (0x11, 0x12)
- Test with Reforged replays, measure baseline

**Week 3-5: Phase 2**
- Create GameVersion struct & extraction
- Build ActionSizeTable with version support
- Refactor ActionIterator to accept version
- Ensure no regression with classic replays

**Week 6-9: Phase 3**
- Implement 0x15 Battle.net sync action
- Investigate 0x00 restructuring (parallel research)
- Implement 0x10-0x14 unit abilities
- Expand test suite with more Reforged replays

**Week 10-12: Phase 4**
- Handle remaining edge cases
- Build unknown action logging
- Comprehensive testing across versions
- Documentation & release notes

---

## Verification Strategy

### Phase Validation Checklist

**Phase 1 Checkpoint:**
```
- [ ] Single-byte actions not in Unknown count
- [ ] 0x19 actions properly parsed (byte consumption correct)
- [ ] 0x00 0x11 & 0x12 recognized as movements
- [ ] 0x1A 0x56 parsed without errors
- [ ] Unknown rate drops 12-25%
- [ ] All classic tests still pass
- [ ] Frame count unchanged for all test replays
```

**Phase 2 Checkpoint:**
```
- [ ] GameVersion extracted from all replay types
- [ ] version_class() correctly categorizes: PreTFT, TFT, Pre1_14b, Classic, Reforged
- [ ] ActionSizeTable returns correct sizes per version
- [ ] Pre-1.07 sizes: 0x10=7, 0x11=14, 0x12=22, 0x13=30, 0x14=35
- [ ] Version context flows through ActionIterator
- [ ] Zero regression with classic replays
- [ ] Unknown rate unchanged (foundation phase)
```

**Phase 3 Checkpoint:**
```
- [ ] 0x15 actions parse to BattleNetSync struct (24 bytes total)
- [ ] 0x15 is included in is_known_action_type()
- [ ] 0x10-0x14 parsed with correct sizes per version
- [ ] 0x00 investigation complete with recommendations
- [ ] Unknown rate drops to <30% for Reforged
- [ ] Unknown rate stays <5% for Classic
```

**Phase 4 Checkpoint:**
```
- [ ] Unknown rate <10% for Reforged replays
- [ ] Unknown rate <5% for Classic replays
- [ ] Graceful degradation for any remaining unknowns
- [ ] Unknown action logging implemented & tested
- [ ] Comprehensive test suite covers all versions
- [ ] Documentation updated with version support
```

### Test Replay Sources

**Collect test replays:**
1. Classic (pre-1.32):
   - Build 5000, 6000, 7000 (pre-TFT)
   - Build 7500+ (TFT era)
   - Build 8500+ (late classic)

2. Reforged (1.32+):
   - Build 10000 (initial Reforged)
   - Build 85000 (mid-cycle)
   - Build 90000 (current)

3. Sources:
   - w3gjs repository (includes reforged1.w3g)
   - W3Champions public replays
   - Warcraft3.info archives
   - Community submissions (via GitHub)

### Measurement Approach

```
Baseline Run (before changes):
  cargo run --bin action_analysis -- <replay.w3g>
  Record:
    - Total actions
    - Unknown count
    - Unknown %
    - Action type distribution
    - Any errors

After Each Phase:
  Re-run same command
  Compare metrics:
    - Unknown % should drop
    - Total action count should stay same (sanity check)
    - New known action types should appear

Success Criteria:
  - Unknown rate Classic: < 5%
  - Unknown rate Reforged: < 10%
  - No parsing errors on valid replays
  - Frame count consistency: ±0 (exact)
```

---

## Implementation Priorities

### High Priority (Phase 1-2)
1. **0x15 Battle.net sync** (high frequency, unknown currently)
2. **Version-aware infrastructure** (foundation for everything)
3. **Single-byte actions** (common in Reforged, easy fix)

### Medium Priority (Phase 3)
4. **0x10-0x14 unit abilities** (documented, correct sizes known)
5. **0x00 restructuring investigation** (may not be needed, but high payoff if confirmed)

### Low Priority (Phase 4)
6. **Rare action types** (low frequency, less impact)
7. **Edge cases** (diminishing returns)

### Research Items (Parallel)
- Decode 0x15 Base64 data (optional, for curiosity)
- Investigate 0x00 subcommand changes (parallel to Phase 3)
- Monitor Blizzard forums for official documentation

---

## Risk Mitigation

### Risk: Version Detection Fails

**Mitigation:**
- Parse version from multiple locations in header
- Fall back to heuristics if primary fails
- Log warnings for ambiguous versions
- Conservative default (assume Classic if unsure)

### Risk: Reforged Changes in Future Patches

**Mitigation:**
- Generic version detection (major.minor.revision.build)
- Extensible size table (easy to add new entries)
- Graceful degradation (unknown actions don't crash)
- Community feedback mechanism (GitHub issues for new formats)

### Risk: Regression in Classic Replay Support

**Mitigation:**
- All classic tests must pass
- Version-specific code only activates when needed
- Classic path unchanged from current implementation
- Continuous integration testing for both Classic and Reforged

### Risk: Reverse-Engineered Formats Change

**Mitigation:**
- Don't assume 0x15 format is stable
- Don't hardcode 24-byte assumption
- Build validation that detects size mismatches
- Log warnings if format appears different

---

## Expected Outcomes

### After Phase 1 (Week 2)
- Unknown rate: 15-20% for Reforged (from ~35%)
- Classic support: No change (still ~5%)
- Quick wins delivered
- Foundation for Phase 2 clear

### After Phase 2 (Week 5)
- Unknown rate: Still 15-20% (infrastructure phase)
- Version detection working for all tested replays
- Size tables validated against historical data
- Ready for Phase 3 implementation

### After Phase 3 (Week 9)
- Unknown rate: 10-20% for Reforged
- 0x15 properly handled (major contribution)
- Unit abilities implemented (0x10-0x14)
- 0x00 investigation complete
- Clear picture of remaining unknowns

### After Phase 4 (Week 12)
- Unknown rate: <10% for Reforged
- Unknown rate: <5% for Classic
- All documented action types supported
- Graceful handling of future unknowns
- Production-ready Reforged support

---

## Future Opportunities

### Post-Phase 4 Enhancements

1. **Machine Learning Classification**
   - Train on known action patterns
   - Classify unknown actions by similarity
   - Improve accuracy over time

2. **Community Data Collection**
   - Gather unknown action samples
   - Share findings with parser community
   - Contribute discoveries back to w3g_actions.txt

3. **Real-Time Parsing**
   - Implement streaming parser
   - Support live replay upload/analysis
   - Partial file handling (like flo does)

4. **Integration with Game Events**
   - Cross-reference actions with map data
   - Verify unit codes against map specs
   - Detect cheats or anomalies

5. **Performance Optimization**
   - Benchmark action parsing speed
   - Optimize hot paths
   - Consider SIMD for boundary detection

---

## Documentation Deliverables

By end of Phase 4:

1. **Version Support Matrix** (.md file)
   - Table of supported versions
   - Known limitations per version
   - Status of each action type

2. **Action Format Reference** (.md file)
   - Updated action type specifications
   - Reforged-specific formats
   - Version-specific size variations

3. **Parser Architecture Guide** (.md file)
   - How version detection works
   - How to add new actions
   - How to extend size tables

4. **Test Suite Documentation** (.md file)
   - How to run test replays
   - How to add new test cases
   - How to measure Unknown rate

5. **Community Contribution Guide** (.md file)
   - How to report new action types
   - How to submit test replays
   - How to propose improvements

---

## Success Metrics

**Primary Goal:**
```
Unknown Action Rate:
  Classic replays:     < 5%    (current: ~5%, maintain)
  Reforged replays:    < 10%   (current: ~35%, improve 70%)
```

**Secondary Goals:**
```
Parsing Performance:
  No regression in parse speed
  Memory usage within bounds

Quality Metrics:
  100% code test coverage for new code
  All classic replays parse without error
  Frame boundaries never misaligned

Community Impact:
  Parser community aware of improvements
  Findings contribute to reverse-engineering efforts
  Reference implementation for other parsers
```

---

## Sign-Off

**Plan Status:** READY FOR IMPLEMENTATION

**Next Steps:**
1. ✅ Research complete (Rex's findings incorporated)
2. ✅ Plan created (this document)
3. ⏭️  Phase 1 implementation begins
4. ⏭️  Weekly progress updates
5. ⏭️  Phase completion reviews before moving to next

**Questions for Development Team:**
- Are test replays available for all versions mentioned?
- Should we start with Phase 1 only, or all 4 phases?
- Is there budget for community outreach (forums, Discord)?
- Any other Reforged-related issues beyond actions?

---

**Document Version:** 1.0
**Last Updated:** November 25, 2025
**Author:** Archie (Planning Agent)
**Status:** Ready for implementation kickoff
