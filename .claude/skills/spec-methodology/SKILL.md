---
name: spec-methodology
description: Use when writing implementation specifications for new Biome language support. Provides the spec document template, tier classification methodology, per-rule metadata format, and quality rubric. Discovered during YAML spec writing and validated as reusable for any language.
---

# Spec Methodology

How to write an actionable implementation specification for a new language in Biome. A spec bridges research (what features exist across tools) with architecture (how Biome integrates languages) to produce a document an implementer can follow without ambiguity.

## When to use

- Writing a new language support spec from Phase 1+2 outputs
- Revising a spec after implementation reveals gaps
- Evaluating whether a spec is complete enough for implementation
- Classifying features into implementation tiers

## Spec template

Every spec follows this structure exactly. See `references/spec-template.md` for the full template with per-section guidance.

```
# {Language} Support Implementation Specification

## Overview
## Prerequisites
## Layer 5: Formatter (Phase 1: MVP → Phase 2: Advanced → Phase 3: Edge Cases)
## Layer 6: Analyzer (Phase 1: Tier 1 → Phase 2: Tier 2 → Phase 3: Tier 3)
## Layer 7: Service Integration
## Implementation Order
## Testing Strategy
## Defaults That Differ from Biome Globals
## Open Questions
```

## Per-rule metadata format

Every lint rule in the spec must include all of these fields:

| Field | Description |
|-------|------------|
| Name | camelCase rule name (e.g., `noDuplicateKeys`) |
| Category | Biome category: `suspicious`, `correctness`, `style`, `complexity`, `a11y`, `performance`, `security` |
| Severity | `error` or `warning` |
| What it checks | One-sentence description |
| Config options | Each option with type and default value |
| Edge cases | Known tricky scenarios |
| Reference | Which tools implement this, with source file paths |
| Target file | `crates/biome_{lang}_analyze/src/lint/{category}/{rule_name}.rs` |

## Tier classification

See `references/tier-classification.md` for the full methodology.

| Tier | Criteria | Ship as |
|------|----------|---------|
| 1 | Consensus + high-impact + spec-mandated/divergence | MVP — must ship in initial release |
| 2 | Common features + spec-ambiguity + medium-prevalence | Follow-up — ship in next minor |
| 3 | Valuable niche + tool-opinion + low-prevalence | Backlog — ship when capacity allows |

## Quality rubric

Before declaring a spec complete:

- [ ] Every format option has: name, type, default, research report reference
- [ ] Every lint rule has all 8 metadata fields populated
- [ ] Every concern from architecture notes is addressed somewhere
- [ ] All file paths match the extension contract's patterns
- [ ] Each phase is independently shippable
- [ ] Implementation order respects the layer dependency graph
- [ ] "Defaults that differ from Biome globals" section exists and is populated
- [ ] Testing strategy covers all layers

## Model selection

Spec writing requires Opus. The task involves synthesizing 3 documents (500+ lines each), maintaining internal consistency across 20+ rules and 10+ format options, and making judgment calls about priorities and edge cases. Sonnet can produce the structure but misses edge case analysis and cross-reference consistency.
