---
name: feature-comparison
description: Use when producing structured feature extraction reports that compare capabilities across language tooling ecosystems. Provides the report template, spec classification categories, and feature matrix format discovered during YAML extraction and validated for reuse across languages.
---

# Feature Comparison

Structured output methodology for cross-tool feature extraction reports. Applies when comparing linters, formatters, parsers, or validators for any language.

## When to use

- Producing a feature research report for a new language
- Comparing tools within a category (linter vs linter)
- Classifying features by spec basis
- Synthesizing multi-agent extraction results into a unified report

## Report structure

Every feature research report follows this template exactly. See `references/output-template.md` for the full structure with section-level guidance.

```
# Feature Research Report: {Language}

## Executive Summary
## Feature Matrices by Category
## Consensus Features (ranked by prevalence)
## Unique Features (with rationale)
## Spec Grounding
## Architectural Observations
## Default Configuration Comparison (appendix)
## Recommended Next Steps
```

## Spec classification

Every feature must be classified into one of four categories. See `references/spec-classification.md` for definitions, examples, and edge cases.

| Category | Meaning |
|----------|---------|
| spec-mandated | The spec requires this behavior |
| spec-divergence | Different behavior between spec versions |
| spec-ambiguity | The spec is silent or underspecified |
| tool-opinion | Stylistic choice beyond the spec |

## Extraction parallelism

For 4+ repos, extract in parallel by tool type (not per-repo). Group all linters together, all formatters together, etc. Each parallel agent produces a coherent within-category comparison directly, avoiding heavy post-hoc synthesis.

## Feature matrix format

```markdown
| Feature | Tool A | Tool B | Tool C | Spec Basis |
|---------|--------|--------|--------|------------|
| feature-name | default/config | default/config | - | spec-mandated |
```

Use checkmarks for boolean support, config values for configurable features, `-` for unsupported.

## Consensus ranking

Rank consensus features by: (1) prevalence across tools, (2) spec basis (spec-mandated > spec-divergence > spec-ambiguity > tool-opinion), (3) user impact. This ranking directly informs tier classification in the spec-writing phase.
