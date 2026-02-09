# Plan: Add `keepUnusedPrefixes` Option to `noUnusedPrefix` Rule

**Date:** 2026-02-09
**Status:** Implementation

## Context

Add a `keepUnusedPrefixes` boolean option to the existing `noUnusedPrefix` lint rule. When set to `true`, the rule emits no diagnostics — effectively disabling unused prefix detection without turning the rule off entirely. This is useful when a document intentionally declares prefixes for documentation/template purposes.

The option goes on the existing `NoUnusedPrefixOptions` struct, which already has `ignoredPrefixes`. This follows the established pattern — rule options flow through the global linter config, not language-specific settings.

---

## Step 1: Add `keepUnusedPrefixes` to options struct

**Modify** `crates/biome_rule_options/src/no_unused_prefix.rs`

Add a boolean field:

```rust
pub struct NoUnusedPrefixOptions {
    /// Prefix namespaces to ignore (e.g., `["owl:", "skos:"]`).
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub ignored_prefixes: Option<Box<[Box<str>]>>,

    /// When `true`, unused prefix declarations are not flagged.
    /// Default: `false`.
    #[serde(skip_serializing_if = "Option::<_>::is_none")]
    pub keep_unused_prefixes: Option<bool>,
}
```

Update the `Merge` impl to handle the new field.

---

## Step 2: Short-circuit the rule when enabled

**Modify** `crates/biome_turtle_analyze/src/lint/nursery/no_unused_prefix.rs`

At the start of `run()`, check the option and return early:

```rust
fn run(ctx: &RuleContext<Self>) -> Self::Signals {
    let options = ctx.options();
    if options.keep_unused_prefixes.unwrap_or(false) {
        return Vec::new();
    }
    // ... existing logic
}
```

Update the rustdoc with the new option example.

---

## Files to Modify

| File | Change |
|------|--------|
| `crates/biome_rule_options/src/no_unused_prefix.rs` | Add `keep_unused_prefixes: Option<bool>` field + update Merge impl |
| `crates/biome_turtle_analyze/src/lint/nursery/no_unused_prefix.rs` | Early return when option is true + update rustdoc |

No new files needed. No changes to diagnostic categories, options.rs, or lib.rs.

---

## Verification

1. `cargo build -p biome_turtle_analyze` — compiles
2. `cargo test -p biome_turtle_analyze` — all 63 tests pass (no behavior change with default options)
3. Review the rustdoc for correct JSON example
