# Plan: Clean Up Formatter Warnings

**Created:** 2026-02-10

---

## Problem

Two unused functions in `crates/biome_markdown_formatter/src/verbatim.rs` produce compiler warnings on every build:

```
warning: function `format_verbatim_node` is never used
warning: function `format_markdown_verbatim_node` is never used
```

Additionally, the `format_markdown_verbatim_node` function has a bug: it uses `text_trimmed_range().len()` instead of `text_range_with_trivia().len()`, unlike all other language formatters.

## Analysis

- `format_bogus_node` and `format_suppressed_node` ARE used (in `lib.rs`)
- `format_verbatim_node` and `format_markdown_verbatim_node` are NOT used anywhere
- All 32 auxiliary formatters have proper implementations — no node falls back to verbatim
- The `prelude.rs` re-exports via `verbatim::*` which is fine (only exports used items after cleanup)

## Implementation

### Step 1: Delete unused functions

Remove `format_verbatim_node` and `format_markdown_verbatim_node` from `verbatim.rs`, keeping `format_bogus_node` and `format_suppressed_node`.

### Step 2: Verify

```bash
cargo check -p biome_markdown_formatter  # No warnings
cargo test -p biome_markdown_formatter   # All tests pass
```

## Files Changed

| File | Change |
|------|--------|
| `crates/biome_markdown_formatter/src/verbatim.rs` | Delete 2 unused functions |

## Risk: None
