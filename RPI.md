# RPI Integration for W3G Multi-Agent System

## Philosophy

**The Problem**: LLMs perform best with well-defined problems and curated inputs. As projects grow, context explodes and agents lose coherence.

**The Solution**: Compression and distillation. Each agent produces focused artifacts that compress their findings. Other agents consume these artifacts rather than raw data.

**Key Insight**: Artifacts are the communication medium. Agents don't "talk" - they produce and consume structured, compressed knowledge.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         USER                                     │
│                           │                                      │
│                           ▼                                      │
│                      ┌────────┐                                  │
│                      │  LEAD  │  ← Orchestrator                  │
│                      └────┬───┘                                  │
│                           │                                      │
│         ┌─────────────────┼─────────────────┐                    │
│         │                 │                 │                    │
│         ▼                 ▼                 ▼                    │
│    ┌─────────┐      ┌──────────┐      ┌──────────┐              │
│    │RESEARCH │      │  PLAN    │      │IMPLEMENT │              │
│    │ PHASE   │      │  PHASE   │      │  PHASE   │              │
│    └────┬────┘      └────┬─────┘      └────┬─────┘              │
│         │                │                 │                    │
│    ┌────┴────┐      ┌────┴────┐      ┌────┴────┐               │
│    │  Scout  │      │ Archie  │      │  Cody   │               │
│    │   Rex   │      │         │      │   Val   │               │
│    └────┬────┘      └────┬────┘      └────┬────┘               │
│         │                │                 │                    │
│         ▼                ▼                 ▼                    │
│    thoughts/        thoughts/         thoughts/                 │
│    research/        plans/            impl/                     │
│                                       validation/               │
└─────────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
thoughts/
├── research/              # Scout + Rex outputs
│   ├── existing-parsers.md
│   ├── format-header.md
│   ├── format-actions.md
│   ├── version-differences.md
│   └── reforged-analysis.md
├── plans/                 # Archie outputs
│   ├── architecture-overview.md
│   ├── phase-1-foundation.md
│   ├── phase-2-core-parsing.md
│   └── component-action-parser.md
├── impl/                  # Cody outputs
│   ├── phase-1-notes.md
│   ├── blockers.md
│   └── technical-decisions.md
├── validation/            # Val outputs
│   ├── phase-1-results.md
│   ├── regression-baseline.md
│   └── performance-benchmarks.md
├── handoffs/              # Inter-agent requests
│   └── {timestamp}-{from}-to-{to}.md
└── decisions/             # Lead's decision log
    └── decision-log.md
```

## RPI Phases

### Phase 1: RESEARCH

**Goal**: Understand what currently exists. Produce compressed, focused reports.

**Active Agents**: Scout, Rex

**Triggers**:
- New project/feature
- Unknown format section
- Integration with external system
- Bug investigation

**Outputs**: `thoughts/research/*.md`

**Workflow**:
```
1. Lead identifies research questions
2. Lead spawns Scout (external) and/or Rex (binary analysis)
3. Agents research independently, may spawn sub-tasks
4. Agents write compressed findings to thoughts/research/
5. Lead reviews and determines readiness for planning
```

### Phase 2: PLAN

**Goal**: Create detailed implementation plans grounded in research.

**Active Agents**: Archie (primary), Lead (review)

**Triggers**:
- Research phase complete
- New feature request
- Architecture change needed

**Inputs**: `thoughts/research/*.md`

**Outputs**: `thoughts/plans/*.md`

**Workflow**:
```
1. Lead provides Archie with research artifacts
2. Archie synthesizes research into implementation plan
3. Plan broken into phases with clear validation criteria
4. Lead reviews plan, may request revisions
5. User approves plan before implementation
```

### Phase 3: IMPLEMENT

**Goal**: Execute the plan phase by phase with validation.

**Active Agents**: Cody (implementation), Val (validation)

**Triggers**:
- Plan approved
- Previous phase validated

**Inputs**: `thoughts/plans/*.md`, `thoughts/research/*.md`

**Outputs**: Source code, `thoughts/impl/*.md`, `thoughts/validation/*.md`

**Workflow**:
```
1. Lead assigns plan phase to Cody
2. Cody implements against spec, writes notes
3. Val validates implementation against plan criteria
4. If validation passes, proceed to next phase
5. If validation fails, cycle back (may need re-research or re-plan)
```

## Artifact Templates

### Research Artifact

```markdown
# Research: [Topic]

## Metadata
- **Agent**: Scout / Rex
- **Date**: YYYY-MM-DD
- **Confidence**: High / Medium / Low
- **Time Spent**: [context for depth]

## Executive Summary
[2-3 sentences capturing the key findings]

## Key Findings

### Finding 1: [Title]
[Compressed, relevant details]

### Finding 2: [Title]
[Compressed, relevant details]

## Implications for Planning
- [Specific recommendation 1]
- [Specific recommendation 2]

## Open Questions
- [What we still don't know]

## Sources
- [Attribution 1]
- [Attribution 2]
```

### Plan Artifact

```markdown
# Plan: [Feature/Component]

## Metadata
- **Agent**: Archie
- **Date**: YYYY-MM-DD
- **Research Used**: [list]
- **Status**: Draft / Under Review / Approved

## Overview
[What this plan accomplishes and why]

## Prerequisites
- [What must exist before starting]

## Phase 1: [Name]

### Goal
[Single sentence describing phase outcome]

### Files to Create/Modify
- `src/path/file.rs` - [purpose]
- `src/path/file2.rs` - [purpose]

### Implementation Steps
1. [Specific, actionable step]
2. [Specific, actionable step]
3. [Specific, actionable step]

### Validation Criteria
- [ ] [Testable criterion]
- [ ] [Testable criterion]

### Estimated Complexity
[Low / Medium / High] - [brief rationale]

## Phase 2: [Name]
[Same structure]

## Edge Cases
- [Edge case 1]: [How to handle]
- [Edge case 2]: [How to handle]

## Risks
- [Risk 1]: [Mitigation]

## Success Criteria
[How we know this plan succeeded overall]
```

### Implementation Notes Artifact

```markdown
# Implementation: [Plan Reference]

## Metadata
- **Agent**: Cody
- **Date**: YYYY-MM-DD
- **Plan**: thoughts/plans/[plan].md
- **Phase**: [N]

## Summary
[What was implemented]

## Deviations from Plan
- [Any changes made and why]

## Technical Decisions
- [Decision 1]: [Rationale]

## Blockers Encountered
- [Blocker]: [Resolution / Needs handoff]

## For Validation
- [Specific things Val should test]

## For Future Phases
- [Notes for subsequent implementation]
```

### Validation Artifact

```markdown
# Validation: [Plan/Phase Reference]

## Metadata
- **Agent**: Val
- **Date**: YYYY-MM-DD
- **Plan**: thoughts/plans/[plan].md
- **Phase**: [N]

## Summary
**Status**: PASS / FAIL / PARTIAL

## Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| [From plan] | ✅/❌ | [Details] |

## Test Results
- Unit tests: [X/Y passing]
- Integration tests: [X/Y passing]
- Replay validation: [X/Y parsing]

## Issues Found
- [Issue 1]: [Severity] - [Assigned to]

## Performance
- [Metric]: [Value] vs [Target]

## Recommendation
[Proceed to next phase / Fix issues first / Re-plan needed]
```

### Handoff Artifact

```markdown
# Handoff: [Source] → [Target]

## Request
- **From**: [Agent]
- **To**: [Agent]
- **Priority**: P0 / P1 / P2
- **Date**: YYYY-MM-DD HH:MM

## Context
[Why this handoff is needed]

## Request
[Specific ask - what the target agent should do]

## Inputs
- `thoughts/[path]` - [what it contains]

## Expected Output
- [Artifact type and location]
- [Success criteria]

---

## Response
- **Status**: Pending / Complete
- **Date**: YYYY-MM-DD HH:MM
- **Output**: `thoughts/[path]`
- **Notes**: [Any additional context]
```

## Agent Integration

### Lead (Orchestrator)

Lead becomes the RPI router:

```
/lead → Assess state, recommend next action
/research [topic] → Lead spawns Scout/Rex with research questions
/plan [topic] → Lead spawns Archie with research artifacts
/implement [plan] [phase] → Lead spawns Cody with plan
/validate [impl] → Lead spawns Val with implementation
```

Lead responsibilities:
1. Read PROGRESS.md and pending handoffs
2. Assess which RPI phase the project is in
3. Identify blockers and route around them
4. Spawn appropriate agents for the phase
5. Review artifacts and advance phases
6. Make cross-cutting decisions

### Scout (Research - External)

```
Inputs: Research question from Lead
Process: Web search, parser analysis, replay acquisition
Output: thoughts/research/[topic].md
```

Scout focuses on:
- Existing parser implementations
- External documentation
- Test replay acquisition
- Community knowledge

### Rex (Research - Binary Analysis)

```
Inputs: Research question from Lead, replay files
Process: Hex analysis, pattern recognition, format verification
Output: thoughts/research/[topic].md, FORMAT.md updates
```

Rex focuses on:
- Binary structure discovery
- FORMAT.md verification
- Unknown data investigation
- Version difference mapping

### Archie (Planning)

```
Inputs: Research artifacts, design question
Process: Architecture synthesis, pattern selection, phase breakdown
Output: thoughts/plans/[feature].md, spec.md updates
```

Archie focuses on:
- Translating research into architecture
- Breaking work into implementable phases
- Defining validation criteria
- Maintaining consistency with existing design

### Cody (Implementation)

```
Inputs: Plan artifact, research artifacts
Process: Code implementation, unit testing
Output: Source code, thoughts/impl/[feature].md
```

Cody focuses on:
- Implementing exactly what the plan specifies
- Writing tests for implementation
- Documenting deviations and decisions
- Flagging blockers for handoff

### Val (Validation)

```
Inputs: Implementation, plan artifact
Process: Testing, benchmarking, verification
Output: thoughts/validation/[feature].md
```

Val focuses on:
- Validating against plan criteria
- Running test suites
- Replay validation
- Performance benchmarking
- Go/no-go recommendations

## Workflow Examples

### Example 1: Starting Fresh

```
User: /lead
Lead: "Project at Phase 0. No research artifacts exist.
       Recommending: Start research phase.
       Questions to answer:
       1. What existing parsers can we learn from?
       2. What does the binary format actually look like?

       Run /research to begin."

User: /research existing parsers and format verification
Lead: *Spawns Scout: "Research existing W3G parsers"*
      *Spawns Rex: "Verify FORMAT.md against real replays"*

[Scout produces: thoughts/research/existing-parsers.md]
[Rex produces: thoughts/research/format-verification.md]

User: /lead
Lead: "Research complete. Key findings:
       - w3gjs is most maintained, covers TFT well
       - FORMAT.md is 70% accurate, several corrections needed

       Recommending: Plan phase for foundation.
       Run /plan foundation"

User: /plan foundation
Lead: *Spawns Archie with research artifacts*

[Archie produces: thoughts/plans/phase-1-foundation.md]

User: /lead
Lead: "Plan ready for review at thoughts/plans/phase-1-foundation.md
       3 phases identified. Please review and approve."

User: Approved. /implement thoughts/plans/phase-1-foundation.md phase 1
Lead: *Spawns Cody with plan*

[Cody implements, produces: src/*, thoughts/impl/phase-1-notes.md]

User: /validate
Lead: *Spawns Val*

[Val validates, produces: thoughts/validation/phase-1-results.md]
Val: "Phase 1 PASS. All criteria met. Ready for Phase 2."
```

### Example 2: Handling Unknown Data

```
User: /lead
Lead: "Val reported unknown action 0x75 in Reforged replays.
       This is blocking Reforged support.

       Recommending: Research cycle on action 0x75"

User: /research action 0x75
Lead: *Spawns Rex: "Analyze unknown action 0x75"*
      *Spawns Scout: "Search for 0x75 documentation in parsers"*

[Rex produces: thoughts/research/action-0x75.md]
[Scout produces: thoughts/research/action-0x75-external.md]

User: /lead
Lead: "Research suggests 0x75 is Reforged metadata sync.
       Confidence: Medium. Need architecture decision.

       Recommending: Plan update for Reforged handler"

User: /plan reforged action handling
[Archie updates plan with 0x75 handling]

User: /implement [plan] phase N
[Cody implements]

User: /validate
[Val confirms fix]
```

## Best Practices

### 1. One RPI Cycle Per Feature

Don't mix features. Complete research → plan → implement for one thing before starting another.

### 2. Artifacts Are King

If it's not in an artifact, it doesn't exist. Agents should never assume knowledge that isn't documented.

### 3. Compress Aggressively

Research artifacts should be 1-2 pages, not 10. Plans should be actionable, not exhaustive. The goal is to fit in context.

### 4. Lead Reviews Everything

Before advancing phases, Lead should review artifacts for completeness and coherence.

### 5. Validation Gates

Never proceed to next phase until current phase validates. Failed validation → investigate → possibly re-research or re-plan.

### 6. Parallel Research OK

Scout and Rex can research in parallel. But planning waits for all research. Implementation waits for plan approval.

### 7. Small Phases

Plans should have phases that can be implemented and validated in one session. If a phase is too big, break it down.

## Quick Reference

| Command | Phase | Agents | Output |
|---------|-------|--------|--------|
| `/lead` | Any | Lead | Status + recommendation |
| `/research [topic]` | Research | Scout, Rex | `thoughts/research/*.md` |
| `/plan [topic]` | Plan | Archie | `thoughts/plans/*.md` |
| `/implement [plan] [phase]` | Implement | Cody | Code + `thoughts/impl/*.md` |
| `/validate` | Implement | Val | `thoughts/validation/*.md` |

## Integration with Existing Files

| File | Role in RPI |
|------|-------------|
| `spec.md` | Living architecture doc, updated by Archie during planning |
| `FORMAT.md` | Living format doc, updated by Rex during research |
| `PROGRESS.md` | Task tracking, updated by Lead |
| `RESEARCH.md` | Summary index of research artifacts |
| `VALIDATION.md` | Summary index of validation results |
