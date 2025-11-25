# Action Parser Fixes Implementation Plan

## Date: 2025-11-25
## Author: Archie (Planning Agent)

---

## Executive Summary

This plan addresses the high "Unknown" action rate by fixing three bugs and implementing missing action types. Expected reduction: 55-85% fewer Unknown actions.

---

## Phase 1: Fix Hotkey Parsing Bug (High Impact)

**Goal**: Fix hotkey (0x17) to properly consume unit IDs for Assign operations.
**Impact**: 15-25% reduction in Unknown actions.

### Changes Required

**File: `src/actions/hotkey.rs`**

Current: Parser always consumes 3 bytes.
Fix: For Assign (operation=0), read unit count and IDs.

```rust
// Structure for Assign:
// 17 [group: 1] [operation: 1] [unit_count: 2 LE] [unit_ids: 8*count]

pub struct HotkeyAction {
    pub group: u8,
    pub operation: HotkeyOperation,
    pub unit_ids: Vec<u32>,  // NEW: Only for Assign
}
```

Parse logic: If operation == Assign, read 2-byte count, then 8*count bytes of unit IDs.

---

## Phase 2: Fix Movement Subcommands (High Impact)

**Goal**: Handle all 0x00 subcommands (not just 0x0D).
**Impact**: 20-30% reduction in Unknown actions.

### Subcommands to Add

| Subcommand | Meaning |
|------------|---------|
| 0x0D | Move (existing) |
| 0x0E | Attack-move |
| 0x0F | Patrol |
| 0x10 | Hold position |
| 0x12 | Smart right-click |

All use same 28-byte structure.

### Changes Required

**File: `src/actions/movement.rs`**
- Add `MovementType` enum
- Update `MovementAction` to include movement_type field

**File: `src/actions/parser.rs`**
- Change match from `(0x00, Some(0x0D))` to `(0x00, Some(0x0D | 0x0E | 0x0F | 0x10 | 0x12))`

---

## Phase 3: Implement Missing Ability Types (Medium Impact)

**Goal**: Add handlers for 0x10-0x14 ability variants.
**Impact**: 15-20% reduction in Unknown actions.

### New Action Types

| Type | Name | Size |
|------|------|------|
| 0x10 | Unit ability (no target) | 14 bytes |
| 0x11 | Unit ability (ground target) | 22 bytes |
| 0x12 | Unit ability (unit target) | 30 bytes |
| 0x13 | Give/Drop Item | 30 bytes |
| 0x14 | Unit ability (two targets) | 38 bytes |

### Also Expand 0x1A Subcommands

- 0x01-0x0F: Ground-targeted (22 bytes)
- 0x10-0x18: Unit-targeted (14 bytes)

---

## Phase 4: Implement Remaining Types (Low Impact)

**Goal**: Handle edge-case action types.
**Impact**: 5-10% reduction in Unknown actions.

| Type | Name | Size |
|------|------|------|
| 0x19 | Select Subgroup | 13 bytes |
| 0x1E | Remove from Queue | 6 bytes |
| 0x50 | Change Ally Options | 6 bytes |
| 0x51 | Transfer Resources | 10 bytes |
| 0x68 | Minimap Ping | 13 bytes |

---

## Verification Commands

```bash
# After each phase:
cargo test
cargo run --bin w3g-parser -- parse --stats /path/to/replay.w3g

# Check Unknown rate
cargo run --bin w3g-parser -- parse --stats replay.w3g 2>&1 | grep -c "Unknown"
```

---

## Summary

| Phase | Focus | Expected Reduction |
|-------|-------|-------------------|
| 1 | Hotkey Assign fix | 15-25% |
| 2 | Movement subcommands | 20-30% |
| 3 | Ability types 0x10-0x14 | 15-20% |
| 4 | Edge cases | 5-10% |
| **Total** | | **55-85%** |
