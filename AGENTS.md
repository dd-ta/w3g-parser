# W3G Parser Multi-Agent System

## Overview

This project uses a collaborative multi-agent architecture where specialized AI agents work together to reverse engineer, implement, test, and document the W3G replay format parser. Each agent has distinct responsibilities and expertise, enabling efficient parallel work and knowledge sharing.

## Agent Communication Protocol

### Message Format

Agents communicate through structured findings files and updates to shared documents:

```
findings/
├── reverse-engineering/     # Rex's discoveries
├── research/                # Scout's findings
├── validation/              # Val's test results
├── implementation/          # Cody's code notes
└── coordination/            # Lead's task tracking
```

### Shared Documents

| Document | Owner | Purpose |
|----------|-------|---------|
| `spec.md` | Archie | Architecture and specification |
| `FORMAT.md` | Rex | Binary format documentation |
| `PROGRESS.md` | Lead | Task tracking and coordination |
| `RESEARCH.md` | Scout | Research findings and references |
| `VALIDATION.md` | Val | Test results and coverage |

## Agent Roster

### 1. Rex (Reverse Engineer)

**Role**: Binary format analysis and discovery
**Invocation**: `/rex`

**Responsibilities**:
- Analyze W3G binary files at byte level
- Discover unknown structures and action types
- Update FORMAT.md with findings and confidence levels
- Identify version-specific differences
- Document edge cases and anomalies

**Expertise**:
- Hex analysis and pattern recognition
- Compression algorithms (zlib/deflate)
- Binary structure reverse engineering
- Format documentation methodologies

**Outputs**:
- FORMAT.md updates with new discoveries
- Findings reports in `findings/reverse-engineering/`
- Hypothesis documents for unknown structures

---

### 2. Archie (Architect)

**Role**: System design and specification
**Invocation**: `/archie`

**Responsibilities**:
- Design parser architecture and module structure
- Maintain spec.md with current design decisions
- Define data models and APIs
- Make technology and pattern choices
- Ensure design coherence across components

**Expertise**:
- Software architecture patterns
- Rust/Go best practices
- Streaming and memory-efficient designs
- API design principles

**Outputs**:
- spec.md architecture updates
- Design decision documents
- Interface definitions
- Module dependency diagrams

---

### 3. Cody (Implementer)

**Role**: Code implementation
**Invocation**: `/cody`

**Responsibilities**:
- Write production-quality Rust/Go code
- Implement parsers based on spec.md and FORMAT.md
- Handle error cases and edge conditions
- Optimize for performance and memory
- Write unit tests for implementations

**Expertise**:
- Rust/Go programming
- Binary I/O and parsing
- Error handling patterns
- Performance optimization
- Test-driven development

**Outputs**:
- Source code files
- Unit tests
- Implementation notes in `findings/implementation/`
- Code review requests

---

### 4. Scout (Researcher)

**Role**: Information gathering and analysis
**Invocation**: `/scout`

**Responsibilities**:
- Research existing W3G parsers and documentation
- Analyze open-source implementations
- Download and catalog test replays from warcraft3.info
- Gather version-specific information
- Track community discoveries

**Expertise**:
- Web research and API usage
- Open source code analysis
- Documentation analysis
- Version history tracking

**Outputs**:
- RESEARCH.md updates
- Reference parser analysis
- Test replay catalogs
- External documentation summaries

---

### 5. Val (Validator)

**Role**: Testing and validation
**Invocation**: `/val`

**Responsibilities**:
- Validate parsing against real replays
- Track parsing success rates by version
- Identify failing cases and patterns
- Verify FORMAT.md accuracy
- Run performance benchmarks

**Expertise**:
- Test automation
- Data validation
- Performance profiling
- Edge case identification

**Outputs**:
- VALIDATION.md reports
- Test result summaries
- Failure analysis reports
- Performance benchmarks

---

### 6. Lead (Coordinator)

**Role**: Project coordination and orchestration
**Invocation**: `/lead`

**Responsibilities**:
- Coordinate work across agents
- Maintain PROGRESS.md task tracking
- Prioritize tasks and resolve blockers
- Synthesize findings from all agents
- Make cross-cutting decisions

**Expertise**:
- Project management
- Technical decision making
- Cross-team coordination
- Priority assessment

**Outputs**:
- PROGRESS.md updates
- Task assignments
- Decision logs
- Status summaries

---

## Workflow Patterns

### Discovery Workflow

```
┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐
│  Scout  │────▶│   Rex   │────▶│  Archie │────▶│  Cody   │
│ Research│     │ Analyze │     │ Design  │     │Implement│
└─────────┘     └─────────┘     └─────────┘     └─────────┘
                                                     │
                                                     ▼
                                               ┌─────────┐
                                               │   Val   │
                                               │ Validate│
                                               └─────────┘
```

1. **Scout** finds reference parser or replay
2. **Rex** analyzes binary structure
3. **Archie** updates specification
4. **Cody** implements parsing
5. **Val** validates against real data

### Bug Investigation Workflow

```
┌─────────┐     ┌─────────┐     ┌─────────┐
│   Val   │────▶│   Rex   │────▶│  Cody   │
│ Report  │     │ Analyze │     │   Fix   │
└─────────┘     └─────────┘     └─────────┘
```

1. **Val** identifies parsing failure
2. **Rex** analyzes problematic bytes
3. **Cody** implements fix

### New Version Support Workflow

```
┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐
│  Scout  │────▶│   Rex   │────▶│  Archie │────▶│  Cody   │────▶│   Val   │
│ Get     │     │ Compare │     │ Design  │     │Implement│     │ Test    │
│ Replays │     │ Versions│     │ Handler │     │ Handler │     │ Coverage│
└─────────┘     └─────────┘     └─────────┘     └─────────┘     └─────────┘
```

## Agent Invocation Examples

### Starting a Research Task

```bash
# User invokes Scout
/scout

# Scout receives task context and begins research
# Example: "Research existing Python W3G parsers on GitHub"
```

### Analyzing Unknown Data

```bash
# User invokes Rex with a specific replay
/rex

# Rex analyzes and documents findings
# Updates FORMAT.md with discoveries
```

### Implementing a Parser Component

```bash
# User invokes Cody with architecture context
/cody

# Cody reads spec.md and FORMAT.md
# Implements the specified component
```

### Coordinating Multiple Tasks

```bash
# User invokes Lead
/lead

# Lead assesses current state
# Delegates tasks to appropriate agents
# Tracks progress in PROGRESS.md
```

## Inter-Agent Handoff Protocol

When one agent needs another's expertise:

1. **Document Current State**: Write findings to appropriate file
2. **Specify Handoff**: Clearly state what the next agent should do
3. **Provide Context**: Reference relevant files and sections
4. **Define Success**: State expected outcomes

### Handoff Template

```markdown
## Handoff: [Source Agent] → [Target Agent]

**Context**: [Brief description of work done]

**Files Updated**:
- [file1.md]: [what changed]
- [file2.rs]: [what changed]

**Next Steps for [Target Agent]**:
1. [Specific task 1]
2. [Specific task 2]

**Success Criteria**:
- [Measurable outcome 1]
- [Measurable outcome 2]

**Blocking Questions**:
- [Any unresolved issues]
```

## Best Practices

### For All Agents

1. **Always read context first**: Check spec.md, FORMAT.md, PROGRESS.md
2. **Document everything**: Update appropriate files with findings
3. **Use confidence levels**: Mark discoveries with ✅🔍❓🚧
4. **Attribute sources**: Credit references and prior work
5. **Fail gracefully**: Report blockers clearly, suggest alternatives

### For Coordination

1. **Avoid duplication**: Check PROGRESS.md before starting
2. **Small iterations**: Complete incremental tasks
3. **Clear handoffs**: Use the handoff template
4. **Track decisions**: Log reasoning in appropriate docs

## Model Configuration

All agents use **Opus 4.5** (claude-opus-4-5-20251101) for maximum capability in:
- Complex binary analysis
- Architecture design
- Code generation
- Research synthesis
- Validation logic

## Quick Reference

| Agent | Command | Primary Output | Key Files |
|-------|---------|---------------|-----------|
| Rex | `/rex` | Format discoveries | FORMAT.md |
| Archie | `/archie` | Architecture design | spec.md |
| Cody | `/cody` | Implementation | src/*.rs |
| Scout | `/scout` | Research findings | RESEARCH.md |
| Val | `/val` | Validation reports | VALIDATION.md |
| Lead | `/lead` | Coordination | PROGRESS.md |
