# YAML Support Implementation Specification

## Overview

This specification covers the three missing layers needed for full YAML support in Biome:

- **Layer 5: Formatter** (`biome_yaml_formatter`) — IR-based formatting with YAML-specific options
- **Layer 6: Analyzer** (`biome_yaml_analyze`) — 22 lint rules across 3 priority tiers
- **Layer 7: Service Integration** (`biome_service` wiring) — File detection, settings, capability dispatch

Layers 1–4 (grammar, syntax, factory, parser) are complete and provide the CST foundation.

**Input documents:**
- Feature research: `references/yaml/feature-research-report.md` (90+ features across 13 tools)
- Extension contract: `references/biome/extension-contract.md` (7-layer integration model)
- Architecture notes: `references/yaml/architecture-notes.md` (YAML-specific concerns and gaps)

**Organization:** Each layer is broken into phases. Each phase is independently shippable. Within each phase, items are ordered by implementation dependency.

---

## Prerequisites

Before implementation begins, confirm:

1. **Completed layers** — `biome_yaml_syntax`, `biome_yaml_factory`, `biome_yaml_parser` all build and pass tests.
2. **Codegen** — `just gen-bindings` succeeds for YAML.
3. **Reference implementations accessible** — `crates/biome_json_formatter/`, `crates/biome_json_analyze/`, `crates/biome_service/src/file_handlers/json.rs` available as templates.
4. **Tools installed** — `cargo-insta` for snapshot testing, `just` for task running.

---

## Layer 5: Formatter

**Crate:** `crates/biome_yaml_formatter/`
**Reference:** `crates/biome_json_formatter/`
**Key traits:** `FormatLanguage`, `CstFormatContext`, `FormatOptions`, `CommentStyle`

### Phase 1: MVP — Core Formatting

Goal: Format single-document YAML files with block-style mappings and sequences using standard options.

#### 1.1 Crate Skeleton

Create `crates/biome_yaml_formatter/` with:

| File | Purpose | Reference |
|------|---------|-----------|
| `Cargo.toml` | Crate manifest, depends on `biome_formatter`, `biome_yaml_syntax` | `crates/biome_json_formatter/Cargo.toml` |
| `src/lib.rs` | `YamlFormatLanguage` implementing `FormatLanguage` | `crates/biome_json_formatter/src/lib.rs:246-279` |
| `src/context.rs` | `YamlFormatContext` and `YamlFormatOptions` | `crates/biome_json_formatter/src/context.rs:16-73` |
| `src/comments.rs` | `YamlCommentStyle` implementing `CommentStyle` | `crates/biome_json_formatter/src/comments.rs` |
| `src/cst.rs` | `FormatYamlSyntaxNode` root format rule | `crates/biome_json_formatter/src/cst.rs` |
| `src/generated/` | Per-node format rules (codegen) | `crates/biome_json_formatter/src/generated/` |

#### 1.2 Core Format Options

Implement `YamlFormatOptions` with:

| Option | Type | Default | Research Reference |
|--------|------|---------|-------------------|
| `indent_style` | `IndentStyle` (tabs/spaces) | `spaces` | yamlfmt: spaces, prettier: `useTabs: false` |
| `indent_width` | `IndentWidth` (1–24) | `2` | All 3 formatters default to 2 |
| `line_width` | `LineWidth` (1–320) | `80` | prettier: 80, yamlfix: 80, yamlfmt: 0 (off) |
| `line_ending` | `LineEnding` (lf/crlf/cr) | `lf` | prettier: lf, yamlfix: lf, yamlfmt: OS |

These 4 are the base `FormatOptions` trait requirements. Implement `as_print_options()` to convert to `PrinterOptions`.

#### 1.3 YAML-Specific Format Options

| Option | Type | Default | Research Reference | Notes |
|--------|------|---------|-------------------|-------|
| `yaml_quote_style` | `YamlQuoteStyle` enum (double/single/preserve) | `double` | prettier: double, yamlfix: single, yamlfmt: preserve | Applied to flow scalars; block scalars are never requoted |
| `yaml_trailing_comma` | `YamlTrailingComma` enum (none/all) | `none` | YAML spec does not require trailing commas in flow collections | For flow sequences/mappings only |

**Deferred to Phase 2:** `yaml_document_start` (require/omit/preserve `---`), `yaml_sequence_indent` (separate indent for sequences).

#### 1.4 Comment Handling — `YamlCommentStyle`

Implement `CommentStyle` for YAML's `#` comments. This is the most complex part of the formatter.

**Comment types to handle:**

| Position | Example | Placement Rule |
|----------|---------|----------------|
| End-of-line | `key: value  # comment` | Trailing on the same node |
| Own-line above node | `# comment\nkey: value` | Leading on the next sibling or first child |
| Own-line between nodes | `item1\n# comment\nitem2` | Trailing on previous sibling |
| Document-level | `# file header\n---` | Leading on the document node |
| Inside empty collections | `{}\n# orphan` | Dangling in parent |

**Implementation strategy:**
1. Use `biome_formatter::comments::CommentPlacement` to classify each comment.
2. For end-of-line comments: always trailing on the enclosing node.
3. For own-line comments: leading on the next node if one exists, otherwise trailing on the previous node, otherwise dangling in the parent.
4. Preserve a minimum of 1 space between content and end-of-line comments.
5. Align consecutive end-of-line comments when they appear on adjacent lines (stretch goal).

**Reference:** `crates/biome_json_formatter/src/comments.rs` — but YAML has more ambiguous positions than JSON. Expect this file to be 2–3x larger.

#### 1.5 Node Formatting Priorities

Implement `FormatRule` for nodes in this order (most structurally important first):

| Priority | Node Type | IR Strategy | Notes |
|----------|-----------|-------------|-------|
| 1 | `YamlDocument` | `hard_line_break()` between documents | Multi-doc boundary |
| 2 | `YamlBlockMapping` | `block_indent()` wrapping entries | Core structure |
| 3 | `YamlBlockMappingEntry` | `key: space value` with indent | Key-value pair |
| 4 | `YamlBlockSequence` | `block_indent()` wrapping items | List structure |
| 5 | `YamlBlockSequenceEntry` | `- space value` with indent | List item |
| 6 | `YamlPlainScalar` | `text()` verbatim | No transformation |
| 7 | `YamlSingleQuotedScalar` | `text()` with quote style application | May restyle |
| 8 | `YamlDoubleQuotedScalar` | `text()` with quote style application | May restyle |
| 9 | `YamlFlowMapping` | `group()` with `soft_line_break()` | May break to multi-line |
| 10 | `YamlFlowSequence` | `group()` with `soft_line_break()` | May break to multi-line |

**Indentation strategy:** YAML indentation is structural. The formatter must:
1. Track current block indentation depth.
2. Apply `indent_width` uniformly to all nested blocks.
3. Never change indentation of literal/folded block scalar content (Phase 2).
4. Emit `block_indent()` IR nodes that the printer resolves to the configured width.

#### 1.6 Testing Strategy (Phase 1)

- **Snapshot tests** using `cargo insta`. Create `tests/specs/` directory with `.yaml` input files and `.snap` expected outputs.
- **Minimum test cases:** Single mapping, nested mapping, single sequence, nested sequence, mixed mapping+sequence, comments in each position, empty document.
- **Round-trip property:** `format(format(input)) == format(input)` — idempotency test.

### Phase 2: Advanced — Complex Nodes and Multi-Document

Goal: Handle all YAML constructs that appear in real-world files.

#### 2.1 Block Scalars

| Node | IR Strategy | Key Concern |
|------|-------------|-------------|
| `YamlBlockLiteralScalar` (`\|`) | Preserve content verbatim; format only the indicator line | Content indentation is relative to indicator, not parent |
| `YamlBlockFoldedScalar` (`>`) | Preserve content verbatim; format only the indicator line | Folding semantics must not change |

**Chomping indicators:** `-` (strip), `+` (keep), default (clip). The formatter must preserve the chosen indicator. Do not add or remove chomping indicators.

**Indentation indicators:** Explicit digit after `|` or `>` (e.g., `|2`). Preserve as-is.

#### 2.2 Anchors and Aliases

| Node | IR Strategy | Notes |
|------|-------------|-------|
| `YamlAnchorProperty` (`&name`) | `text()` before the value node | Space between anchor and value |
| `YamlAliasValue` (`*name`) | `text()` verbatim | Never transform anchor names |

**Merge keys:** `<<: *alias` is a YAML 1.1 pattern. Format as a regular mapping entry. The formatter does not interpret merge semantics.

#### 2.3 Tags

| Node | IR Strategy | Notes |
|------|-------------|-------|
| `YamlTagProperty` (`!!str`, `!custom`) | `text()` before the value node | Space between tag and value |
| `YamlVerbatimTag` (`!<tag:uri>`) | `text()` verbatim | Never modify tag URIs |

#### 2.4 Multi-Document Formatting

- Insert `hard_line_break()` before each `---` marker.
- Preserve `...` document end markers if present; do not add or remove them.
- Format `%YAML` and `%TAG` directives at the top of the file, before the first `---`.
- Each document is formatted independently (no cross-document indentation inheritance).

#### 2.5 YAML-Specific Options (Phase 2)

| Option | Type | Default | Research Reference |
|--------|------|---------|-------------------|
| `yaml_document_start` | `YamlDocumentStart` enum (require/omit/preserve) | `preserve` | yamllint: `present: true`, prettier: preserve, yamlfix: `true` |
| `yaml_sequence_indent` | `YamlSequenceIndent` (u8, 0–24) | same as `indent_width` | yamlfmt: `array_indent`, yamlfix: `indent_sequence: 4` |

#### 2.6 Testing Strategy (Phase 2)

- **Block scalar tests:** Literal, folded, with each chomping indicator, with explicit indentation indicator.
- **Anchor/alias tests:** Definition, usage, unused anchor, multiple aliases to same anchor.
- **Multi-document tests:** 2-document file, directives, `---` only, `---` + `...`.
- **Kubernetes-style tests:** Real-world YAML files (deployments, services, configmaps) as integration tests.

### Phase 3: Edge Cases — Spec-Sensitive and Rare Constructs

Goal: Handle all valid YAML, including constructs that differ between spec versions.

#### 3.1 Mixed Flow and Block Styles

Flow collections inside block context and vice versa:
```yaml
block_key:
  - {nested: flow, inside: block}
  - [mixed, styles]
```

The formatter should preserve the author's style choice (flow vs block) by default. A future option (`yaml_collection_style: block/flow/preserve`) could force conversion, but this is deferred.

#### 3.2 Special-Character Scalars

Scalars that require quoting due to special characters:
- Scalars starting with `{`, `[`, `*`, `&`, `!`, `|`, `>`, `'`, `"`, `%`, `@`, `` ` ``
- Scalars containing `: ` or ` #`
- Empty string (`""` or `''`)
- Null indicators (`~`, `null`)

The formatter must not strip necessary quotes. When applying `yaml_quote_style`, verify the scalar remains valid in the target style.

#### 3.3 YAML 1.1 Boolean Ambiguity

Scalars like `yes`, `no`, `on`, `off`, `y`, `n` are booleans in YAML 1.1 but strings in 1.2. The formatter does not interpret scalar types — it preserves the original text. However, if `yaml_quote_style` forces quoting, these values become unambiguous strings. This interaction should be documented but not special-cased in the formatter.

#### 3.4 Testing Strategy (Phase 3)

- **Round-trip fuzz testing:** Generate random valid YAML and verify `parse(format(input))` produces the same AST as `parse(input)`.
- **Spec compliance tests:** Port relevant test cases from the YAML Test Suite (https://github.com/yaml/yaml-test-suite).

---

## Layer 6: Analyzer

**Crate:** `crates/biome_yaml_analyze/`
**Reference:** `crates/biome_json_analyze/`
**Key traits:** `Rule`, `RuleMeta`, `declare_lint_rule!`

### Crate Skeleton

Create `crates/biome_yaml_analyze/` with:

| File | Purpose | Reference |
|------|---------|-----------|
| `Cargo.toml` | Depends on `biome_analyze`, `biome_yaml_syntax` | `crates/biome_json_analyze/Cargo.toml` |
| `src/lib.rs` | `analyze()` entry point, `YamlAnalyzeServices`, suppression parsing | `crates/biome_json_analyze/src/lib.rs` |
| `src/lint/` | Rule implementations by category | `crates/biome_json_analyze/src/lint/` |
| `src/lint/suspicious/` | Rules catching potential bugs | — |
| `src/lint/correctness/` | Rules catching definite errors | — |
| `src/lint/style/` | Rules enforcing style conventions | — |

### Phase 1: Tier 1 Rules — Consensus + High-Impact

10 rules implementing features found in 5+ tools (Tier 1 from research report). These are the minimum viable analyzer.

---

#### Rule 1: `noKeyDuplicates`

| Field | Value |
|-------|-------|
| **Category** | `correctness` |
| **Severity** | `error` |
| **Recommended** | `true` |
| **What it checks** | Duplicate keys within the same mapping |
| **Spec basis** | Spec-mandated (YAML 1.2 §3.2.1.3: "mapping keys are unique") |
| **Config options** | `forbidDuplicatedMergeKeys: bool` (default: `false`) — whether `<<` merge keys count as duplicates |
| **Edge cases** | Nested mappings: only check siblings, not ancestors. Flow mappings: same rule applies. Multi-document: scope per-document. |
| **Fixable** | No (ambiguous which duplicate to keep) |
| **Reference** | yamllint `key-duplicates`, yamllint-rs `key-duplicates`, yaml-lint-rs `DuplicateKey` |
| **Target file** | `src/lint/correctness/no_key_duplicates.rs` |

---

#### Rule 2: `useConsistentIndentation`

| Field | Value |
|-------|-------|
| **Category** | `style` |
| **Severity** | `error` |
| **Recommended** | `true` |
| **What it checks** | Consistent indentation width and style (spaces vs tabs) |
| **Spec basis** | Spec-mandated (YAML requires spaces for indentation, tabs are not allowed) |
| **Config options** | `spaces: u8 \| "consistent"` (default: `"consistent"`) — expected indent width. `indentSequences: bool` (default: `true`) — whether sequence items add an indent level. |
| **Edge cases** | Block scalar content uses its own indentation context. Flow collections ignore indentation rules. Empty lines are exempt. |
| **Fixable** | Yes (safe fix — adjust whitespace) |
| **Reference** | yamllint `indentation` (3/3 linters), all 3 formatters |
| **Target file** | `src/lint/style/use_consistent_indentation.rs` |

---

#### Rule 3: `noTrailingSpaces`

| Field | Value |
|-------|-------|
| **Category** | `style` |
| **Severity** | `error` |
| **Recommended** | `true` |
| **What it checks** | Trailing whitespace at end of lines |
| **Spec basis** | Tool-opinion (spec is silent on trailing whitespace) |
| **Config options** | None |
| **Edge cases** | Block scalar content: trailing spaces may be significant in literal blocks. Consider exempting `\|` block content. |
| **Fixable** | Yes (safe fix — remove trailing whitespace, except in block scalars) |
| **Reference** | yamllint `trailing-spaces` (3/3 linters), yamlfmt `trim_trailing_whitespace`, prettier always trims |
| **Target file** | `src/lint/style/no_trailing_spaces.rs` |

---

#### Rule 4: `useLineEndingStyle`

| Field | Value |
|-------|-------|
| **Category** | `style` |
| **Severity** | `error` |
| **Recommended** | `true` |
| **What it checks** | Consistent line endings (LF vs CRLF) |
| **Spec basis** | Tool-opinion (spec normalizes line breaks during processing) |
| **Config options** | `type: "lf" \| "crlf"` (default: `"lf"`) |
| **Edge cases** | Mixed line endings in a single file should all be flagged. Block scalar content should use the same line ending. |
| **Fixable** | Yes (safe fix — normalize line endings) |
| **Reference** | yamllint `new-lines` (2/3 linters), prettier `endOfLine`, yamlfmt `line_ending` |
| **Target file** | `src/lint/style/use_line_ending_style.rs` |

---

#### Rule 5: `useFileEndNewline`

| Field | Value |
|-------|-------|
| **Category** | `style` |
| **Severity** | `error` |
| **Recommended** | `true` |
| **What it checks** | File ends with a newline character |
| **Spec basis** | Tool-opinion (POSIX convention, not YAML spec) |
| **Config options** | None |
| **Edge cases** | Empty files (0 bytes) are exempt. |
| **Fixable** | Yes (safe fix — append newline) |
| **Reference** | yamllint `new-line-at-end-of-file` (3/3 linters), prettier always, yamlfix always |
| **Target file** | `src/lint/style/use_file_end_newline.rs` |

---

#### Rule 6: `noTruthyStrings`

| Field | Value |
|-------|-------|
| **Category** | `suspicious` |
| **Severity** | `warning` |
| **Recommended** | `true` |
| **What it checks** | Plain scalars that are boolean-like in YAML 1.1 but strings in 1.2 (`yes`, `no`, `on`, `off`, `y`, `n` and case variants) |
| **Spec basis** | Spec-divergence (YAML 1.1 has 22 boolean values, 1.2 has 6) |
| **Config options** | `allowedValues: string[]` (default: `["true", "false"]`) — values that are NOT flagged. `checkKeys: bool` (default: `true`) — also check mapping keys. |
| **Edge cases** | Quoted scalars are never flagged (`"yes"` is unambiguously a string). Values in `allowedValues` are case-sensitive. |
| **Fixable** | Yes (unsafe fix — quote the value, changing it from boolean to string in 1.1 parsers) |
| **Reference** | yamllint `truthy` (3/3 linters), yamlfix truthy normalization |
| **Target file** | `src/lint/suspicious/no_truthy_strings.rs` |

---

#### Rule 7: `useExplicitLineLength`

| Field | Value |
|-------|-------|
| **Category** | `style` |
| **Severity** | `warning` |
| **Recommended** | `false` |
| **What it checks** | Lines exceeding a maximum length |
| **Spec basis** | Tool-opinion (no spec-defined line length limit) |
| **Config options** | `max: u16` (default: `80`). `allowNonBreakableWords: bool` (default: `true`) — exempt lines with a single long token (URLs, hashes). |
| **Edge cases** | Comment-only lines should be checked. Block scalar content lines should be checked unless the scalar has a `>` fold indicator (folded scalars are expected to be reformatted). |
| **Fixable** | No (line breaking changes semantics in many contexts) |
| **Reference** | yamllint `line-length` (3/3 linters), prettier `printWidth`, yamlfix `line_length` |
| **Target file** | `src/lint/style/use_explicit_line_length.rs` |

---

#### Rule 8: `useConsistentColonSpacing`

| Field | Value |
|-------|-------|
| **Category** | `style` |
| **Severity** | `error` |
| **Recommended** | `true` |
| **What it checks** | Spacing around colons in mapping entries |
| **Spec basis** | Spec-ambiguity (spec requires space after `:` in block context but is permissive in flow) |
| **Config options** | `maxSpacesBefore: u8` (default: `0`). `maxSpacesAfter: u8` (default: `1`). |
| **Edge cases** | Flow mappings (`{a: 1}`) vs block mappings (`a: 1`) — same rules apply. Multi-line keys use `: ` on the next line. |
| **Fixable** | Yes (safe fix — adjust whitespace around colons) |
| **Reference** | yamllint `colons` (3/3 linters) |
| **Target file** | `src/lint/style/use_consistent_colon_spacing.rs` |

---

#### Rule 9: `useConsistentHyphenSpacing`

| Field | Value |
|-------|-------|
| **Category** | `style` |
| **Severity** | `error` |
| **Recommended** | `true` |
| **What it checks** | Spacing after hyphens (`-`) in block sequence entries |
| **Spec basis** | Spec-ambiguity (spec requires separation but doesn't constrain spacing) |
| **Config options** | `maxSpacesAfter: u8` (default: `1`) |
| **Edge cases** | Compact notation (`-item` with no space) is invalid YAML — always flag. |
| **Fixable** | Yes (safe fix — normalize to single space) |
| **Reference** | yamllint `hyphens` (3/3 linters) |
| **Target file** | `src/lint/style/use_consistent_hyphen_spacing.rs` |

---

#### Rule 10: `noExcessiveEmptyLines`

| Field | Value |
|-------|-------|
| **Category** | `style` |
| **Severity** | `warning` |
| **Recommended** | `true` |
| **What it checks** | Too many consecutive empty lines |
| **Spec basis** | Tool-opinion (spec allows arbitrary empty lines) |
| **Config options** | `max: u8` (default: `2`). `maxStart: u8` (default: `0`) — max empty lines at file start. `maxEnd: u8` (default: `0`) — max empty lines at file end. |
| **Edge cases** | Empty lines inside block scalars are content, not structure — exempt. |
| **Fixable** | Yes (safe fix — remove excess empty lines) |
| **Reference** | yamllint `empty-lines` (3/3 linters), yamlfmt `retain_line_breaks` |
| **Target file** | `src/lint/style/no_excessive_empty_lines.rs` |

---

### Phase 2: Tier 2 Rules — Common Features

8 rules implementing features found in 2–4 tools (Tier 2 from research report).

---

#### Rule 11: `useSortedKeys`

| Field | Value |
|-------|-------|
| **Category** | `style` |
| **Severity** | `warning` |
| **Recommended** | `false` |
| **What it checks** | Mapping keys sorted alphabetically |
| **Spec basis** | Tool-opinion (neither spec defines key ordering) |
| **Config options** | `ignoredKeys: string[]` (default: `[]`) — regex patterns for keys to skip. |
| **Edge cases** | Nested mappings: check each level independently. Comment blocks between keys should move with their key. |
| **Fixable** | Yes (unsafe fix — reordering keys may change semantics in ordered consumers) |
| **Reference** | yamllint `key-ordering` (2/3 linters) |
| **Target file** | `src/lint/style/use_sorted_keys.rs` |

---

#### Rule 12: `useConsistentCommentSpacing`

| Field | Value |
|-------|-------|
| **Category** | `style` |
| **Severity** | `warning` |
| **Recommended** | `true` |
| **What it checks** | Space between `#` and comment text; minimum spaces between content and inline comment |
| **Spec basis** | Tool-opinion (spec only defines `#` as comment start) |
| **Config options** | `requireStartingSpace: bool` (default: `true`). `minSpacesFromContent: u8` (default: `2`). |
| **Edge cases** | Shebangs (`#!/...`) are not YAML comments — exempt. Empty comments (`#` with no text) are allowed. |
| **Fixable** | Yes (safe fix — add spaces) |
| **Reference** | yamllint `comments` (3/3 linters) |
| **Target file** | `src/lint/style/use_consistent_comment_spacing.rs` |

---

#### Rule 13: `useConsistentCommentIndentation`

| Field | Value |
|-------|-------|
| **Category** | `style` |
| **Severity** | `warning` |
| **Recommended** | `false` |
| **What it checks** | Comments indented at the same level as the content they precede |
| **Spec basis** | Tool-opinion |
| **Config options** | None |
| **Edge cases** | Top-level comments (column 0) are always valid. Comments after the last item in a block may align with the parent or the sibling. |
| **Fixable** | Yes (safe fix — adjust comment indentation) |
| **Reference** | yamllint `comments-indentation` (2/3 linters) |
| **Target file** | `src/lint/style/use_consistent_comment_indentation.rs` |

---

#### Rule 14: `useConsistentBraceSpacing`

| Field | Value |
|-------|-------|
| **Category** | `style` |
| **Severity** | `warning` |
| **Recommended** | `false` |
| **What it checks** | Spacing inside flow mapping braces `{ }` |
| **Spec basis** | Tool-opinion (spec doesn't constrain spacing inside indicators) |
| **Config options** | `minSpacesInside: u8` (default: `0`). `maxSpacesInside: u8` (default: `0`). `forbid: bool` (default: `false`) — forbid flow mappings entirely. |
| **Edge cases** | Empty mappings (`{}`) should have 0 spaces regardless of config. Nested flow collections: apply recursively. |
| **Fixable** | Yes (safe fix — adjust spaces) |
| **Reference** | yamllint `braces` (2/3 linters), prettier `bracketSpacing` |
| **Target file** | `src/lint/style/use_consistent_brace_spacing.rs` |

---

#### Rule 15: `useConsistentBracketSpacing`

| Field | Value |
|-------|-------|
| **Category** | `style` |
| **Severity** | `warning` |
| **Recommended** | `false` |
| **What it checks** | Spacing inside flow sequence brackets `[ ]` |
| **Spec basis** | Tool-opinion |
| **Config options** | `minSpacesInside: u8` (default: `0`). `maxSpacesInside: u8` (default: `0`). `forbid: bool` (default: `false`) — forbid flow sequences entirely. |
| **Edge cases** | Empty sequences (`[]`) should have 0 spaces regardless. |
| **Fixable** | Yes (safe fix — adjust spaces) |
| **Reference** | yamllint `brackets` (2/3 linters) |
| **Target file** | `src/lint/style/use_consistent_bracket_spacing.rs` |

---

#### Rule 16: `useConsistentCommaSpacing`

| Field | Value |
|-------|-------|
| **Category** | `style` |
| **Severity** | `warning` |
| **Recommended** | `false` |
| **What it checks** | Spacing around commas in flow collections |
| **Spec basis** | Tool-opinion |
| **Config options** | `maxSpacesBefore: u8` (default: `0`). `minSpacesAfter: u8` (default: `1`). `maxSpacesAfter: u8` (default: `1`). |
| **Edge cases** | Only applies to flow collections (block style doesn't use commas). Trailing commas after the last element. |
| **Fixable** | Yes (safe fix — adjust spaces) |
| **Reference** | yamllint `commas` (2/3 linters) |
| **Target file** | `src/lint/style/use_consistent_comma_spacing.rs` |

---

#### Rule 17: `noEmptyValues`

| Field | Value |
|-------|-------|
| **Category** | `suspicious` |
| **Severity** | `warning` |
| **Recommended** | `false` |
| **What it checks** | Mapping entries with no value (`key:` with nothing after the colon) |
| **Spec basis** | Tool-opinion (both specs allow empty values, interpreted as null) |
| **Config options** | `forbidInBlockMappings: bool` (default: `true`). `forbidInFlowMappings: bool` (default: `true`). |
| **Edge cases** | Explicit null (`key: null`, `key: ~`) should NOT be flagged — only truly empty values. |
| **Fixable** | No (ambiguous whether to insert `null`, `~`, or remove the key) |
| **Reference** | yamllint `empty-values` (2/3 linters) |
| **Target file** | `src/lint/suspicious/no_empty_values.rs` |

---

#### Rule 18: `useConsistentDocumentMarkers`

| Field | Value |
|-------|-------|
| **Category** | `style` |
| **Severity** | `warning` |
| **Recommended** | `false` |
| **What it checks** | Presence or absence of `---` document start markers |
| **Spec basis** | Spec-ambiguity (markers are optional for single-document files) |
| **Config options** | `documentStart: "require" \| "forbid" \| "preserve"` (default: `"preserve"`). `documentEnd: "require" \| "forbid" \| "preserve"` (default: `"preserve"`). |
| **Edge cases** | Multi-document files always need `---` between documents. `%YAML`/`%TAG` directives require `---` after them. |
| **Fixable** | Yes (unsafe fix — adding/removing markers may affect multi-doc consumers) |
| **Reference** | yamllint `document-start`/`document-end` (3/3 linters) |
| **Target file** | `src/lint/style/use_consistent_document_markers.rs` |

---

### Phase 3: Tier 3 Rules — Valuable Features

4 rules implementing features found in 2 tools or addressing spec-divergence concerns.

---

#### Rule 19: `noOctalValues`

| Field | Value |
|-------|-------|
| **Category** | `suspicious` |
| **Severity** | `warning` |
| **Recommended** | `false` |
| **What it checks** | Implicit C-style octal values (`0777`) that are integers in YAML 1.1 but strings in 1.2 |
| **Spec basis** | Spec-divergence (YAML 1.1 `0777` = octal, YAML 1.2 requires `0o777`) |
| **Config options** | `forbidImplicitOctal: bool` (default: `true`) — flag `0777`. `forbidExplicitOctal: bool` (default: `false`) — flag `0o777`. |
| **Edge cases** | `0` alone is not octal. `00` is ambiguous. Quoted values (`"0777"`) are never flagged. |
| **Fixable** | Yes (unsafe fix — convert `0777` to `0o777` or quote it) |
| **Reference** | yamllint `octal-values` (2/3 linters) |
| **Target file** | `src/lint/suspicious/no_octal_values.rs` |

---

#### Rule 20: `useValidFloatValues`

| Field | Value |
|-------|-------|
| **Category** | `suspicious` |
| **Severity** | `warning` |
| **Recommended** | `false` |
| **What it checks** | Float values with ambiguous or non-standard representations |
| **Spec basis** | Spec-divergence (1.1 and 1.2 differ on `.inf`, `.Inf`, `.nan`, `.NaN`) |
| **Config options** | `forbidNan: bool` (default: `false`). `forbidInf: bool` (default: `false`). `forbidScientificNotation: bool` (default: `false`). |
| **Edge cases** | Case sensitivity: `.nan` vs `.NaN` vs `.NAN`. Only applies to plain scalars. |
| **Fixable** | No (semantically ambiguous which representation is intended) |
| **Reference** | yamllint `float-values` (2/3 linters) |
| **Target file** | `src/lint/suspicious/use_valid_float_values.rs` |

---

#### Rule 21: `noUnusedAnchors`

| Field | Value |
|-------|-------|
| **Category** | `correctness` |
| **Severity** | `warning` |
| **Recommended** | `true` |
| **What it checks** | Anchors (`&name`) that are never referenced by an alias (`*name`) |
| **Spec basis** | Tool-opinion (unused anchors are valid YAML but likely dead code) |
| **Config options** | None |
| **Edge cases** | Must scan the entire document for aliases before reporting. Multi-document: anchors are scoped per-document. `<<` merge keys with `*alias` count as usage. |
| **Fixable** | Yes (unsafe fix — remove the anchor, which may affect external consumers) |
| **Reference** | yamllint `anchors` (2/3 linters) |
| **Target file** | `src/lint/correctness/no_unused_anchors.rs` |

---

#### Rule 22: `useConsistentQuoteStyle`

| Field | Value |
|-------|-------|
| **Category** | `style` |
| **Severity** | `warning` |
| **Recommended** | `false` |
| **What it checks** | Consistent use of quote style for string scalars |
| **Spec basis** | Tool-opinion (spec allows all scalar styles freely) |
| **Config options** | `quoteStyle: "single" \| "double" \| "preserve"` (default: `"preserve"`). `requireQuotesForSpecialValues: bool` (default: `false`) — require quotes for values that could be misinterpreted (truthy, numbers). |
| **Edge cases** | Some strings cannot be single-quoted (those containing `'`). Some strings cannot be plain (those starting with special characters). The rule must not suggest an invalid transformation. |
| **Fixable** | Yes (safe fix when target style is valid for the value; skip otherwise) |
| **Reference** | yamllint `quoted-strings` (2/3 linters), prettier `singleQuote`, yamlfmt `force_quote_style` |
| **Target file** | `src/lint/style/use_consistent_quote_style.rs` |

---

### Suppression Comments

YAML uses `#` for comments. Biome suppression comments follow the `biome-ignore` pattern.

#### Syntax

```yaml
# biome-ignore lint/correctness/noKeyDuplicates: known duplicate for compatibility
duplicate_key: value1
duplicate_key: value2
```

- Prefix: `# biome-ignore` (must be a standalone comment, not end-of-line)
- Rule path: `lint/{category}/{ruleName}` (standard Biome rule path)
- Separator: `: ` (colon + space)
- Reason: Free text explanation (required)

#### Parsing Strategy

In `src/lib.rs`, implement suppression parsing in the `analyze()` function:

1. Walk all trivia tokens of kind `COMMENT`.
2. Strip the `#` prefix and leading whitespace.
3. Check for `biome-ignore` prefix.
4. Parse the rule path and reason.
5. Associate the suppression with the next non-trivia sibling node.

#### `YamlSuppressionAction`

Implement the `SuppressionAction` trait to handle `biome-ignore` comments:

| Method | Behavior |
|--------|----------|
| `find_token_to_apply_suppression` | Find the comment token preceding the diagnosed node |
| `apply_suppression` | Insert `# biome-ignore {rule}: {reason}` as a new comment line before the node |

**Reference:** `crates/biome_json_analyze/src/lib.rs` — JSON's suppression implementation. YAML's is identical in structure, differing only in comment syntax (`#` vs `//`).

---

## Layer 7: Service Integration

**Location:** `crates/biome_service/`
**Reference:** `crates/biome_service/src/file_handlers/json.rs`

### 7.1 `DocumentFileSource`

Add YAML variant to the `DocumentFileSource` enum at `crates/biome_service/src/file_handlers/mod.rs:78`:

```rust
pub enum DocumentFileSource {
    // ... existing variants ...
    Yaml(YamlFileSource),
}
```

Implement:
- `From<YamlFileSource> for DocumentFileSource`
- Wire `try_from_extension()`: `.yaml`, `.yml`, and all extensions from `YamlFileSource` (14+ total)
- Wire `try_from_file_path()`: well-known files (`.yamllint`, `.prettierrc.yaml`, etc.)
- Wire `try_from_language_id()`: `"yaml"` language ID for LSP

### 7.2 `YamlFileHandler`

Create `crates/biome_service/src/file_handlers/yaml.rs`:

```rust
pub(crate) struct YamlFileHandler;

impl ExtensionHandler for YamlFileHandler {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            parser: ParserCapabilities {
                parse: Some(parse),
            },
            debug: DebugCapabilities {
                debug_syntax_tree: Some(debug_syntax_tree),
                debug_control_flow: None,
                debug_formatter_ir: Some(debug_formatter_ir),
            },
            analyzer: AnalyzerCapabilities {
                lint: Some(lint),
                code_actions: Some(code_actions),
                rename: None,
                fix_all: Some(fix_all),
                organize_imports: None,
            },
            formatter: FormatterCapabilities {
                format: Some(format),
                format_range: Some(format_range),
                format_on_type: None,
            },
            search: SearchCapabilities::default(),
            enabled_for_path: EnabledForPath::default(),
        }
    }
}
```

### 7.3 Capability Functions

Implement these standalone functions in `yaml.rs`:

| Function | Signature Pattern | What It Does |
|----------|------------------|--------------|
| `parse` | `fn(&BiomePath, DocumentFileSource, &str, &Settings, &mut NodeCache) -> ParseResult` | Calls `parse_yaml()`, converts to `AnyParse` |
| `format` | `fn(&BiomePath, DocumentFileSource, &str, &Settings, &mut NodeCache) -> Result<Printed, WorkspaceError>` | Parses then formats using `YamlFormatLanguage` |
| `format_range` | Same + `TextRange` | Formats a sub-range of the document |
| `lint` | `fn(&BiomePath, DocumentFileSource, &str, &Settings, &mut NodeCache) -> PullDiagnosticsResult` | Parses then runs `analyze()` from `biome_yaml_analyze` |
| `code_actions` | `fn(&BiomePath, DocumentFileSource, &str, &Settings, &mut NodeCache, TextRange) -> PullActionsResult` | Returns code actions for a cursor range |
| `fix_all` | `fn(&BiomePath, DocumentFileSource, &str, &Settings, &mut NodeCache) -> FixFileResult` | Applies all safe fixes |
| `debug_syntax_tree` | `fn(&BiomePath, DocumentFileSource, &str, &Settings, &mut NodeCache) -> String` | Returns debug representation of the CST |
| `debug_formatter_ir` | `fn(&BiomePath, DocumentFileSource, &str, &Settings, &mut NodeCache) -> String` | Returns debug representation of the formatter IR |

### 7.4 `ServiceLanguage for YamlLanguage`

Implement in `crates/biome_service/src/settings.rs` or `yaml.rs`:

```rust
impl ServiceLanguage for YamlLanguage {
    type FormatterSettings = YamlFormatterSettings;
    type LinterSettings = YamlLinterSettings;
    type AssistSettings = YamlAssistSettings;
    type FormatOptions = YamlFormatOptions;
    type ParserSettings = YamlParserSettings;
    type ParserOptions = YamlParserOptions;
    type EnvironmentSettings = ();

    fn lookup_settings(languages: &LanguageListSettings) -> &LanguageSettings<Self> {
        &languages.yaml
    }

    fn resolve_format_options(
        global: &FormatSettings,
        overrides: &OverrideSettings,
        language: &Self::FormatterSettings,
        path: &BiomePath,
    ) -> Self::FormatOptions {
        // Merge global → language → override settings
    }
}
```

### 7.5 Settings Types

Create these settings structs:

| Struct | Fields | Location |
|--------|--------|----------|
| `YamlFormatterSettings` | `quote_style`, `document_start`, `sequence_indent` (all `Option<T>`) | `yaml.rs` or dedicated settings file |
| `YamlLinterSettings` | (empty initially, placeholder for future per-rule defaults) | Same |
| `YamlAssistSettings` | (empty initially) | Same |
| `YamlParserSettings` | (empty initially, future: `yaml_version`) | Same |
| `YamlParserOptions` | (empty initially) | Same |

### 7.6 `LanguageListSettings` Wiring

Add `yaml: LanguageSettings<YamlLanguage>` field to `LanguageListSettings` struct at `crates/biome_service/src/settings.rs`.

Wire up `biome.json` configuration:
```json
{
  "yaml": {
    "formatter": {
      "quoteStyle": "double",
      "documentStart": "preserve"
    },
    "linter": {
      "rules": {
        "correctness": {
          "noKeyDuplicates": "error"
        }
      }
    }
  }
}
```

### 7.7 `Features` Struct

Add `YamlFileHandler` to the `Features` struct at `crates/biome_service/src/file_handlers/mod.rs:1022`:

```rust
pub(crate) struct Features {
    // ... existing handlers ...
    yaml: YamlFileHandler,
}
```

Wire up the `capabilities()` dispatch to return `yaml.capabilities()` when the document source is `DocumentFileSource::Yaml(_)`.

---

## Implementation Order

```
                    ┌──────────────────────┐
                    │  Layer 5: Formatter   │
                    │  Phase 1 → 2 → 3     │
                    └──────────┬───────────┘
                               │
                               │  (independent)
                               │
                    ┌──────────┴───────────┐
                    │  Layer 6: Analyzer    │
                    │  Phase 1 → 2 → 3     │
                    └──────────┬───────────┘
                               │
                               │  (both required)
                               ▼
                    ┌──────────────────────┐
                    │  Layer 7: Service     │
                    │  Integration          │
                    └──────────────────────┘
```

**Recommended sequence:**

1. **Layer 5, Phase 1** (Formatter MVP) — Crate skeleton, core options, basic node formatting, comment handling. This is the critical path because comment handling design informs all subsequent formatting.
2. **Layer 6, Phase 1** (Tier 1 Rules) — Start in parallel with formatter Phase 1. The analyzer only needs the parser (Layer 4), not the formatter.
3. **Layer 7** (Service Integration) — Can begin as soon as formatter Phase 1 and analyzer Phase 1 produce compilable crates, even before all rules/formatting is complete. Initially wire up parse-only capabilities, then add format and lint as they stabilize.
4. **Layer 5, Phase 2 + Layer 6, Phase 2** — Continue in parallel after service integration is wired up (enables end-to-end testing).
5. **Layer 5, Phase 3 + Layer 6, Phase 3** — Final polish, edge cases, spec compliance.

**Critical path:** Formatter comment handling → Formatter Phase 1 complete → Service integration wiring → End-to-end testing possible.

---

## Testing Strategy

### Layer 5: Formatter

| Test Type | Tool | Directory | When |
|-----------|------|-----------|------|
| Snapshot tests | `cargo insta` | `crates/biome_yaml_formatter/tests/specs/` | Every node type |
| Idempotency tests | Custom harness | Same directory | Phase 1 complete |
| Round-trip tests | Custom harness | Same directory | Phase 2 complete |
| Spec compliance | YAML Test Suite | `crates/biome_yaml_formatter/tests/yaml-test-suite/` | Phase 3 |

**Snapshot test pattern:** Each `.yaml` input file gets a `.snap` file with the formatted output. Run `cargo insta review` to accept changes.

### Layer 6: Analyzer

| Test Type | Tool | Directory | When |
|-----------|------|-----------|------|
| Per-rule snapshot tests | `cargo insta` | `crates/biome_yaml_analyze/tests/specs/{category}/{rule_name}/` | Every rule |
| Invalid case tests | Same | `{rule_name}/invalid.yaml` | Every rule |
| Valid case tests | Same | `{rule_name}/valid.yaml` | Every rule |
| Fix tests | Same | `{rule_name}/fix.yaml` + `.snap` | Fixable rules |
| Suppression tests | Same | `tests/specs/suppression/` | After suppression impl |

**Per-rule test pattern:** Each rule gets `valid.yaml` (should produce no diagnostics) and `invalid.yaml` (should produce diagnostics at marked locations). Fixable rules also get a fix snapshot.

### Layer 7: Service Integration

| Test Type | Tool | Directory | When |
|-----------|------|-----------|------|
| Integration tests | `cargo test` | `crates/biome_service/tests/` | After wiring |
| CLI tests | `cargo test` | `crates/biome_cli/tests/` | After wiring |
| End-to-end | Manual + CI | Project root | After all layers |

**E2E test pattern:** Create a sample project with YAML files and run `biome check`, `biome format`, `biome lint` against it. Verify output matches expectations.

---

## Open Questions

These decisions are deferred to the implementation phase. They require hands-on experimentation to resolve.

### Formatter

1. **Flow ↔ block conversion:** Should the formatter ever convert between flow and block styles? Current spec says preserve, but `yaml_collection_style` option is a natural extension.
2. **Comment alignment:** Should consecutive end-of-line comments be aligned to the same column? Desirable but complex to implement in IR.
3. **Key quoting:** Should the formatter add quotes to keys that could be misinterpreted (e.g., `true`, `null`, `3.14`)? This overlaps with the linter's `noTruthyStrings` rule.

### Analyzer

4. **YAML version detection:** Should rules like `noTruthyStrings` and `noOctalValues` change behavior based on `%YAML 1.1` vs `%YAML 1.2` directives? The research says yes (yamllint does this), but implementation complexity is non-trivial.
5. **Suppression comment position:** Should `# biome-ignore` work as an end-of-line comment (`key: value # biome-ignore ...`) or only as an own-line comment? JSON uses `// biome-ignore` on its own line. YAML should follow the same pattern for consistency.
6. **Key ordering beyond alphabetical:** Should `useSortedKeys` support custom orderings (e.g., Kubernetes conventional order)? Defer to Phase 3 or later.

### Service

7. **Parser options exposure:** Should `biome.json` expose YAML parser options (e.g., `yaml_version: "1.1" | "1.2"`)? This affects how the parser interprets scalars. Defer until parser supports version-aware scanning.
8. **Format-on-type triggers:** Which characters should trigger `format_on_type` for YAML? Candidates: `\n` (newline), `:` (colon in mappings). Low priority.

---

## Deferred Features

These features are identified in the research report but explicitly out of scope for this spec:

| Feature | Reason for Deferral |
|---------|-------------------|
| JSON Schema validation | Requires schema registry infrastructure, separate from core lint/format |
| Schema Store integration | Depends on JSON Schema validation |
| Kubernetes-specific rules | Domain-specific, not part of core YAML support |
| Jinja2/Ansible template handling | Domain-specific preprocessing |
| Merge key (`<<`) semantic analysis | YAML 1.1 feature, complex to implement correctly |
| Embedded language formatting | Would require formatter plugin architecture |
| `format_on_type` | Low priority; requires analysis of useful trigger points |
| Custom key ordering strategies | Complex configuration; basic alphabetical is sufficient initially |
