# Plan: Generic Markdown Lint Rules Ontology for Biome

## Context

The existing ontology at `/home/pke/Projects/biome/kb/schemas/markdown-lint-rules.ttl` catalogs 53 markdownlint rules and 80 remark-lint rules as separate tool-specific individuals with cross-tool equivalence mappings. The user wants a new version that:

1. **Removes all tool-specific references** (no markdownlint, no remark-lint)
2. **Deduplicates** the combined rule set into a single canonical list
3. **Serves as the implementation guide** for markdown lint rules in the local biome fork at `~/Projects/biome/`

The biome fork already has 5 markdown rules implemented in nursery (`noDuplicateHeadings`, `noEmptyLinks`, `noInvalidHeadingLevel`, `noMissingLanguage`, `noReversedLinks`).

## File to create

`/home/pke/Projects/biome/kb/schemas/markdown-lint-rules.ttl` (replaces existing)

## Deduplication Math

- 33 rules exist in both tools (equivalences) → 33 merged entries
- 20 markdownlint-only rules → 20 entries
- 47 remark-lint-only rules → 47 entries
- **Total: 100 unique markdown lint rules**

## Schema Design

### Alignment with Biome's Rule Metadata

Map ontology properties to biome's `declare_lint_rule!` macro fields:

| Ontology Property | Biome Field | Values |
|---|---|---|
| `:biomeName` | `name` | camelCase string (e.g. "noDuplicateHeadings") |
| `:belongsToGroup` | group directory | correctness, style, a11y, suspicious, nursery |
| `:fixKind` | `fix_kind` | "none", "safe", "unsafe" |
| `:severity` | `severity` | "error", "warning", "information" |
| `:recommended` | `recommended` | boolean |
| `:implementationStatus` | (custom) | "implemented", "planned", "deferred" |

### Classes

- `:MarkdownRule` — a generic markdown lint rule (replaces `:MarkdownlintRule` and `:RemarkLintRule`)
- `:RuleGroup` — biome-style rule group (replaces `:RuleCategory`)

### Properties

- `:biomeName` — camelCase rule name for biome config
- `:belongsToGroup` — links rule to its biome group
- `:fixKind` — "none", "safe", or "unsafe"
- `:severity` — "error", "warning", or "information"
- `:recommended` — whether included in recommended config
- `:implementationStatus` — "implemented", "planned", or "deferred"

### Rule Groups (biome-aligned)

- `:Correctness` — catches definite bugs (empty URLs, undefined references, reversed links)
- `:Style` — enforces conventions (heading style, list markers, whitespace, code fences)
- `:A11y` — accessibility (alt text, descriptive link text)
- `:Suspicious` — likely mistakes (duplicate headings, emphasis as heading)
- `:Nursery` — experimental rules not yet categorized

### Rule Naming Convention

Every rule gets a `biomeName` in camelCase following biome's pattern:
- `noXxx` for rules that forbid something
- `useXxx` for rules that require something

### Implementation Status

Mark each of the 100 rules as:
- `"implemented"` — already exists in biome (5 rules)
- `"planned"` — should be implemented (76 rules)
- `"deferred"` — low priority or framework-specific: MDX, directives, file-name rules (19 rules)

## Implementation Results

### Verification

1. serdi validation: **PASSED**
2. Total rule count (`a :MarkdownRule`): **100**
3. Implemented count (`"implemented"`): **5**
4. Group distribution:
   - Correctness: 17
   - Style: 75
   - A11y: 2
   - Suspicious: 6
   - Nursery: 0

### Implemented Rules (5)

| biomeName | Group |
|---|---|
| `noDuplicateHeadings` | Suspicious |
| `noEmptyLinks` | Correctness |
| `noInvalidHeadingLevel` | Correctness |
| `noMissingLanguage` | Style |
| `noReversedLinks` | Correctness |
