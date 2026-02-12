# Implementation Spec Template

The canonical document structure. Copy this skeleton and fill in language-specific content.

---

```markdown
# {Language} Support Implementation Specification

## Overview

Brief summary covering:
- Which layers are being specified (e.g., "Layers 5-7: Formatter, Analyzer, Service Integration")
- How many phases within each layer
- Total lint rules specified
- Total formatter options specified
- Language spec version(s) targeted

## Prerequisites

What must exist before implementation starts:
- Completed layers (grammar, syntax factory, parser) with crate paths
- Required tools: cargo-insta, cargo-expand, cargo-fuzz
- Reference implementation to follow (usually JSON)
- Input documents: feature-research-report.md, extension-contract.md, architecture-notes.md

## Layer 5: Formatter

### Crate Skeleton
- Crate path: `crates/biome_{lang}_formatter/`
- Required files: lib.rs, context.rs, {lang}_module.rs, generated.rs
- Codegen command: `cargo run -p xtask_codegen -- formatter`

### Formatter Options
Table of all options with:
| Option | Type | Default | Reference |
|--------|------|---------|-----------|
| indent_style | IndentStyle | Space | YAML spec §6.1 |

### Phase 1: MVP
Core node formatting. List each node type with:
- Node name (from .ungram)
- Formatting strategy (block_indent, verbatim, etc.)
- Special handling needed
- Reference: JSON equivalent node

### Phase 2: Advanced
Complex nodes, flow style, multi-document support.

### Phase 3: Edge Cases
Spec-version-specific behavior, rare constructs.

## Layer 6: Analyzer

### Phase 1: Tier 1 Rules (Consensus + High-Impact)

For each rule:

#### `ruleName`
- **Category:** suspicious/correctness/style
- **Severity:** error/warning
- **Recommended:** true/false
- **What it checks:** One-sentence description
- **Config options:**
  - `optionName` (type, default: value) — what it controls
- **Edge cases:**
  - Case 1: description
  - Case 2: description
- **Reference:** tool-name (path/to/implementation.ext)
- **Target file:** `crates/biome_{lang}_analyze/src/lint/{category}/{rule_name}.rs`

### Phase 2: Tier 2 Rules (Common)
Same format as Phase 1.

### Phase 3: Tier 3 Rules (Valuable)
Same format as Phase 1.

### Suppression Comments
- Comment syntax for the language
- Suppression format: `{comment-prefix} biome-ignore {rule-name}: {reason}`
- Parsing strategy
- Action implementation

## Layer 7: Service Integration

### DocumentFileSource
- File extensions to register (e.g., `.yaml`, `.yml`)
- MIME types (if applicable)

### ExtensionHandler
- Which capabilities to declare: Formatter, Analyzer, Search
- Handler file path: `crates/biome_service/src/file_handlers/{lang}.rs`

### ServiceLanguage
- How format/lint/check delegate to language crates
- Configuration resolution order

## Implementation Order

Dependency-respecting order:
1. Stage 1: Formatter crate (Layer 5) — can start immediately
2. Stage 2: Analyzer crate (Layer 6) — can start in parallel with Stage 1
3. Stage 3: Configuration (Layer 6) — depends on Stages 1+2
4. Stage 4: Service integration (Layer 7) — depends on Stages 1-3
5. Stage 5: Tests and polish — depends on Stages 1-4

Note which stages can be parallelized and which are serial.

## Testing Strategy

Per-layer testing approach:
- **Formatter:** Snapshot tests (spec_tests.rs + fixtures), quick_test.rs, idempotency check
- **Analyzer:** Per-rule valid/invalid fixtures, suppression tests
- **Configuration:** Snapshot test updates (accept new language key)
- **Integration:** CLI e2e tests (format, lint, check)
- **Fuzz:** Parser fuzz target (pre-implementation), formatter fuzz target (post-Stage 1)

## Defaults That Differ from Biome Globals

| Setting | Biome global default | {Language} default | Reason |
|---------|---------------------|-------------------|--------|
| indent_style | Tab | Space | {Language} spec §X.Y requires spaces |

## Open Questions

Numbered list of decisions deferred to implementation. Each with:
- The question
- Options considered
- Recommended default
- When it should be resolved
```
