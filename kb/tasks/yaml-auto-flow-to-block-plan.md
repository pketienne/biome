# Auto Flow-to-Block with `best_fitting!` for `Expand::Auto`

## Date: 2026-02-09

## Problem

`Expand::Auto` currently uses `group()` for flow collections, which only toggles between
flat flow (`[a, b, c]`) and expanded flow (`[\n  a,\n  b,\n  c\n]`). It never converts
to block style (`- a\n- b\n- c`). The `best_fitting!` macro would let the printer
automatically choose between compact flow and block style based on line width, but
Biome's token-tracking system panics when the same AST token is formatted in multiple
`best_fitting!` variants.

## Root Cause

`BestFitting::fmt()` (`biome_formatter/src/builders.rs:2670`) writes ALL variants into
a shared `VecBuffer` backed by one `PrintedTokens` (`printed_tokens.rs`). The
`PrintedTokens` set tracks each token's start offset. When variant 1 formats token X
and variant 2 also formats token X, the second insert panics:

```
You tried to print the token 'PLAIN_LITERAL@6..7 "a"' twice
```

## Solution: Per-Entry Memoization (Solution A)

Use `.memoized()` to format each entry and bracket token exactly **once**, producing
cached `Interned` IR elements. Subsequent writes of a `Memoized` value emit the cached
element without re-tracking tokens. Then use `best_fitting!` with two variants that
reference only memoized/synthetic elements.

This is the same technique JSX uses (`jsx/tag/element.rs:69,96`) — memoize shared
tokens, pre-build variant-specific IR from memoized parts.

### Key Insight

- `entry.format().memoized()` → formats the entry once (tracks tokens), caches IR
- Writing a `Memoized` a second time → emits cached `FormatElement::Interned`, no re-tracking
- Bracket tokens: memoized, appear in compact variant only — but tokens are tracked during
  memoization regardless of which variant the printer picks, satisfying `assert_all_tracked`
- Commas: removed upfront via `format_removed()` (tracked once), then re-synthesized as
  `token(",")` (synthetic, not tracked) in the compact variant

### Why Commas Must Be Removed Upfront

The comma tokens exist in the AST as separators. If we don't account for them, the
end-of-format `assert_all_tracked` check panics with "token has not been seen." We must
either `format()` or `format_removed()` every AST token. Since the block variant has no
commas, we `format_removed()` all commas upfront (before `best_fitting!`), then use
synthetic `token(",")` in the compact variant to visually restore them.

## Implementation

### File: `crates/biome_yaml_formatter/src/yaml/auxiliary/flow_sequence.rs`

Replace the `Expand::Auto` arm:

```rust
Expand::Auto => {
    // 1. Collect entry nodes (owned values from separated list iterator)
    let entry_nodes: Vec<_> = entries.iter().flatten().collect();

    // 2. Memoize each entry — tokens tracked once during memoization
    let mut memo_entries: Vec<_> = entry_nodes
        .iter()
        .map(|e| e.format().memoized())
        .collect();

    // 3. Memoize bracket tokens
    let mut l_brack = node.l_brack_token()?.format().memoized();
    let mut r_brack = node.r_brack_token()?.format().memoized();

    // 4. Remove all comma separators upfront (tracked once as removed)
    for element in entries.elements() {
        if let Some(separator) = element.trailing_separator()? {
            write!(f, [format_removed(&separator)])?;
        }
    }

    // 5. Inspect memoized entries to trigger interning
    //    (ensures tokens are tracked before best_fitting! evaluation)
    for memo in &mut memo_entries {
        memo.inspect(f)?;
    }
    l_brack.inspect(f)?;
    r_brack.inspect(f)?;

    // 6. best_fitting! with two variants — all tokens already tracked
    write!(
        f,
        [best_fitting!(
            // Variant 1 (widest): compact flow [a, b, c]
            format_with(|f| {
                write!(f, [&l_brack, space()])?;
                for (i, memo) in memo_entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, [token(","), space()])?;
                    }
                    write!(f, [memo])?;
                }
                write!(f, [space(), &r_brack])
            }),
            // Variant 2 (narrowest): block sequence style
            format_with(|f| {
                for memo in &memo_entries {
                    write!(f, [hard_line_break(), token("- "), memo])?;
                }
                Ok(())
            }),
        )]
    )
}
```

### File: `crates/biome_yaml_formatter/src/yaml/auxiliary/flow_mapping.rs`

Same pattern, without the `- ` prefix in the block variant:

```rust
Expand::Auto => {
    let entry_nodes: Vec<_> = entries.iter().flatten().collect();
    let mut memo_entries: Vec<_> = entry_nodes
        .iter()
        .map(|e| e.format().memoized())
        .collect();

    let mut l_curly = node.l_curly_token()?.format().memoized();
    let mut r_curly = node.r_curly_token()?.format().memoized();

    for element in entries.elements() {
        if let Some(separator) = element.trailing_separator()? {
            write!(f, [format_removed(&separator)])?;
        }
    }

    for memo in &mut memo_entries {
        memo.inspect(f)?;
    }
    l_curly.inspect(f)?;
    r_curly.inspect(f)?;

    write!(
        f,
        [best_fitting!(
            // Variant 1 (widest): compact flow {a: 1, b: 2}
            format_with(|f| {
                write!(f, [&l_curly, space()])?;
                for (i, memo) in memo_entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, [token(","), space()])?;
                    }
                    write!(f, [memo])?;
                }
                write!(f, [space(), &r_curly])
            }),
            // Variant 2 (narrowest): block mapping style
            format_with(|f| {
                for memo in &memo_entries {
                    write!(f, [hard_line_break(), memo])?;
                }
                Ok(())
            }),
        )]
    )
}
```

### File: `crates/biome_yaml_formatter/src/lib.rs`

Add/update tests:

```rust
#[test]
fn flow_sequence_auto_fits_on_line() {
    let src = "key: [1, 2, 3]\n";
    let parse = parse_yaml(src);
    let options = YamlFormatOptions::default(); // Expand::Auto, line_width=80
    let result = format_node(options, &parse.syntax()).unwrap().print().unwrap();
    // Fits on line → stays compact flow
    assert_eq!(result.as_code(), "key: [1, 2, 3]\n");
}

#[test]
fn flow_sequence_auto_exceeds_line_width() {
    use biome_formatter::LineWidth;
    let src = "key: [aaaa, bbbb, cccc, dddd]\n";
    let parse = parse_yaml(src);
    let options = YamlFormatOptions::default()
        .with_line_width(LineWidth::try_from(20).unwrap());
    let result = format_node(options, &parse.syntax()).unwrap().print().unwrap();
    // Exceeds line width → converts to block
    assert_eq!(result.as_code(), "key:\n  - aaaa\n  - bbbb\n  - cccc\n  - dddd\n");
}

#[test]
fn flow_mapping_auto_exceeds_line_width() {
    use biome_formatter::LineWidth;
    let src = "key: {alpha: 1, bravo: 2, charlie: 3}\n";
    let parse = parse_yaml(src);
    let options = YamlFormatOptions::default()
        .with_line_width(LineWidth::try_from(20).unwrap());
    let result = format_node(options, &parse.syntax()).unwrap().print().unwrap();
    // Exceeds line width → converts to block
    assert_eq!(result.as_code(), "key:\n  alpha: 1\n  bravo: 2\n  charlie: 3\n");
}
```

## Token Tracking Flow

```
Step 1: entry.format().memoized()
  → f.intern() → VecBuffer writes entry IR → track_token(PLAIN_LITERAL@6..7)
  → Returns Interned(cached_ir)

Step 2: format_removed(&comma)
  → track_token(COMMA@7..9) — marked as removed

Step 3: l_brack.format().memoized()
  → f.intern() → track_token(L_BRACK@5..6)
  → Returns Interned(cached_ir)

Step 4: best_fitting! variant 1 writes [&l_brack, memo_entry, ...]
  → Memoized::fmt() → writes cached FormatElement::Interned
  → NO re-tracking (already tracked in steps 1+3)

Step 5: best_fitting! variant 2 writes [memo_entry, ...]
  → Same: writes cached Interned, no re-tracking

Step 6: assert_all_tracked()
  → L_BRACK ✓ (step 3), PLAIN_LITERAL ✓ (step 1), COMMA ✓ (step 2), R_BRACK ✓ (step 3)
  → All tokens accounted for ✓
```

## Risks and Edge Cases

### 1. Compact variant spacing differs from current `entries.format()`

Current `entries.format()` uses `soft_line_break_or_space()` between entries (collapses
to space in flat mode). The memoized approach uses `token(","), space()` — a hard comma
and space. In flat mode this is identical: `a, b, c`. When expanded (inside a group), the
soft break would become a line break, but `token(","), space()` stays as `, `. This is
acceptable since `best_fitting!` chooses between flat compact and block — there's no
intermediate "expanded flow" variant.

### 2. Nested flow collections

A flow sequence entry can itself be a flow sequence: `[[1, 2], [3, 4]]`. The inner
sequences are memoized as single entries. If the inner sequence also uses `Expand::Auto`,
it would recursively apply the same `best_fitting!` logic. This should work correctly
since each level memoizes independently.

### 3. Comments attached to commas

If a comma token has trailing trivia (comments), `format_removed()` discards it. This
could lose inline comments like `[a, # comment\n b]`. Mitigation: check if the comma has
non-whitespace trailing trivia and, if so, emit the trivia before the next entry. This is
a follow-up enhancement, not a blocker.

### 4. `inspect()` requirement

`Memoized::inspect()` must be called before `best_fitting!` to trigger interning. Without
it, the first call to `Memoized::fmt()` inside `best_fitting!` would track tokens, and
the second variant would double-track. Calling `inspect()` upfront ensures tokens are
tracked in the main formatter state, and both variants only write cached IR.

### 5. Idempotency

The block variant output (`- a\n- b\n- c`) is NOT valid flow syntax — it's block syntax.
Re-parsing and re-formatting this output would go through the block sequence formatter,
not the flow sequence formatter. This means the output is stable (idempotent) only if the
re-parsed block sequence produces identical output. This should be verified with
`CheckReformat` in tests.

## Verification

1. `cargo build -p biome_yaml_formatter` — compiles
2. `cargo test -p biome_yaml_formatter` — all tests pass
3. New tests verify:
   - Compact flow preserved when content fits on line
   - Block conversion triggers when content exceeds line width
   - Nested flow collections handled correctly
   - Empty collections unchanged
   - `Expand::Always` still forces block (unchanged)
   - `Expand::Never` still keeps flow (unchanged)
4. Idempotency: formatted output re-formats identically

## Files Summary

| File | Action |
|------|--------|
| `crates/biome_yaml_formatter/src/yaml/auxiliary/flow_sequence.rs` | Replace `Expand::Auto` arm with memoized `best_fitting!` |
| `crates/biome_yaml_formatter/src/yaml/auxiliary/flow_mapping.rs` | Same pattern for mappings |
| `crates/biome_yaml_formatter/src/lib.rs` | Add unit tests for auto flow-to-block |
