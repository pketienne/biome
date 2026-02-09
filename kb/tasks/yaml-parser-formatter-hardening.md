# Implement 4 YAML Parser/Formatter Features

## Context

The YAML parser has edge cases with mismatched brackets causing panics, doesn't validate escape sequences in double-quoted strings, lacks format_range wiring in the service layer, and always uses spaces for sequence indentation even with tab indent style. These 4 features round out the parser/formatter robustness.

## Feature 1: Format Range Wiring (simplest — service layer only)

**File**: `crates/biome_service/src/file_handlers/yaml.rs`

`biome_yaml_formatter::format_range()` and `is_range_formatting_node()` already exist in `crates/biome_yaml_formatter/src/lib.rs` (lines 317-323, 233-244). Only the service wiring is missing.

1. Add `format_range` handler function (follow JSON pattern from `json.rs:441-453`):
```rust
fn format_range(
    path: &BiomePath,
    document_file_source: &DocumentFileSource,
    parse: AnyParse,
    settings: &Settings,
    range: TextRange,
) -> Result<Printed, WorkspaceError> {
    let options = settings.format_options::<YamlLanguage>(path, document_file_source);
    let tree = parse.syntax();
    let printed = biome_yaml_formatter::format_range(options, &tree, range)?;
    Ok(printed)
}
```
2. Change `format_range: None` to `format_range: Some(format_range)` in `FormatterCapabilities`
3. Add `use biome_rowan::TextRange;` if not already imported

## Feature 2: Mismatched Bracket Recovery (parser fix)

**File**: `crates/biome_yaml_parser/src/parser/flow.rs`

Root cause: when `[1, 2}` is parsed, `FlowSequenceEntryList::is_at_list_end()` never returns true because it only checks `T![']']` and `FLOW_END`, not `T!['}']`. This means `parse_list()` loops forever since `parse_element()` returns `Absent` on `}` and recovery also fails.

Changes:
1. `FlowSequenceEntryRecovery::is_at_recovered()` (line ~136): add `|| p.at(T!['}'])`
2. `FlowSequenceEntryList::is_at_list_end()` (line ~192): add `|| p.at(T!['}'])`
3. `FlowMapEntryRecovery::is_at_recovered()` (line ~220): add `|| p.at(T![']'])`
4. `FlowMapEntryList::is_at_list_end()` (line ~251): add `|| p.at(T![']'])`
5. In `parse_flow_sequence()` (line ~105): after list parsing, if `!p.eat(T![']'])`, check if at `}` and bump it with a diagnostic
6. In `parse_flow_mapping()` (line ~117): after list parsing, if `!p.eat(T!['}'])`, check if at `]` and bump it with a diagnostic
7. Re-add test file `tests/yaml_test_suite/err/flow/mismatched_brackets.yaml` with content like `[1, 2}` and `{a: 1]`

## Feature 3: Double-Quoted Escape Sequence Validation (lexer)

**File**: `crates/biome_yaml_parser/src/lexer/mod.rs`

Currently `consume_double_quoted_literal()` only handles `\"` escape. Need full YAML 1.2.2 validation.

1. Replace the `Some(b'\\')` arm in `consume_double_quoted_literal()` (line ~577):
```rust
Some(b'\\') => {
    self.advance(1); // skip backslash
    match self.current_byte() {
        // Single-char escapes: \0 \a \b \t \n \v \f \r \e \" \\ \/ \  \_ \N \L \P
        Some(b'0' | b'a' | b'b' | b't' | b'n' | b'v' | b'f' | b'r'
             | b'e' | b'"' | b'\\' | b'/' | b' ' | b'_' | b'N' | b'L' | b'P') => {
            self.advance(1);
        }
        // \xNN — 2 hex digits
        Some(b'x') => {
            self.advance(1);
            self.expect_hex_digits(2);
        }
        // \uNNNN — 4 hex digits
        Some(b'u') => {
            self.advance(1);
            self.expect_hex_digits(4);
        }
        // \UNNNNNNNN — 8 hex digits
        Some(b'U') => {
            self.advance(1);
            self.expect_hex_digits(8);
        }
        // \<newline> — line continuation
        Some(b'\n' | b'\r') => {
            self.advance(1);
        }
        // Invalid escape — emit diagnostic, advance past the char
        Some(_) => {
            // TODO: emit diagnostic for invalid escape sequence
            self.advance(1);
        }
        None => {}
    }
}
```
2. Add helper `expect_hex_digits(count: usize)`:
```rust
fn expect_hex_digits(&mut self, count: usize) {
    for _ in 0..count {
        match self.current_byte() {
            Some(c) if c.is_ascii_hexdigit() => self.advance(1),
            _ => break, // TODO: emit diagnostic for incomplete hex escape
        }
    }
}
```
3. Add lexer tests for valid and invalid escape sequences
4. Add test YAML file with escape sequences

## Feature 4: Tab Indentation for Compact Sequences (formatter)

**File**: `crates/biome_yaml_formatter/src/yaml/auxiliary/block_sequence_entry.rs`

Currently `align(2, ...)` at line ~34 always renders as spaces (printer hardcoded behavior). With tab indent style, should use `indent()` instead.

Change the formatting logic:
```rust
let indent_style = f.context().options().indent_style();
if indent_style.is_tab() {
    write!(f, [indent(&format_with(|f| { /* existing body */ }))])?;
} else {
    write!(f, [align(2, &format_with(|f| { /* existing body */ }))])?;
}
```

This ensures tabs are used when `indent_style: tab`, and spaces (align(2)) when `indent_style: space`.

## Verification

1. `cargo build -p biome_yaml_parser` — compiles
2. `cargo test -p biome_yaml_parser` — all tests pass (accept snapshots for new test files)
3. `cargo build -p biome_yaml_formatter` — compiles
4. `cargo test -p biome_yaml_formatter` — all tests pass (accept snapshots)
5. `cargo build -p biome_service` — compiles (format_range wiring)
6. Test mismatched brackets: `[1, 2}` and `{a: 1]` parse without panic
7. Test escape sequences: `"hello\nworld"`, `"bad\q"` lex correctly
8. Test tab indentation with quick_test if applicable

## Files Summary

| File | Feature | Action |
|------|---------|--------|
| `crates/biome_service/src/file_handlers/yaml.rs` | F1 | Add `format_range` handler + wire capability |
| `crates/biome_yaml_parser/src/parser/flow.rs` | F2 | Add cross-bracket recovery to list end/recovery checks |
| `crates/biome_yaml_parser/src/lexer/mod.rs` | F3 | Full YAML 1.2.2 escape validation in double-quoted strings |
| `crates/biome_yaml_formatter/src/yaml/auxiliary/block_sequence_entry.rs` | F4 | Conditional indent() vs align(2) based on indent style |
| Test files (parser + formatter) | All | New test files + snapshot updates |
