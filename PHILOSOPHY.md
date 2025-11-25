# W3G Parser Philosophy

## Core Principle: Ground-Up Reverse Engineering

**We reverse engineer the W3G format ourselves.** We don't copy existing documentation or parsers. We discover the format by analyzing actual binary files.

### What This Means

**DO:**
- Download real replay files
- Analyze bytes directly with hex tools
- Form hypotheses from observed patterns
- Validate hypotheses against multiple files
- Build understanding incrementally
- Document our own discoveries with confidence levels

**DON'T:**
- Copy format specifications from wikis
- Read other parsers' source code for "answers"
- Use existing documentation as ground truth
- Skip the discovery process

### Why This Matters

1. **Learning**: The goal is to understand reverse engineering, not just build a parser
2. **Accuracy**: Our findings are validated against real data, not copied errors
3. **Completeness**: We may discover things others missed
4. **Confidence**: We know *why* each byte matters, not just *that* it matters

### How Existing Resources Can Be Used

**Allowed:**
- Download test replays from warcraft3.info (data source)
- Use existing parsers to *validate* our findings after we make them
- Reference other work for *comparison* after independent discovery
- Cite prior art when our findings align

**Not Allowed:**
- Reading format docs before analyzing the binary
- Using other parsers as the source of truth
- Skipping analysis because "someone already figured it out"

### The Process

```
1. Get raw replay file
2. Examine bytes (xxd, hexdump)
3. Identify patterns (magic bytes, lengths, structures)
4. Form hypothesis ("these 4 bytes are a length field")
5. Test hypothesis against multiple files
6. Document finding with confidence level
7. Repeat for next unknown section
```

### Confidence Levels (Earned, Not Copied)

- ✅ **CONFIRMED**: We verified this in 5+ replays, behavior is predictable
- 🔍 **LIKELY**: Pattern holds in 2+ replays, hypothesis fits
- ❓ **UNKNOWN**: We see it but don't know what it means yet
- 🚧 **INVESTIGATING**: Currently analyzing

### Validation Against Prior Art

After we've made our own discovery and documented it, we CAN:
- Compare against existing parsers to see if they agree
- Note discrepancies (maybe we found something they missed!)
- Update confidence level if external validation supports our finding

But the discovery must come first.

## Agent Roles Under This Philosophy

### Rex (Primary Discoverer)
Rex is the **primary reverse engineer**. Rex:
- Analyzes raw binary files
- Discovers format structures from scratch
- Documents findings in FORMAT.md
- Does NOT read existing format documentation

### Scout (Data Acquisition Only)
Scout acquires **test data**, not answers. Scout:
- Downloads replay files from warcraft3.info
- Catalogs replays by version/features
- Does NOT research format documentation
- Does NOT read other parsers' source code

### Post-Discovery Validation
After Rex documents a finding, Scout CAN:
- Search for whether others found the same thing
- Note if our finding aligns with or differs from prior art
- This is validation, not discovery

## Example Workflow

```
1. Scout downloads 10 replays (TFT, various versions)
2. Rex examines first 64 bytes of each
3. Rex notices: bytes 0-27 are identical across all files
4. Rex hypothesis: "This is a magic string identifier"
5. Rex decodes: "Warcraft III recorded game" + null bytes
6. Rex documents in FORMAT.md with ✅ CONFIRMED
7. Rex moves to next unknown section (bytes 28+)
8. Repeat until format is understood
```

## Success Metrics

- FORMAT.md filled with **our own discoveries**
- Each field has evidence from actual replay analysis
- We can explain *how* we figured out each structure
- Parser works because we understand the format, not because we copied code
