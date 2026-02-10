# Plan 33: Bidirectional Quote Style Conversion

## Status: PENDING

## Context

`useConsistentQuoteStyle` currently only converts single→double quotes. This plan adds double→single conversion and makes the direction configurable via rule options.

---

## Changes

### Rule Options
**`crates/biome_rule_options/src/use_consistent_quote_style.rs`** (NEW or modify existing):
```rust
pub struct UseConsistentQuoteStyleOptions {
    pub preferred_quote: Option<PreferredQuote>,
}
pub enum PreferredQuote { Double, Single }
```
Default: `Double` (preserves current behavior)

### Rule Logic Update
**`crates/biome_yaml_analyze/src/lint/nursery/use_consistent_quote_style.rs`**:
- `run()`: Check `ctx.options().preferred_quote()`. If `Double`, flag single-quoted scalars (current behavior). If `Single`, flag double-quoted scalars.
- `action()`: For `Single` preference, convert `"text"` → `'text'`:
  - Unescape: `\"` → `"`
  - Escape for single quotes: `'` → `''`
  - Skip if content contains backslash escapes that can't be represented in single quotes (`\n`, `\t`, `\\`, etc.)

### Registration
- Add options struct to `crates/biome_rule_options/src/lib.rs`
- Wire `type Options = UseConsistentQuoteStyleOptions` in the rule

## Verification
- `cargo build -p biome_yaml_analyze`
- `cargo test -p biome_yaml_analyze`
