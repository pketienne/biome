# Expand Markdown Formatter

**Status:** Planned
**Created:** 2026-02-09
**Effort:** Medium
**Impact:** More useful `biome format` for markdown files

---

## Context

The markdown formatter currently has real formatting logic for only 4 of 28 auxiliary node types:
- `document.rs` — BOM/EOF handling, final newline
- `header.rs` — ATX heading space normalization
- `hash.rs` — hash token formatting
- `thematic_break_block.rs` — normalize to `---`

The remaining 24 formatters use `format_verbatim_node()` (passthrough). The block list uses `join_nodes_with_hardline()` for blank line preservation.

## Current State

### Formatted (4 files)
- `document.rs` — delegates to children, adds final newline
- `header.rs` — hashes + space + content, skips trailing hashes
- `hash.rs` — hash token
- `thematic_break_block.rs` — normalizes `***`/`___`/`- - -` to `---`

### Verbatim (24 files)
- `bullet_list_item.rs`, `bullet.rs` — list items
- `fenced_code_block.rs` — code fences
- `hard_line.rs`, `soft_break.rs` — line breaks
- `html_block.rs` — raw HTML
- `indent_code_block.rs`, `indented_code_line.rs`, `indent.rs` — indented code
- `inline_code.rs`, `inline_emphasis.rs`, `inline_italic.rs` — inline formatting
- `inline_image.rs`, `inline_image_alt.rs`, `inline_image_link.rs`, `inline_image_source.rs` — images
- `inline_link.rs` — links
- `link_block.rs` — link reference definitions
- `order_list_item.rs` — ordered list items
- `paragraph.rs` — paragraphs
- `quote.rs` — blockquotes
- `setext_header.rs` — setext headings
- `textual.rs` — text content

### Available AST node fields

- **MdFencedCodeBlock**: `l_fence_token()`, `code_list()`, `l_hard_line()`, `content()`, `r_hard_line()`, `r_fence_token()`
- **MdBulletListItem**: `bullet()`, `body()`
- **MdOrderListItem**: `bullet()`, `body()`
- **MdQuote**: no typed fields (verbatim content)
- **MdParagraph**: `list: MdInlineItemList`, `hard_line: MdHardLine`

## Proposed Expansion (ordered by value/feasibility)

### Phase A: Code fence normalization

**Files**: `fenced_code_block.rs`

Normalize fence markers (backticks vs tildes), ensure consistent style. The node has `l_fence_token` and `r_fence_token` which can be replaced.

- Normalize ``` ~~~ ``` to ` ``` ` (or configurable)
- Preserve language identifier
- Preserve content verbatim
- Ensure closing fence matches opening style

### Phase B: Trailing whitespace removal

**Files**: `paragraph.rs`, `textual.rs`

Remove trailing whitespace from text lines (except intentional hard breaks with `  \n`). This works even with verbatim content by post-processing the text.

### Phase C: Setext heading normalization

**Files**: `setext_header.rs`

Normalize setext heading underlines to match content width, or optionally convert to ATX style.

### Phase D: Inline whitespace normalization (future, needs parser work)

Would need parser improvements to properly handle emphasis, links, code spans at the AST level. Currently all flattened to `MdTextual` tokens.

## Additional warnings to note

24 auxiliary files have unused `use biome_rowan::AstNode;` imports (generated code artifact). These should be cleaned in plan #1 but won't block formatter expansion.

## Test strategy

- Add spec test fixtures for each phase under `tests/specs/md/`
- Run `cargo insta test -p biome_markdown_formatter --accept`
- Verify idempotency (format twice, same result)

## Verification

```bash
cargo test -p biome_markdown_formatter --lib
cargo test -p biome_markdown_formatter --test spec_tests
```
