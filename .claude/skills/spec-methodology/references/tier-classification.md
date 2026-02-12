# Tier Classification Methodology

How to assign features to implementation tiers. The classification uses two axes: **spec basis** (from feature extraction) and **prevalence** (how many tools implement it).

## Decision matrix

|  | High prevalence (3+ tools) | Medium (2 tools) | Low (1 tool) |
|--|--------------------------|-------------------|---------------|
| **spec-mandated** | Tier 1 | Tier 1 | Tier 2 |
| **spec-divergence** | Tier 1 | Tier 2 | Tier 2 |
| **spec-ambiguity** | Tier 2 | Tier 2 | Tier 3 |
| **tool-opinion** | Tier 2 | Tier 3 | Tier 3 |

## Tier definitions

### Tier 1: MVP

Ship in the initial release. These are non-negotiable.

**Characteristics:**
- Spec requires them OR near-universal tool consensus
- Users would consider the integration incomplete without them
- Typically 3-8 features

**Formatter example:** Indent style, indent width, line width, trailing newline
**Analyzer example:** Duplicate keys, invalid anchors, required key quoting

### Tier 2: Follow-up

Ship in the next minor release after MVP.

**Characteristics:**
- Common across tools but not universal
- Address spec ambiguities with sensible defaults
- Users would appreciate them but can work without them initially
- Typically 5-15 features

**Formatter example:** Quote style normalization, flow vs block style conversion, comment alignment
**Analyzer example:** Truthy values warning, empty mapping/sequence, key ordering

### Tier 3: Backlog

Ship when capacity allows. May never be implemented if value is low.

**Characteristics:**
- Single-tool features or pure style opinions
- Nice-to-have, not expected by most users
- Typically 5-20 features

**Formatter example:** Maximum line length for scalars, blank line enforcement between sections
**Analyzer example:** Maximum nesting depth, disallowed key names, schema validation

## Promotion rules

Features move UP tiers when:
- Implementation reveals they're prerequisites for other features
- User feedback shows high demand
- A spec update makes them mandatory

Features move DOWN tiers when:
- Implementation reveals unexpected complexity
- The feature conflicts with Biome's design philosophy
- Edge cases make the feature unreliable

## Applying to formatter options vs analyzer rules

**Formatter options** tend to cluster in Tier 1-2 because formatting is inherently opinionated and users expect control. A formatter with zero options is useless.

**Analyzer rules** spread more evenly across tiers because individual rules are independently valuable. A linter with 3 rules is useful (unlike a formatter with 0 options).

## Example classification (from YAML)

| Feature | Prevalence | Spec basis | Tier |
|---------|-----------|------------|------|
| Indent style (spaces) | 6/6 formatters | spec-mandated | 1 |
| Duplicate keys | 3/3 linters | spec-mandated | 1 |
| Truthy values | 2/3 linters | spec-divergence | 1 |
| Quote style | 4/6 formatters | tool-opinion | 2 |
| Key ordering | 2/3 linters | tool-opinion | 2 |
| Max line length | 1/3 linters | tool-opinion | 3 |
| Disallow anchors | 1/3 linters | tool-opinion | 3 |
