# W3G Parser Research

**Maintained by**: Scout
**Last Updated**: 2025-11-25

## Overview

This document tracks research findings about W3G replay format, existing parsers, and useful resources for the project.

## Existing Parsers

### To Analyze

| Parser | Language | Repository | Status |
|--------|----------|------------|--------|
| w3gjs | TypeScript | github.com/PBug90/w3gjs | Pending |
| wc3-replay-parser | JavaScript | TBD | Pending |
| w3g-python | Python | Various | Pending |
| w3g-go | Go | TBD | Pending |
| w3g-rust | Rust | TBD | Pending |

### Analyzed

*None yet*

## Format Documentation Sources

### Known Resources

| Source | URL | Coverage | Notes |
|--------|-----|----------|-------|
| WC3 Replay Wiki | TBD | Unknown | To research |
| Older format docs | TBD | Pre-Reforged | To find |
| Blizzard official | N/A | None | No official docs |

### Community Sources

*To be populated*

## Test Replay Catalog

### warcraft3.info API

**Endpoint**: `https://warcraft3.info/api/v1/replays/{id}/download`

**Usage**:
```bash
curl -o replay.w3g "https://warcraft3.info/api/v1/replays/{id}/download"
```

**API Notes**:
- Rate limits: TBD
- Available metadata: TBD
- Replay ID format: TBD

### Downloaded Replays

| ID | Version | Size | Features | Status |
|----|---------|------|----------|--------|
| *None yet* | | | | |

### Replay Categories Needed

- [ ] Classic (RoC) - various versions
- [ ] TFT 1.07-1.26 (common versions)
- [ ] TFT 1.27-1.31 (pre-Reforged)
- [ ] Reforged 1.32+
- [ ] Large files (1+ hour games)
- [ ] Edge cases (unusual actions, corrupted)
- [ ] Multi-player (8+ players)
- [ ] Campaign replays
- [ ] Custom map replays

## Version History

### Build Number Ranges

| Version | Build Range | Release Date | Notes |
|---------|-------------|--------------|-------|
| RoC 1.00 | ~3xxx | 2002-07 | Original release |
| TFT 1.07 | ~4xxx | 2003-07 | Expansion |
| TFT 1.26 | ~6059 | 2011 | Long-stable version |
| Reforged 1.32 | 6100+ | 2020-01 | Major format changes |

### Known Format Changes

*To be researched and documented*

## Parsing Approaches

### Error Handling Strategies

| Parser | Approach | Notes |
|--------|----------|-------|
| *TBD* | | |

### Unknown Action Handling

| Parser | Approach | Notes |
|--------|----------|-------|
| *TBD* | | |

### Version Detection

| Parser | Approach | Notes |
|--------|----------|-------|
| *TBD* | | |

## Community Intelligence

### Active Projects

*To be populated*

### Known Experts

*To be populated (with public attribution only)*

### Useful Discussions

| Source | Topic | URL | Relevance |
|--------|-------|-----|-----------|
| *TBD* | | | |

## Research Tasks

### Pending

- [ ] Find and analyze w3gjs parser
- [ ] Search GitHub for other W3G parsers
- [ ] Find format documentation wikis
- [ ] Document warcraft3.info API behavior
- [ ] Identify Reforged-specific format changes
- [ ] Find campaign replay format differences

### Completed

*None yet*

## Attribution Log

All research findings will include proper attribution:

| Finding | Source | Date |
|---------|--------|------|
| *TBD* | | |

## Notes

- Prioritize TypeScript/JavaScript parsers (likely most maintained)
- Look for parsers handling Reforged specifically
- Note any parsing success rate claims
- Document any format uncertainties mentioned
