# Unknown Actions Analysis for W3G Replay Parser

## Date: 2025-11-25
## Author: Rex + Scout (Research Agents)

---

## Executive Summary

Analysis of the W3G parser revealed several bugs and missing action type handlers causing high "Unknown" action rates. The primary issues are:

1. **Hotkey (0x17) byte consumption** - Only 3 bytes consumed, but Assign operations have more data
2. **Limited 0x00 subcommands** - Only 0x0D handled, missing attack-move, patrol, hold
3. **Limited 0x1A subcommands** - Only 0x00 and 0x19 handled
4. **Many action types not implemented** - 0x10-0x14, 0x19, 0x1E, 0x50-0x51, 0x60-0x6B

---

## 1. Currently Handled Action Types

| Type | Subcommand | Name | Size | Location |
|------|------------|------|------|----------|
| 0x00 | 0x0D | Movement | 28 bytes | `movement.rs:114-167` |
| 0x0F | 0x00 | InstantAbility | 18 bytes | `ability.rs:348-395` |
| 0x16 | any | Selection | 4+8*n bytes | `selection.rs:57-98` |
| 0x17 | any | Hotkey | 3 bytes | `hotkey.rs:52-84` |
| 0x18 | any | EscapeKey | 1 byte | `parser.rs:185-188` |
| 0x1A | 0x00 | AbilityWithSelection | variable | `ability.rs:248-292` |
| 0x1A | 0x19 | DirectAbility | 14 bytes | `ability.rs:168-208` |
| 0x1B | any | ItemAction | 14 bytes | `parser.rs:191-200` |
| 0x1C | any | BasicCommand | 14 bytes | `parser.rs:203-212` |
| 0x1D | any | BuildTrain | 14 bytes | `parser.rs:215-224` |

---

## 2. Identified Bugs

### Bug 1: Hotkey Action Size Mismatch

**Location:** `src/actions/hotkey.rs:36`

**Issue:** Parser always consumes exactly 3 bytes for hotkeys, but Assign operations (Ctrl+N) include the unit IDs being assigned to the group.

**Evidence:** Output shows:
```
Unknown { type_id: 17, subcommand: Some(0), data: [0, 24, 0] }
```
This is hotkey data being captured as "unknown" because parser stops at 3 bytes.

**Fix:** For Assign (operation=0), read additional unit ID data.

### Bug 2: Movement Subcommand Restriction

**Location:** `src/actions/parser.rs:148-151`

**Issue:** Only handles `(0x00, 0x0D)` but many movement-like commands use 0x00 with different subcommands.

**0x00 Subcommand meanings:**
| Subcommand | Meaning |
|------------|---------|
| 0x0D | Move (right-click ground) |
| 0x0E | Attack-move (A-click) |
| 0x0F | Patrol |
| 0x10 | Hold position |
| 0x12 | Smart right-click |

**Fix:** Handle additional 0x00 subcommands with same 28-byte structure.

### Bug 3: Limited 0x1A Subcommands

**Location:** `src/actions/parser.rs:172-181`

**Issue:** Only handles subcommands 0x00 and 0x19 for ability actions.

**Other 0x1A subcommands:**
| Subcommand | Meaning | Size |
|------------|---------|------|
| 0x01-0x0F | Ground-targeted abilities | 22 bytes |
| 0x10-0x18 | Unit-targeted abilities | 14 bytes |

**Fix:** Add handlers for additional 0x1A subcommands.

---

## 3. Missing Action Types (Need Implementation)

### High Priority (Common Actions)

| Type | Name | Size | Structure |
|------|------|------|-----------|
| 0x10 | Unit Ability (no target) | 14 | type + flags + ability |
| 0x11 | Unit Ability (ground target) | 22 | type + flags + ability + x,y |
| 0x12 | Unit Ability (unit target) | 30 | type + flags + ability + target |
| 0x13 | Give/Drop Item | 30 | type + flags + item + target + pos |
| 0x14 | Unit Ability (2 targets) | 38 | type + flags + ability + target1 + target2 |

### Medium Priority

| Type | Name | Size | Structure |
|------|------|------|-----------|
| 0x19 | Select Subgroup | 13 | type + ability + object_id |
| 0x1E | Remove from Queue | 6 | type + slot + unit_type |
| 0x50 | Change Ally Options | 6 | type + player + flags |
| 0x51 | Transfer Resources | 10 | type + player + gold + lumber |

### Low Priority (Rare/Special)

| Type | Name | Size | Notes |
|------|------|------|-------|
| 0x01 | Pause Game | 1 | Single byte |
| 0x02 | Resume Game | 1 | Single byte |
| 0x03 | Set Game Speed | 2 | speed value |
| 0x06 | Save Game | var | filename string |
| 0x60 | Map Trigger Chat | var | Custom map actions |
| 0x61 | ESC (alternate) | 1 | Duplicate of 0x18? |
| 0x68 | Minimap Ping | 13 | x + y + duration |

---

## 4. Data Structure Patterns

### FourCC Ability Codes
4 bytes stored reversed: `[0x77, 0x6F, 0x74, 0x68]` = "woth" -> "htow" (Town Hall)

Race prefixes:
- `h`/`H` - Human (lowercase=units, uppercase=heroes)
- `o`/`O` - Orc
- `u`/`U` - Undead
- `e`/`E` - Night Elf
- `n`/`N` - Neutral

### Unit IDs
8-byte blocks: `[id_low: 4 bytes LE] [id_high/counter: 4 bytes LE]`

### Coordinates
IEEE 754 single-precision floats (little-endian), range -15000 to +15000

### Target Encoding
8 bytes: All 0xFF = no target (ground click), otherwise unit ID

---

## 5. Observed Unknown Patterns

From actual parser output:

| Pattern | Count | Hypothesis |
|---------|-------|------------|
| `type_id: 17, data: [0, 24, X]` | High | Hotkey Assign with extra data |
| `type_id: 0, subcommand: 0x16` | High | Selection via 0x00 family |
| `type_id: 38 (0x26)` | Medium | Hero skill / learn ability |
| `type_id: 3` | Medium | Ping / communication |
| `type_id: 12 (0x0C)` | Medium | Unknown state change |
| `type_id: 14 (0x0E)` | Medium | Possibly patrol command |

---

## 6. Implementation Priority

### Phase 1: Fix Bugs (Immediate Impact)
1. Fix hotkey parsing to handle Assign operation data
2. Add 0x00 subcommands 0x0E, 0x0F, 0x10, 0x12

### Phase 2: Add Common Actions
1. Implement 0x10-0x14 ability variants
2. Implement 0x19 Select Subgroup
3. Implement 0x1E Remove from Queue

### Phase 3: Add Remaining
1. Implement 0x50, 0x51 (ally/resource)
2. Implement 0x68 (minimap ping)
3. Implement game state actions (0x01-0x06)

---

## 7. Verification Strategy

1. Parse test replays before/after changes
2. Compare Unknown action counts
3. Verify action categorization in stats output
4. Cross-reference with warcraft3.info stats
