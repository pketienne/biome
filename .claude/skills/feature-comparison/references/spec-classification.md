# Spec Classification Guide

Every extracted feature must be classified into exactly one of four categories. This classification drives tier ranking and implementation priority.

## Categories

### spec-mandated

The language specification **requires** this behavior. Any conforming implementation must handle it.

**Test:** Can you point to a specific section of the spec that mandates this behavior?

**Examples (YAML):**
- Duplicate keys produce undefined behavior (YAML 1.2 §3.2.1.3) — tools flag them
- Block scalars must use space indentation, not tabs (YAML 1.2 §6.1)
- Anchors must be defined before alias references (YAML 1.2 §3.2.2.2)

**Implementation priority:** Highest. These are correctness requirements.

### spec-divergence

The behavior **differs between spec versions**. Tools may target different versions or handle the divergence differently.

**Test:** Does the behavior depend on which version of the spec the tool follows?

**Examples (YAML):**
- `yes`/`no`/`on`/`off` are booleans in YAML 1.1 but strings in 1.2
- `0777` is octal in YAML 1.1 but a string in 1.2
- Merge key (`<<`) is a type in 1.1 but removed in 1.2

**Implementation priority:** High. Must document which version is targeted and how divergence is handled. May need a configuration option for spec version.

### spec-ambiguity

The spec is **silent, underspecified, or internally inconsistent** on this point. Tools make their own interpretation.

**Test:** Is there a reasonable reading of the spec that would allow different behaviors?

**Examples (YAML):**
- Comment placement after flow collections — spec doesn't specify formatting
- Trailing whitespace on empty lines — spec doesn't address
- Maximum nesting depth — spec doesn't define a limit
- Key ordering within mappings — spec says order is not significant, but tools may enforce it

**Implementation priority:** Medium. Document the ambiguity, choose a sensible default, consider making it configurable.

### tool-opinion

A **stylistic choice** that goes beyond the spec entirely. The spec neither requires nor addresses this behavior.

**Test:** Would an implementation without this feature still be fully spec-compliant?

**Examples (YAML):**
- Maximum line length enforcement
- Requiring quoted strings for certain values
- Enforcing consistent quote style (single vs double)
- Requiring blank lines between top-level keys
- File must end with newline

**Implementation priority:** Lower, but high-prevalence tool-opinions are strong Biome candidates (they reflect real user needs even though the spec doesn't address them).

## Edge cases

**Feature spans multiple categories:** Use the highest-priority category. If a feature enforces spec-mandated behavior but also addresses spec-divergence, classify as spec-divergence (because the divergence is the complexity driver).

**Tool implements more than the spec requires:** If a tool enforces spec-mandated behavior AND adds additional strictness beyond the spec, classify the spec-mandated part separately from the tool-opinion part.

**Spec version not determinable:** If the tool's documentation doesn't say which spec version it targets, classify based on the behavior observed. Note the ambiguity in the report.

## Using classification for tier ranking

| Tier | Primary composition |
|------|-------------------|
| Tier 1 | spec-mandated + high-prevalence spec-divergence |
| Tier 2 | spec-ambiguity + remaining spec-divergence + high-prevalence tool-opinion |
| Tier 3 | remaining tool-opinion + low-prevalence features |

Prevalence acts as a tiebreaker within each spec basis category. A tool-opinion implemented by 5/6 tools may rank higher than a spec-ambiguity feature in only 1 tool.
