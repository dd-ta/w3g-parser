# W3G Parser Stats and Metadata Improvement Plan

## Date: 2025-11-25
## Author: Archie

## Overview

This plan outlines a phased approach to improve the W3G parser's stats output and metadata extraction.

---

## Phase 1: Quick Wins (Header/Metadata)

### 1.1 Version String Derivation
- Build 10100 should display as "2.00" (Reforged)
- Formula: For builds >= 10000, version = 2.{(build-10000)/100:02}

### 1.2 Game Mode Inference
- 2 players = 1v1
- 4 players = 2v2
- etc.

### 1.3 Duration Format
- Display as hh:mm:ss

---

## Phase 2: Action Type Expansion

### Missing Action Types
- 0x18 = ESC key press
- 0x1B = Item/transfer command
- 0x1C = Basic command (stop, hold)
- 0x1D = Build/train command

---

## Phase 3: Stats Breakdown

### Per-Player Categories (matching warcraft3.info)
- APM (actions per minute)
- rightclick (movement)
- basic (stop, hold, patrol)
- buildtrain (build, train units)
- ability (spells)
- item (item usage)
- select (unit selection)
- assigngroup (Ctrl+N)
- selecthotkey (N to select)
- esc (ESC presses)

---

## Implementation Order

1. Phase 1.1 - version_string() method [15 min]
2. Phase 1.2 - game_mode inference [20 min]
3. Phase 1.3 - duration hh:mm:ss [5 min]
4. Phase 2 - action types [2-3 hrs]
5. Phase 3 - stats breakdown [3-4 hrs]
