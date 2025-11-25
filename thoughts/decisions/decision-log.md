# Decision Log

This log tracks cross-cutting decisions made by Lead that affect the project.

---

## 2025-11-25: Project Language Selection

**Status**: PENDING (requires research)

**Context**: Need to choose between Rust and Go for implementation.

**Options**:
1. **Rust** - Memory safety, excellent binary handling, Cargo ecosystem
2. **Go** - Simpler, faster compile, good CLI tooling

**Decision**: Deferred to planning phase after research

**Action**: Scout to research existing parser ecosystems during first research cycle

---

## 2025-11-25: RPI Methodology Adoption

**Status**: DECIDED

**Context**: Complex reverse engineering project with multiple agents needs structured workflow.

**Options**:
1. Ad-hoc task assignment
2. RPI (Research, Plan, Implement) methodology
3. Waterfall phases

**Decision**: Adopt RPI methodology

**Rationale**:
- Context compression through artifacts prevents context explosion
- Phase discipline ensures research before planning, planning before implementation
- Validation gates ensure quality
- Supports parallel agent work through artifact handoffs

**Impact**:
- All work flows through RPI cycles
- Agents produce artifacts, not just outputs
- Lead orchestrates phase transitions
- Nothing advances without validation

---

## 2025-11-25: Artifact-Based Agent Communication

**Status**: DECIDED

**Context**: Multiple specialized agents need to share findings without overwhelming context.

**Options**:
1. Shared chat history (context explosion)
2. Direct file editing (conflicts)
3. Structured artifacts in `thoughts/` directory

**Decision**: Use `thoughts/` directory with structured artifacts

**Rationale**:
- Each artifact is compressed, focused knowledge
- Templates ensure consistency
- Artifacts can be referenced by path
- Enables asynchronous agent work
- Natural handoff mechanism

**Directory Structure**:
```
thoughts/
├── research/      # Scout, Rex outputs
├── plans/         # Archie outputs
├── impl/          # Cody outputs
├── validation/    # Val outputs
├── handoffs/      # Inter-agent requests
└── decisions/     # Lead decisions (this file)
```

---

## 2025-11-25: Model Selection for Agents

**Status**: DECIDED

**Context**: Need to select appropriate model for agent tasks.

**Options**:
1. Sonnet for all tasks (faster, cheaper)
2. Opus for all tasks (better quality)
3. Mixed (Sonnet for simple, Opus for complex)

**Decision**: Opus 4.5 for all agents

**Rationale**:
- Binary analysis and reverse engineering require strong reasoning
- Architecture decisions benefit from comprehensive analysis
- Implementation quality improves with better understanding
- Cost trade-off acceptable for project quality
- Consistency across agents simplifies coordination

**Impact**: All Task spawns use `model: "opus"`

---

## Template for Future Decisions

```markdown
## YYYY-MM-DD: [Decision Title]

**Status**: PENDING / DECIDED / SUPERSEDED

**Context**: [Why this decision is needed]

**Options**:
1. [Option A] - [Brief description]
2. [Option B] - [Brief description]

**Decision**: [What was decided]

**Rationale**: [Why this option was chosen]

**Impact**: [How this affects the project]
```
