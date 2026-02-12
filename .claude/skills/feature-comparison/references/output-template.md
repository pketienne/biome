# Feature Research Report Template

The canonical structure for feature extraction reports. Each section has specific requirements for content and formatting.

## Section: Executive Summary

**Purpose:** Give a reader the full picture in 30 seconds.

**Required content:**
- Total features found across all tools (count)
- Number of consensus features (in 2+ tools)
- Number of unique features (in 1 tool only)
- Key patterns observed (2-3 sentences)
- Language spec versions covered (e.g., "YAML 1.1 and 1.2")

**Length:** 8-15 lines.

## Section: Feature Matrices by Category

**Purpose:** Dense, scannable comparison of every feature across every tool.

**Required:** One matrix per tool type. Each matrix is a markdown table.

**Columns:** Feature name | Tool 1 | Tool 2 | ... | Tool N | Spec Basis

**Cell values:**
- `default` — feature is on by default
- `opt-in` — feature exists but is off by default
- `{value}` — configurable with this default value
- `-` — not supported
- `partial` — partially implemented (add footnote explaining what's missing)

**Ordering:** Group features by subcategory (e.g., "Key Rules", "Value Rules", "Formatting Rules" for linters). Within each group, order by prevalence (most common first).

## Section: Consensus Features

**Purpose:** Identify the strongest candidates for Biome implementation.

**Format:** Numbered list, ranked by prevalence then spec basis.

Each entry:
```
N. **feature-name** (N/M tools) — spec-basis
   Brief description. Tools: tool-a (default), tool-b (opt-in).
   Config: option-name (type, default).
```

## Section: Unique Features

**Purpose:** Capture potentially valuable features that only one tool implements.

Each entry:
```
- **feature-name** (tool-name only) — spec-basis
  What it does, why it might matter, whether it's worth implementing.
```

## Section: Spec Grounding

**Purpose:** Map the feature landscape onto the language specification.

**Structure by spec area**, not by tool. For each area of the spec:
- What the spec says
- How tools interpret it
- Where tools diverge from each other or the spec
- Which spec version matters (if there are version differences)

## Section: Architectural Observations

**Purpose:** Cross-cutting insights about implementation approaches.

Look for:
- Common configuration patterns (CLI flags, config files, inline comments)
- Error message conventions
- AST/CST structure choices
- Plugin/extension architectures
- Performance approaches

## Section: Default Configuration Comparison (appendix)

**Purpose:** Side-by-side comparison of what each tool does with zero configuration.

**Format:** Table with tools as columns, config categories as rows.

## Section: Recommended Next Steps

**Purpose:** Actionable priorities for the next phase.

**Format:** Numbered list of 3-7 items, each with a brief justification. Ordered by priority.
