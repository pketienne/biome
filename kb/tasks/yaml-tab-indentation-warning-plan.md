# Tab Indentation Warning Diagnostic

## Date: 2026-02-09

## Problem

YAML 1.2.2 spec section [6.1 Indentation Spaces](https://yaml.org/spec/1.2.2/#61-indentation-spaces)
states: "Note: To maintain portability, tab characters must not be used in indentation." Currently
the YAML lexer treats tabs as valid whitespace (via `is_space()`) and silently accepts them in
indentation context. There is no diagnostic warning when tabs appear at the start of a line
where they contribute to indentation level.

## Spec References

- [Section 5.5 White Space](https://yaml.org/spec/1.2.2/#55-white-space-characters): Tabs and
  spaces are both valid whitespace (`s-white ::= s-space | s-tab`).
- [Section 6.1 Indentation Spaces](https://yaml.org/spec/1.2.2/#61-indentation-spaces):
  `s-indent(n) ::= s-space × n` — indentation uses **only spaces**, not tabs.
- Tabs ARE allowed **inside** values, after the indentation prefix, and in flow contexts.

## Current Code

### `is_space()` (line 1227)
```rust
fn is_space(c: u8) -> bool {
    c == b' ' || c == b'\t'
}
```
Used throughout the lexer for both indentation-context whitespace and general whitespace.
Changing this function would break tab handling everywhere.

### `consume_whitespaces()` (line 992)
Consumes all space/tab characters. Already has a diagnostic for non-standard whitespace
(characters that aren't tab or space but match `WHS` dispatch). Does NOT distinguish
between indentation tabs and inline tabs.

### `consume_trivia(trailing: bool)` (line 755)
Consumes whitespace, newlines, and comments. Called from `evaluate_block_scope()` with
`trailing: false` (start of line) and from various places with `trailing: true`.

### `evaluate_block_scope()` (line 343)
Called when the lexer encounters a newline character. Consumes trivia (whitespace + newlines)
and closes any block scopes that are breached by the new indentation level.

### `TextCoordinate` (line 1156)
Tracks `offset` (byte position) and `column` (bytes since last newline). The `column`
field determines indentation level. When `column == 0` and whitespace is consumed, that
whitespace IS the indentation.

## Solution

Add a diagnostic in `consume_whitespaces()` when tabs are encountered in indentation
context (i.e., at the start of a line, `column == 0` or in leading whitespace before
any non-whitespace on the line). The key insight: **before** `consume_whitespaces()` runs,
if the current `column` position minus the column at line start is 0 (or more precisely,
only spaces and tabs have been seen since the last newline), any tabs in that span are
indentation tabs.

### Approach: Track "in indentation" state

The simplest approach is to add a parameter or check the column/context when entering
`consume_whitespaces()`:

1. Add an `in_indentation: bool` parameter to `consume_whitespaces()` (or create a
   separate `consume_indentation_whitespace()` method).
2. When `in_indentation` is true and a tab is encountered, emit a diagnostic.
3. The tab is still consumed (lexer doesn't reject it — it's a warning, not an error).

## Implementation

### File: `crates/biome_yaml_parser/src/lexer/mod.rs`

#### Step 1: Add `consume_indentation_whitespace()` method

This avoids changing the signature of `consume_whitespaces()` which is called from many
places where tabs are perfectly valid.

```rust
/// Consumes whitespace at the start of a line, emitting diagnostics for tabs.
/// YAML 1.2.2 §6.1: "tab characters must not be used in indentation"
fn consume_indentation_whitespace(&mut self) {
    self.assert_current_char_boundary();

    while let Some(c) = self.current_byte() {
        let dispatch = lookup_byte(c);
        if !matches!(dispatch, WHS) {
            break;
        }

        if c == b'\t' {
            let start = self.text_position();
            self.advance(1);
            // Continue consuming any adjacent tabs as one diagnostic
            while self.current_byte() == Some(b'\t') {
                self.advance(1);
            }
            self.push_diagnostic(
                ParseDiagnostic::new(
                    "Tabs are not allowed in YAML indentation",
                    start..self.text_position(),
                )
                .with_hint(
                    "YAML 1.2.2 forbids tab characters in indentation. \
                     Use spaces instead. See https://yaml.org/spec/1.2.2/#61-indentation-spaces"
                ),
            );
        } else if is_space(c) {
            self.advance(1);
        } else if is_break(c) {
            break;
        } else {
            // Non-standard whitespace — use existing diagnostic from consume_whitespaces
            let start = self.text_position();
            self.advance(1);
            self.push_diagnostic(
                ParseDiagnostic::new(
                    "The YAML standard allows only two types of whitespace characters: tabs and spaces",
                    start..self.text_position(),
                )
                .with_hint("Use a regular whitespace character instead. For more detail, please check https://yaml.org/spec/1.2.2/#55-white-space-characters"),
            );
        }
    }
}
```

#### Step 2: Use `consume_indentation_whitespace()` in `consume_trivia()`

In `consume_trivia()` (line 755), when `trailing: false` and the lexer just consumed a
newline (next whitespace is indentation), call `consume_indentation_whitespace()` instead
of `consume_whitespace_token()`:

```rust
fn consume_trivia(&mut self, trailing: bool) -> Vec<LexToken> {
    let mut trivia = Vec::new();
    let mut at_line_start = !trailing; // If not trailing, we're at start of line
    while let Some(current) = self.current_byte() {
        if is_space(current) {
            if at_line_start {
                let start = self.current_coordinate;
                self.consume_indentation_whitespace();
                trivia.push(LexToken::new(WHITESPACE, start, self.current_coordinate));
            } else {
                trivia.push(self.consume_whitespace_token());
            }
        } else if is_break(current) {
            if trailing {
                break;
            }
            trivia.push(self.consume_newline_token());
            at_line_start = true; // After newline, next whitespace is indentation
        } else if current == b'#' {
            trivia.push(self.consume_comment());
            at_line_start = false;
        } else {
            break;
        }
    }
    trivia
}
```

#### Step 3: Also check in `evaluate_block_scope()`

`evaluate_block_scope()` (line 343) calls `consume_trivia(false)` which with the above
change will already use `consume_indentation_whitespace()`. No additional changes needed
here.

### File: `crates/biome_yaml_parser/src/lexer/tests.rs`

Add tests:

```rust
#[test]
fn tab_in_indentation_diagnostic() {
    let src = "key:\n\tvalue";
    let lexer = YamlLexer::from_str(src);
    let diagnostics = lex_all(lexer);
    assert!(diagnostics.iter().any(|d|
        d.message().to_string().contains("Tabs are not allowed")
    ));
}

#[test]
fn tab_after_content_no_diagnostic() {
    // Tabs after content (inline) are allowed
    let src = "key:\tvalue";
    let lexer = YamlLexer::from_str(src);
    let diagnostics = lex_all(lexer);
    assert!(!diagnostics.iter().any(|d|
        d.message().to_string().contains("Tabs are not allowed")
    ));
}

#[test]
fn mixed_space_tab_indentation_diagnostic() {
    // Spaces followed by tab in indentation
    let src = "key:\n  \tvalue";
    let lexer = YamlLexer::from_str(src);
    let diagnostics = lex_all(lexer);
    assert!(diagnostics.iter().any(|d|
        d.message().to_string().contains("Tabs are not allowed")
    ));
}
```

### File: `tests/yaml_test_suite/err/lexer/tab_indentation.yaml` (new)

```yaml
key:
	value
nested:
	inner:
		deep: true
```

## Edge Cases

### 1. Tabs in flow context
Flow collections (`[a, b]`, `{a: 1}`) don't use indentation for structure. Tabs between
flow elements are fine. The `consume_trivia(true)` path (trailing trivia in flow context)
won't trigger the indentation check because `at_line_start` defaults to `false` when
`trailing: true`.

### 2. Tabs after indentation spaces
`"  \tvalue"` — spaces then tab. The tab is still in the indentation prefix (before any
non-whitespace). `consume_indentation_whitespace()` will catch this since it runs until
non-whitespace is found.

### 3. Tab-only lines (blank lines with tabs)
A line with only tabs followed by a newline. The tabs are technically indentation
whitespace. The diagnostic fires. This matches the spec intent — tabs should not
determine indentation level.

### 4. Document with `%YAML 1.1` directive
YAML 1.1 was more permissive about tabs. Our parser targets 1.2.2. If future work
adds version-aware parsing, this diagnostic could be conditional.

### 5. Performance
The diagnostic adds a string allocation per tab occurrence. In practice, well-formed YAML
has no tabs in indentation, so the hot path (spaces only) is unchanged. The `at_line_start`
boolean in `consume_trivia()` adds negligible overhead.

## Verification

1. `cargo build -p biome_yaml_parser` — compiles
2. `cargo test -p biome_yaml_parser` — all existing tests pass
3. New tests verify:
   - Tab at start of indentation → diagnostic emitted
   - Tab after content (inline) → no diagnostic
   - Mixed space+tab in indentation → diagnostic
   - Flow context tabs → no diagnostic
4. Update any existing snapshots that contain tabs in indentation (if any)
5. Verify `cargo insta review` for snapshot changes

## Files Summary

| File | Action |
|------|--------|
| `crates/biome_yaml_parser/src/lexer/mod.rs` | Add `consume_indentation_whitespace()`, modify `consume_trivia()` |
| `crates/biome_yaml_parser/src/lexer/tests.rs` | Add tab indentation diagnostic tests |
| `tests/yaml_test_suite/err/lexer/tab_indentation.yaml` | New test fixture |
