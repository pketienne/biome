# Plan 17: Multi-Document Support Hardening

## Status: COMPLETE

## Goal

Harden multi-document YAML support by:
1. Implementing YAML directive lexing (`%YAML`, `%TAG`)
2. Adding edge case parser tests
3. Adding formatter tests for multi-document edge cases

## Current State

- `---` (document start) and `...` (document end) are fully supported in lexer and parser
- `DIRECTIVE_LITERAL` token kind exists in grammar but lexer never emits it
- `%` at column 0 falls through to `consume_unexpected_token()`, creating ERROR_TOKEN
- Parser has `DirectiveList` and `parse_directive()` that check for `DIRECTIVE_LITERAL` but never receive it
- Formatter handles `YamlDirective` node (just outputs the token verbatim)
- Multi-document parsing and formatting works for basic `---`/`...` separated docs

## Changes

### 17A: Add directive lexing

**File**: `crates/biome_yaml_parser/src/lexer/mod.rs`

Add `consume_directive()` function and dispatch `b'%'` at column 0:

```rust
fn is_at_directive(&self) -> bool {
    self.current_coordinate.column == 0
        && self.current_byte() == Some(b'%')
}

fn consume_directive(&mut self) -> LinkedList<LexToken> {
    self.assert_byte(b'%');
    let start = self.current_coordinate;
    let mut tokens = self.close_all_scopes();

    // Consume the entire directive line as DIRECTIVE_LITERAL
    while let Some(c) = self.current_byte() {
        if is_break(c) { break; }
        if c == b'#' { break; } // comment starts
        self.advance(1);
    }

    tokens.push_back(LexToken::new(DIRECTIVE_LITERAL, start, self.current_coordinate));
    let mut trivia = self.consume_trailing_trivia();
    tokens.append(&mut trivia);
    tokens
}
```

In `consume_tokens()` dispatch, add before `_ => self.consume_unexpected_token()`:

```rust
b'%' if self.is_at_directive() => self.consume_directive(),
```

### 17B: Add parser edge case tests

**File**: `crates/biome_yaml_parser/tests/yaml_test_suite/ok/document/` (new test files)

Add test cases:
- `directive_yaml.yaml`: `%YAML 1.2\n---\nkey: value`
- `directive_tag.yaml`: `%TAG ! tag:example.com,2014:\n---\n!app data`
- `consecutive_doc_start.yaml`: `---\nfoo: 1\n---\nbar: 2`
- `doc_end_only.yaml`: `key: value\n...`
- `empty_between_markers.yaml`: `---\n---\n...`

### 17C: Add formatter snapshot tests

**File**: `crates/biome_yaml_formatter/tests/specs/yaml/document/directives.yaml`

```yaml
%YAML 1.2
---
key: value
```

**File**: `crates/biome_yaml_formatter/tests/specs/yaml/document/separated_by_doc_end.yaml`

```yaml
first: 1
...
---
second: 2
...
---
third: 3
```

### 17D: Add stress test for directive-heavy multi-doc

**File**: `crates/biome_yaml_formatter/tests/stress/multi_document.yaml`

Multi-document file with directives, `---`, `...`, empty docs, various content types.

## Test Plan

1. Parser tests: new snapshot tests for directive parsing
2. Formatter tests: new snapshot tests for directive formatting
3. Stress test: idempotency check for multi-document with directives
4. Existing tests: all must continue passing

## Files Changed

| File | Change |
|------|--------|
| `lexer/mod.rs` | Add `consume_directive()`, `is_at_directive()`, dispatch `b'%'` |
| Parser test files | New directive and multi-doc edge case tests |
| Formatter test files | New directive and multi-doc formatter tests |
| `stress/multi_document.yaml` | New stress test |
| `stress_test.rs` | Add `stress_multi_document` test |
| Snapshot files | Accept new snapshots |
