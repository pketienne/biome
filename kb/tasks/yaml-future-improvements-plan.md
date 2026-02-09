# YAML Future Improvements Implementation Plan

## Date: 2026-02-09

Four features to bring the YAML implementation to production-grade quality: smart flow-to-block conversion, merge key validation, richer parser diagnostics, and performance optimization.

---

## Feature A: Flow-to-Block Style Conversion

### Context

Currently `Expand::Always` forces all flow collections to multi-line form, but keeps them as flow syntax (`{`/`}`). There is no way to automatically convert inline flow collections to block style based on line width. This is the most common formatting preference for YAML files — short collections stay inline, long ones become block mappings/sequences.

### Approach: Enhance `Expand::Auto` Semantics

Rather than adding a new `Expand` variant, make `Expand::Auto` smarter by using the existing `group()`/`should_expand()` infrastructure with `best_fitting!` for flow-to-block conversion.

### Implementation

#### Step 1: Add `best_fitting!` to flow collection formatters

**Files:**
- `crates/biome_yaml_formatter/src/yaml/auxiliary/flow_sequence.rs`
- `crates/biome_yaml_formatter/src/yaml/auxiliary/flow_mapping.rs`

Current code uses `group().should_expand(should_expand)` which only toggles between flat flow and expanded flow. Change to `best_fitting!` with three variants:

```rust
// flow_sequence.rs
fn fmt_fields(&self, node: &YamlFlowSequence, f: &mut YamlFormatter) -> FormatResult<()> {
    let entries = node.entries();
    let expand = f.context().options().expand();

    if entries.is_empty() {
        return write!(f, [node.l_brack_token()?.format(), node.r_brack_token()?.format()]);
    }

    match expand {
        Expand::Never => {
            // Keep flow, single line
            write!(f, [
                node.l_brack_token()?.format(),
                space(), entries.format(), space(),
                node.r_brack_token()?.format(),
            ])
        }
        Expand::Always => {
            // Convert to block sequence
            format_as_block_sequence(entries, f)
        }
        Expand::Auto => {
            // Use best_fitting! — try flow first, fall back to block
            write!(f, [best_fitting!(
                // Variant 1: Compact flow style [a, b, c]
                format_args![
                    node.l_brack_token()?.format(),
                    space(), entries.format(), space(),
                    node.r_brack_token()?.format(),
                ],
                // Variant 2: Expanded flow style
                format_args![
                    node.l_brack_token()?.format(),
                    group(&format_args![
                        indent(&format_args![
                            soft_line_break_or_space(),
                            entries.format(),
                        ]),
                        soft_line_break_or_space(),
                    ]).should_expand(true),
                    node.r_brack_token()?.format(),
                ],
                // Variant 3: Block style (most expanded)
                format_as_block_sequence_args(entries),
            )])
        }
    }
}
```

#### Step 2: Add block-style formatting helpers

**File:** `crates/biome_yaml_formatter/src/yaml/auxiliary/flow_sequence.rs`

Add a helper that formats flow sequence entries as block sequence entries:

```rust
fn format_as_block_sequence(entries: &YamlFlowSequenceEntryList, f: &mut YamlFormatter) -> FormatResult<()> {
    // Suppress the flow brackets (they become synthetic tokens in block form)
    for entry in entries {
        write!(f, [hard_line_break(), text("- "), entry.format()])?;
    }
    Ok(())
}
```

Similar helper for flow mappings converting to block form.

#### Step 3: Handle nested conversions

When a flow mapping inside a block mapping converts to block, the parent must adjust indentation. This is handled naturally by the `indent()` wrapper already present in block node formatters.

### Complexity: Medium-High

The `best_fitting!` macro requires all variants to be pre-computed, and the block variant needs to suppress flow brackets while emitting dash/key-colon syntax. Edge cases:
- Nested flow collections (inner stays flow even if outer converts to block)
- Flow collections with comments (comments force expansion)
- Flow collections used as mapping keys (cannot convert to block)

### Testing

- Add formatter tests with varying `line_width` values (30, 50, 80, 120)
- Test nested flow collections
- Test flow collections as mapping values vs keys
- Verify idempotency (formatted output re-formats identically)

---

## Feature B: Semantic Merge Key (`<<: *defaults`) Validation

### Context

The YAML merge key (`<<`) is a widely-used pattern for DRY configuration (anchors/aliases). Currently the parser treats `<<` as a regular plain scalar key with no semantic validation. This means:
- `<<: not_an_alias` is silently accepted
- `<<: *alias_to_scalar` is silently accepted (should be mapping)
- `noDuplicateKeys` may false-positive on multiple `<<` keys (which are valid)

### Implementation

#### Step 1: New lint rule `useValidMergeKeys`

**File:** `crates/biome_yaml_analyze/src/lint/nursery/use_valid_merge_keys.rs`

```rust
declare_lint_rule! {
    /// Validates that merge keys (`<<`) have alias values pointing to mappings.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    /// ```yaml,expect_diagnostic
    /// person:
    ///   <<: not_an_alias
    ///   name: John
    /// ```
    ///
    /// ### Valid
    /// ```yaml
    /// defaults: &defaults
    ///   timeout: 30
    /// server:
    ///   <<: *defaults
    ///   port: 8080
    /// ```
    pub UseValidMergeKeys {
        version: "next",
        name: "useValidMergeKeys",
        language: "yaml",
        recommended: true,
    }
}
```

Rule logic:
1. Query: `Ast<YamlBlockMapping>` and `Ast<YamlFlowMapping>`
2. For each entry, check if key text is `<<`
3. If key is `<<`, validate that:
   - Value is an alias node (`YamlAliasNode`) or a sequence of alias nodes
   - (Optional advanced check) The referenced anchor resolves to a mapping, not a scalar/sequence

#### Step 2: Exempt `<<` from `noDuplicateKeys`

**File:** `crates/biome_yaml_analyze/src/lint/nursery/no_duplicate_keys.rs`

In the `run()` method, skip entries where `get_key_text()` returns `"<<"`:

```rust
if key_text == "<<" {
    continue; // Merge keys can appear multiple times
}
```

Same change in `no_duplicate_flow_keys.rs`.

#### Step 3: Tests

**Files:**
- `crates/biome_yaml_analyze/tests/specs/nursery/useValidMergeKeys/valid.yaml`
- `crates/biome_yaml_analyze/tests/specs/nursery/useValidMergeKeys/invalid.yaml`

Valid cases:
```yaml
defaults: &defaults
  timeout: 30
server:
  <<: *defaults
  port: 8080

# Multiple merge keys (valid per spec)
combined:
  <<: *defaults
  <<: *overrides
  name: final

# Sequence of aliases
merged:
  <<: [*defaults, *overrides]
```

Invalid cases:
```yaml
# Merge key with plain scalar value
bad:
  <<: not_an_alias

# Merge key with inline mapping (not an alias)
also_bad:
  <<: {a: 1}
```

### Complexity: Medium

Existing anchor/alias infrastructure provides the foundation. The main challenge is cross-referencing alias targets to validate they resolve to mappings (requires walking the AST to find matching anchors).

---

## Feature C: Additional Parser Diagnostics

### Context

The lexer silently accepts several invalid constructs that should produce warnings or errors:
1. Invalid escape sequences in double-quoted strings (e.g., `\q`)
2. Incomplete hex escapes (e.g., `\x4` with only 1 of 2 required digits)
3. Tab characters in indentation (YAML 1.2.2 forbids tabs for indentation)

### Implementation

#### Step 1: Invalid escape sequence diagnostics

**File:** `crates/biome_yaml_parser/src/lexer/mod.rs`

In `consume_double_quoted_literal()`, the `Some(_)` arm (line ~591) currently just advances. Add a diagnostic:

```rust
// Invalid escape — advance past the character with diagnostic
Some(_) => {
    let escape_start = self.text_position() - TextSize::from(1); // backslash position
    let char = self.current_char_unchecked();
    self.advance(char.len_utf8());
    let err = ParseDiagnostic::new(
        format!("Invalid escape sequence `\\{char}` in double-quoted string"),
        escape_start..self.text_position(),
    ).with_hint(
        "Valid escapes: \\0 \\a \\b \\t \\n \\v \\f \\r \\e \\\" \\\\ \\/ \\\\ \\_ \\N \\L \\P \\xNN \\uNNNN \\UNNNNNNNN"
    );
    self.diagnostics.push(err);
}
```

#### Step 2: Incomplete hex escape diagnostics

**File:** `crates/biome_yaml_parser/src/lexer/mod.rs`

Change `consume_hex_digits()` to return the count of consumed digits and emit a diagnostic when fewer than expected:

```rust
fn consume_hex_digits(&mut self, expected: usize) {
    let start = self.text_position();
    let mut consumed = 0;
    for _ in 0..expected {
        match self.current_byte() {
            Some(c) if c.is_ascii_hexdigit() => {
                self.advance(1);
                consumed += 1;
            }
            _ => break,
        }
    }
    if consumed < expected {
        let err = ParseDiagnostic::new(
            format!(
                "Expected {expected} hex digits in escape sequence, found {consumed}"
            ),
            start..self.text_position(),
        );
        self.diagnostics.push(err);
    }
}
```

#### Step 3: Tab indentation warning

**File:** `crates/biome_yaml_parser/src/lexer/mod.rs`

In `evaluate_block_scope()` or at the start of `consume_tokens()`, after consuming leading whitespace on a new line, check if any tabs were used before non-whitespace content:

This is lower priority since tabs in indentation are already handled by the existing whitespace validation. Only add if the current validation doesn't cover this case.

#### Step 4: Tests

**File:** `crates/biome_yaml_parser/src/lexer/tests/flow.rs`

Add tests that verify diagnostics are emitted:
```rust
#[test]
fn lex_invalid_escape_produces_diagnostic() {
    // Test that \q produces a diagnostic but still lexes the full string
    assert_lex!(
        r#""\q""#,
        FLOW_START:0,
        DOUBLE_QUOTED_LITERAL:4,
        FLOW_END:0,
    );
    // TODO: Also verify diagnostic count/content
}

#[test]
fn lex_incomplete_hex_escape() {
    // \x4 is missing second hex digit
    assert_lex!(
        r#""\x4""#,
        FLOW_START:0,
        DOUBLE_QUOTED_LITERAL:5,
        FLOW_END:0,
    );
}
```

Add spec test YAML files:
- `tests/yaml_test_suite/err/flow/invalid_escape.yaml`: `key: "hello\qworld"`
- `tests/yaml_test_suite/err/flow/incomplete_hex_escape.yaml`: `key: "\x4"`

### Complexity: Low

Straightforward additions to existing diagnostic infrastructure. All patterns already exist in the codebase.

---

## Feature D: Performance Optimization for Large YAML Files

### Context

The lexer uses `LinkedList<LexToken>` extensively (28 occurrences) for token accumulation. LinkedList has poor cache locality and per-node heap allocation. For large YAML files (10K+ lines), this creates measurable overhead.

### Implementation

#### Phase 1: Replace `LinkedList<LexToken>` with `Vec<LexToken>` (High Impact)

**File:** `crates/biome_yaml_parser/src/lexer/mod.rs`

**Step 1a:** Change main token store

```rust
pub(crate) struct YamlLexer<'src> {
    // ...
    tokens: VecDeque<LexToken>,  // Was LinkedList<LexToken>
}
```

Use `VecDeque` for the main token queue since it needs both `push_back` and `pop_front`. This provides O(1) amortized for both operations with contiguous memory.

**Step 1b:** Change local token accumulation to `Vec`

All helper functions that return `LinkedList<LexToken>` should return `Vec<LexToken>` instead:

```rust
// Before:
fn consume_sequence_entry(&mut self) -> LinkedList<LexToken> {
    let mut tokens = LinkedList::new();
    tokens.push_back(indicator);
    tokens.push_front(LexToken::pseudo(SEQUENCE_START, indicator.start));
    // ...
    tokens
}

// After:
fn consume_sequence_entry(&mut self) -> Vec<LexToken> {
    let mut tokens = Vec::with_capacity(3);  // Pre-sized for typical case
    tokens.push(LexToken::pseudo(SEQUENCE_START, indicator.start));
    tokens.push(indicator);
    // ...
    tokens
}
```

**Key change:** `push_front` becomes insert-at-0 or pre-allocate in reverse order. Since most `push_front` calls add exactly 1 element before push_back calls, we can restructure to avoid front-insertion entirely:

```rust
// Pattern: push_front(A), push_back(B), push_back(C)
// Becomes: push(A), push(B), push(C) — same order, no front-insert needed
```

**Step 1c:** Change `append` to `extend`

```rust
// Before:
self.tokens.append(&mut tokens);

// After:
self.tokens.extend(tokens.drain(..));
// Or simply:
self.tokens.extend(tokens);
```

**Step 1d:** Remove `From<LexToken> for LinkedList<LexToken>`

Replace the `impl From<LexToken> for LinkedList<LexToken>` with returning `vec![token]` or pushing directly to the main queue.

#### Phase 2: Reduce String Allocations in Analyzer (Medium Impact)

**Files:**
- `crates/biome_yaml_analyze/src/lint/nursery/no_duplicate_keys.rs`
- `crates/biome_yaml_analyze/src/lint/nursery/no_duplicate_flow_keys.rs`
- `crates/biome_yaml_analyze/src/lint/nursery/use_consistent_key_ordering.rs`

**Step 2a:** Replace `get_key_text()` double allocation:

```rust
// Before (2 String allocations per call):
pub(crate) fn get_key_text(key: &AnyYamlMappingImplicitKey) -> Option<String> {
    match key {
        AnyYamlMappingImplicitKey::YamlFlowJsonNode(node) => {
            node.content().map(|content| content.to_string().trim().to_string())
        }
        // ...
    }
}

// After (1 allocation, no intermediate):
pub(crate) fn get_key_text(key: &AnyYamlMappingImplicitKey) -> Option<String> {
    match key {
        AnyYamlMappingImplicitKey::YamlFlowJsonNode(node) => {
            node.content().map(|content| {
                let s = content.to_string();
                if s.starts_with(char::is_whitespace) || s.ends_with(char::is_whitespace) {
                    s.trim().to_string()
                } else {
                    s // No trim needed, avoid second allocation
                }
            })
        }
        // ...
    }
}
```

**Step 2b:** Use `TokenText` or `SyntaxNodeText` instead of `String` where possible. These are borrowed references into the source text and avoid allocation entirely:

```rust
// If the syntax API supports it:
let key_text = key.syntax().text_trimmed();  // Returns SyntaxNodeText (no allocation)
```

#### Phase 3: Pre-size Vectors (Low Impact, Easy Win)

**File:** `crates/biome_yaml_parser/src/lexer/mod.rs`

Add capacity hints to frequently-created vectors:

```rust
// consume_flow_collection — typical flow has 5-20 tokens
let mut collection_tokens = Vec::with_capacity(16);

// consume_block_header_tokens — typically 1-3 tokens
let mut tokens = Vec::with_capacity(4);

// consume_trivia — typically 1-3 tokens
let mut trivia = Vec::with_capacity(4);
```

#### Phase 4: Benchmark Suite

**File:** `crates/biome_yaml_parser/benches/yaml_bench.rs` (new)

Create benchmarks using `criterion`:

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use biome_yaml_parser::parse_yaml;

fn bench_parse_large_file(c: &mut Criterion) {
    let sizes = [100, 1000, 10000];
    let mut group = c.benchmark_group("parse_yaml");

    for size in sizes {
        let yaml = generate_yaml(size); // Generate N-entry mapping
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &yaml,
            |b, yaml| b.iter(|| parse_yaml(yaml)),
        );
    }
    group.finish();
}

fn generate_yaml(entries: usize) -> String {
    let mut s = String::new();
    for i in 0..entries {
        s.push_str(&format!("key_{i}: value_{i}\n"));
    }
    s
}

criterion_group!(benches, bench_parse_large_file);
criterion_main!(benches);
```

Add to `Cargo.toml`:
```toml
[[bench]]
name = "yaml_bench"
harness = false

[dev-dependencies]
criterion = "0.5"
```

### Complexity: Medium (Phase 1), Low (Phases 2-4)

Phase 1 is the most impactful but requires careful refactoring of 28 LinkedList usages. The key risk is ensuring `push_front` semantics are preserved (pseudo tokens like `MAPPING_START` must come before the indicator token).

### Expected Impact

| Optimization | Impact | Files Changed |
|-------------|--------|---------------|
| LinkedList → VecDeque/Vec | ~20-30% parser speedup (cache locality) | 1 file, 28 sites |
| String allocation reduction | ~5-10% analyzer speedup per mapping | 3 files |
| Pre-sized vectors | ~2-5% parser speedup | 1 file |
| Benchmark suite | Measurement infrastructure | 1 new file + Cargo.toml |

---

## Implementation Order

| Priority | Feature | Effort | Impact |
|----------|---------|--------|--------|
| 1 | C: Parser diagnostics | Low (1-2 hours) | Immediate user-facing error quality |
| 2 | B: Merge key validation | Medium (3-4 hours) | Common YAML pattern support |
| 3 | D: Performance (Phase 1) | Medium (4-6 hours) | Foundation for large file handling |
| 4 | A: Flow-to-block conversion | High (6-10 hours) | Advanced formatting capability |
| 5 | D: Performance (Phases 2-4) | Low (2-3 hours) | Polish and measurement |

## Files Summary

| Feature | Files |
|---------|-------|
| A: Flow-to-block | `flow_sequence.rs`, `flow_mapping.rs`, `flow_sequence_entry_list.rs`, `flow_map_entry_list.rs` |
| B: Merge keys | New `use_valid_merge_keys.rs`, `no_duplicate_keys.rs`, `no_duplicate_flow_keys.rs`, test files |
| C: Diagnostics | `lexer/mod.rs`, `lexer/tests/flow.rs`, new test YAML files |
| D: Performance | `lexer/mod.rs` (28 sites), 3 analyzer files, new benchmark file |
