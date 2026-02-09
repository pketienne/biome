# Plan 16: Expand Lint Rule Coverage

## Status: COMPLETE

## Goal

Add 5 new high-value lint rules that catch real bugs and improve consistency, bringing the total from 23 to 28.

## Current State

23 rules covering core bug prevention and style. Gap analysis says "100% coverage of identified rules." However, comparison with yamllint and eslint-plugin-yml reveals these additional high-value rules:

## New Rules

### 16A: `noEmptyKeys` — Disallow empty mapping keys

**Catches**: Accidental `: value` at the start of a line (key is null/empty).

```yaml
# Invalid
: value
"": also bad

# Valid
key: value
```

**Query**: `Ast<YamlBlockMapping>` — iterate entries, check if key slot is empty or key text is empty string.

**File**: `crates/biome_yaml_analyze/src/lint/nursery/no_empty_keys.rs`

### 16B: `noEmptySequenceEntries` — Disallow empty sequence entries

**Catches**: Flow sequences with missing entries like `[1, , 3]`.

```yaml
# Invalid
items: [1, , 3]

# Valid
items: [1, 2, 3]
```

**Query**: `Ast<YamlFlowSequence>` — iterate entries, check for empty/missing values.

**File**: `crates/biome_yaml_analyze/src/lint/nursery/no_empty_sequence_entries.rs`

### 16C: `useConsistentIndentation` — Enforce consistent indentation width

**Catches**: Mixed 2-space and 4-space indentation in the same file.

```yaml
# Invalid (mixed 2 and 4 space)
parent:
  child:
      grandchild: value

# Valid (consistent 2 space)
parent:
  child:
    grandchild: value
```

**Query**: `Ast<YamlRoot>` — scan all lines for leading spaces, detect the base indent unit, flag deviations.

**File**: `crates/biome_yaml_analyze/src/lint/nursery/use_consistent_indentation.rs`

### 16D: `noAnchorReferences` — Forbid anchors and aliases (strict mode)

**Catches**: Teams that want to ban anchors/aliases for clarity (merge keys can make YAML hard to read).

```yaml
# Invalid
defaults: &defaults
  timeout: 30
production:
  <<: *defaults

# Valid
defaults:
  timeout: 30
production:
  timeout: 30
```

**Query**: `Ast<YamlRoot>` — check for any `YamlAnchorProperty` or `YamlAliasNode` in the tree.

**File**: `crates/biome_yaml_analyze/src/lint/nursery/no_anchor_references.rs`

### 16E: `useQuotedStrings` — Require all string values to be quoted

**Catches**: Unquoted strings that could be ambiguous (e.g., `yes`, `null`, `3.14` when intended as strings).

```yaml
# Invalid
name: John
status: active

# Valid
name: "John"
status: "active"
```

**Query**: `Ast<YamlPlainScalar>` — flag any plain scalar that appears as a mapping value (not a key).

**File**: `crates/biome_yaml_analyze/src/lint/nursery/use_quoted_strings.rs`

## Implementation Pattern

Each rule follows the established pattern:
1. `declare_lint_rule!` macro with docs, version "next", severity
2. `impl Rule` with Query, State, Signals, Options types
3. `run()` function returning signals
4. `diagnostic()` function returning `RuleDiagnostic`

Rules are auto-discovered by `declare_group_from_fs!` in `nursery.rs` — just creating the `.rs` file is enough for registration.

## Test Plan

Each rule gets inline doc tests (the `expect_diagnostic` and valid examples in the doc comment). These are automatically run by the test infrastructure.

Additionally, run `cargo test -p biome_yaml_analyze` to verify all rules compile and pass.

## Files Changed

| File | Change |
|------|--------|
| `nursery/no_empty_keys.rs` | New rule |
| `nursery/no_empty_sequence_entries.rs` | New rule |
| `nursery/use_consistent_indentation.rs` | New rule |
| `nursery/no_anchor_references.rs` | New rule |
| `nursery/use_quoted_strings.rs` | New rule |
