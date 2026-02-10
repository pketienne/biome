# Plan: Implement `useConsistentSequenceIndentation` lint rule

## Context

yamllint has an `indent-sequences: consistent` option that detects whether block sequences inside mappings are indented or not, then enforces the same style throughout the file. Biome's existing `useConsistentIndentation` rule only checks that indentation width is uniform — it doesn't address sequence-specific indentation. This new rule fills that gap.

The two styles:

```yaml
# Style A — indented (dash deeper than parent key)
parent:
  - item1
  - item2

# Style B — non-indented (dash at same column as parent key)
parent:
- item1
- item2
```

Default behavior: **consistent** — detect the first style used, require all others to match.

## Implementation

### 1. Create the rule file

**File:** `crates/biome_yaml_analyze/src/lint/nursery/use_consistent_sequence_indentation.rs`

**Pattern:** Follow `useBlockStyle` — query `Ast<YamlRoot>`, traverse descendants, collect violations as `Box<[State]>`.

**Rule logic:**
1. Walk all `YamlBlockSequence` nodes in the tree
2. For each sequence, check if its parent chain includes a `YamlBlockMapImplicitEntry` or `YamlBlockMapExplicitEntry` (i.e., the sequence is a value of a mapping entry)
3. If so, compare the column of the first `-` token against the column of the parent mapping key token:
   - `-` column > key column → **indented** (Style A)
   - `-` column == key column → **non-indented** (Style B)
4. Record the first style encountered as the "expected" style
5. Flag all sequences that don't match the expected style

**Column detection:** Use the `-` token's `text_range().start()` offset and walk backwards in the source text to find the preceding newline, then calculate column = offset - newline_offset - 1. Similarly for the parent key token.

Alternatively, since `useConsistentIndentation` already works with raw text and line-based analysis, we can use a simpler approach: get the text offset of each token and compute its column from the source text.

**State struct:**
```rust
pub struct InconsistentSequenceIndent {
    range: TextRange,          // range of the `-` token
    found_style: SeqIndent,    // what was found
    expected_style: SeqIndent, // what was expected (from first occurrence)
}

enum SeqIndent { Indented, NonIndented }
```

**Diagnostic message:**
> "Inconsistent sequence indentation: found non-indented sequence, but indented style was used earlier."
> Note: "Use consistent sequence indentation throughout the file."

### 2. Register the rule

**File:** `crates/biome_yaml_analyze/src/lint/nursery.rs`

This uses `declare_group_from_fs!` which auto-discovers rules from the directory. Just creating the file is sufficient — then run `just gen-analyzer` (or the equivalent codegen) to regenerate.

### 3. Create test files

**File:** `crates/biome_yaml_analyze/tests/specs/nursery/useConsistentSequenceIndentation/valid.yaml`

```yaml
# All indented (consistent)
parent:
  - item1
  - item2
other:
  - a
  - b
```

**File:** `crates/biome_yaml_analyze/tests/specs/nursery/useConsistentSequenceIndentation/valid_non_indented.yaml`

```yaml
# All non-indented (consistent)
parent:
- item1
- item2
other:
- a
- b
```

**File:** `crates/biome_yaml_analyze/tests/specs/nursery/useConsistentSequenceIndentation/invalid.yaml`

```yaml
# Mixed styles — first is indented, second is not
parent:
  - item1
  - item2
other:
- a
- b
```

### 4. Run codegen and tests

```bash
# Regenerate analyzer registration (if needed)
just gen-analyzer

# Run tests to generate snapshots
cargo test -p biome_yaml_analyze

# Accept snapshots
cargo insta accept
```

## Key files to reference

| File | Purpose |
|------|---------|
| `crates/biome_yaml_analyze/src/lint/nursery/use_block_style.rs` | Pattern: root query + descendant traversal + `Box<[State]>` signals |
| `crates/biome_yaml_analyze/src/lint/nursery/use_consistent_indentation.rs` | Pattern: text-based column analysis |
| `crates/biome_yaml_syntax/src/generated/nodes.rs` | `YamlBlockSequence`, `YamlBlockSequenceEntry`, `YamlBlockMapImplicitEntry` |
| `crates/biome_yaml_formatter/src/yaml/auxiliary/block_sequence_entry.rs` | Shows compact vs non-compact formatting logic |
| `crates/biome_yaml_analyze/tests/spec_tests.rs` | Test infrastructure — auto-discovers `tests/specs/**/*.yaml` |

## Verification

1. `cargo test -p biome_yaml_analyze` — all tests pass, new snapshots generated
2. `cargo insta accept` — review and accept new snapshots
3. Manually verify diagnostic output shows correct line/column ranges
4. Verify valid files produce no diagnostics
5. `cargo clippy -p biome_yaml_analyze` — no warnings
