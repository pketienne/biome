# Plan: Inline Whitespace Normalization

## Context

The markdown formatter currently strips trailing whitespace from paragraph lines and preserves hard breaks, but does **not** collapse multiple consecutive spaces/tabs between words to a single space. In rendered CommonMark, multiple spaces between words are equivalent to a single space, so collapsing them is a safe normalization.

This was listed as "blocked on GFM table parser" but investigation shows it's fully independent — it affects paragraphs, headings, list items, and blockquotes, all of which are already parsed and formatted.

## Approach

### 1. Add `collapse_inline_whitespace` function to `paragraph.rs`

Add a function that collapses runs of spaces/tabs to a single space within each line, while **skipping content inside backtick-delimited code spans** (both single and double-backtick).

```
"hello   world"         → "hello world"
"**bold**   text"       → "**bold** text"
"`code   here`  text"   → "`code   here` text"
"``code `x` here`` ok"  → "``code `x` here`` ok"
```

Pipeline order in paragraph formatter:
1. `normalize_newlines` (existing)
2. `collapse_inline_whitespace` (new)
3. `strip_trailing_whitespace` (existing)

### 2. Apply to text-blob formatters

The text-blob pattern (`node.syntax().text_trimmed()` → normalize → write) is used in these formatters that contain inline content:

| File | Needs collapsing | Reason |
|------|:---:|--------|
| `paragraph.rs` | Yes | Paragraphs + heading content |
| `bullet_list_item.rs` | Yes | Unordered list items |
| `order_list_item.rs` | Yes | Ordered list items |
| `quote.rs` | Yes | Blockquote content |
| `inline_code.rs` | **No** | Code span content is literal |
| `fenced_code_block.rs` | **No** | Code block content is literal |
| `indent_code_block.rs` | **No** | Code block content is literal |
| `header.rs` | N/A | Delegates to paragraph via `content.format()` |
| `setext_header.rs` | N/A | Delegates to paragraph via `content().format()` |
| `inline_emphasis.rs` | **No** | Inside paragraphs, handled by paragraph's text blob |
| `inline_italic.rs` | **No** | Same — inside paragraphs |
| `inline_link.rs` | **No** | Same — inside paragraphs |
| `inline_image.rs` | **No** | Same — inside paragraphs |

### 3. Add test spec

Create `crates/biome_markdown_formatter/tests/specs/md/paragraphs/inline_whitespace.md`:
```md
Hello   world   here.

Multiple    spaces   in   heading content.

**bold**   and   *italic*   text.

`code   preserved`   outside   collapsed.

- list   item   spaces
- another   item

> quote   with   spaces

1. ordered   item   spaces
```

### 4. Unit tests

Add unit tests for `collapse_inline_whitespace` in `paragraph.rs` (same pattern as existing `strip_trailing_whitespace` tests).

### 5. Update snapshots

Run `cargo insta accept --workspace` to update affected snapshots.

## Files to modify

- `crates/biome_markdown_formatter/src/md/auxiliary/paragraph.rs` — add function + apply in pipeline + unit tests
- `crates/biome_markdown_formatter/src/md/auxiliary/bullet_list_item.rs` — apply collapsing
- `crates/biome_markdown_formatter/src/md/auxiliary/order_list_item.rs` — apply collapsing
- `crates/biome_markdown_formatter/src/md/auxiliary/quote.rs` — apply collapsing
- `crates/biome_markdown_formatter/tests/specs/md/paragraphs/inline_whitespace.md` — new test spec

## Verification

1. `cargo test -p biome_markdown_formatter` — all formatter tests pass
2. `cargo test -p biome_markdown_analyze` — all analyzer tests pass
3. `cargo test -p biome_markdown_parser` — all parser tests pass
4. `cargo test -p biome_cli` — all CLI tests pass
5. Inspect snapshot diffs to confirm only whitespace collapsing changes
