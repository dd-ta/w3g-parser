# W3G Parser Validation

**Maintained by**: Val
**Last Updated**: 2025-11-25

## Overview

This document tracks validation results, test coverage, and quality metrics for the W3G parser.

## Current Status

**Parser Status**: Not yet implemented
**Test Suite Status**: Not yet created
**Last Full Validation**: N/A

## Validation Summary

### Parse Success Rates

| Version | Replays | Passed | Failed | Rate | Target |
|---------|---------|--------|--------|------|--------|
| Classic (RoC) | 0 | - | - | N/A | 90% |
| TFT (1.07-1.26) | 0 | - | - | N/A | 95% |
| TFT (1.27-1.31) | 0 | - | - | N/A | 95% |
| Reforged (1.32+) | 0 | - | - | N/A | 85% |
| **Total** | 0 | - | - | N/A | 90% |

### Test Coverage

| Module | Coverage | Target |
|--------|----------|--------|
| parser/ | 0% | 80% |
| version/ | 0% | 80% |
| models/ | 0% | 80% |
| compression/ | 0% | 80% |
| error/ | 0% | 80% |
| cli/ | 0% | 70% |
| **Total** | 0% | 80% |

## Test Replay Catalog

### By Version

| Version | Count | Source | Notes |
|---------|-------|--------|-------|
| *None yet* | | | |

### By Feature

| Feature | Replay IDs | Notes |
|---------|------------|-------|
| Chat messages | *TBD* | |
| Unknown actions | *TBD* | |
| Large files (1h+) | *TBD* | |
| Multi-player (8+) | *TBD* | |
| Campaign | *TBD* | |
| Custom maps | *TBD* | |

## Known Issues

### Open

| ID | Severity | Description | Assigned | Status |
|----|----------|-------------|----------|--------|
| *None yet* | | | | |

### Resolved

| ID | Severity | Description | Resolution | Date |
|----|----------|-------------|------------|------|
| *None yet* | | | | |

## Unknown Data Tracking

### Unknown Action Types

| Action ID | Occurrences | Replays | Hypothesis | Status |
|-----------|-------------|---------|------------|--------|
| *None observed yet* | | | | |

### Unknown Header Fields

| Offset | Size | Occurrences | Hypothesis | Status |
|--------|------|-------------|------------|--------|
| *None observed yet* | | | | |

## Performance Benchmarks

### Current Metrics

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Parse throughput | N/A | 10 MB/s | - |
| Memory (streaming) | N/A | 100 MB | - |
| Memory (batch) | N/A | 500 MB | - |
| Startup time | N/A | <100ms | - |

### Historical Benchmarks

| Date | Throughput | Memory | Notes |
|------|------------|--------|-------|
| *None yet* | | | |

## Validation Reports

### Latest Report

*No reports yet*

### Report Archive

| Date | Version | Summary | Link |
|------|---------|---------|------|
| *None yet* | | | |

## Test Plan

### Unit Tests

- [ ] Binary reading utilities
- [ ] Decompression functions
- [ ] Header parsing
- [ ] Action parsing (per type)
- [ ] Version detection
- [ ] Error handling

### Integration Tests

- [ ] Parse complete Classic replay
- [ ] Parse complete TFT replay
- [ ] Parse complete Reforged replay
- [ ] Streaming mode validation
- [ ] Batch processing validation
- [ ] CLI command validation

### Regression Tests

- [ ] Establish baseline replay set
- [ ] Automated comparison testing
- [ ] CI integration

### Performance Tests

- [ ] Large file benchmarks
- [ ] Memory profiling
- [ ] Streaming efficiency
- [ ] Parallel processing

### Fuzz Tests

- [ ] Header parsing fuzzing
- [ ] Action parsing fuzzing
- [ ] Compression handling fuzzing
- [ ] Full file fuzzing

## Validation Procedures

### For New Features

1. Write unit tests before implementation
2. Add integration test with real replay
3. Run full regression suite
4. Check performance impact
5. Document in this file

### For Bug Fixes

1. Add failing test case
2. Implement fix
3. Verify test passes
4. Run full regression
5. Update issue tracker

### Weekly Validation

1. Run full test suite
2. Parse entire replay catalog
3. Update success rate metrics
4. Run performance benchmarks
5. Generate validation report

## Quality Gates

### For Merge

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] No performance regression
- [ ] Code coverage maintained

### For Release

- [ ] 90%+ parse success rate
- [ ] 80%+ code coverage
- [ ] All P0/P1 issues resolved
- [ ] Performance targets met
- [ ] Documentation updated

## Notes

- Prioritize TFT replays for initial validation (most common)
- Track unknown data carefully for FORMAT.md updates
- Report all parsing failures to Rex for analysis
- Keep test replays versioned and documented
