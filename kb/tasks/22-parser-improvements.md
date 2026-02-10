# Plan: Parser Improvements

**Created:** 2026-02-10
**Status:** Partially Complete

---

## Goal

Improve the markdown parser to handle:
1. Nested blockquotes (`> > nested`) — **DONE**
2. Lazy continuation in blockquotes (lines without `>`) — **DONE**
3. Nested lists (sub-items at deeper indentation) — **DONE** (unordered only)
4. Table cell-level parsing (use existing `MD_TABLE_CELL` kind) — **DEFERRED** to task #17

Checkbox parsing is already implemented and working.

## What Was Implemented

### Nested Blockquotes
After consuming the outer `>`, if the next token on the same line is also `>`, `parse_blockquote` recursively calls itself. This creates a proper AST:
```
MdQuote > MdBlockList > MdQuote > MdBlockList > MdParagraph
```
Triple nesting (`> > > text`) also works via recursion.

### Lazy Continuation
The blockquote continuation loop now has three branches:
1. Line starts with `>` → standard continuation (consume `>` and rest of line)
2. Line doesn't start with `>` AND `at_continuation_stop()` is false → lazy continuation
3. Otherwise → stop the blockquote

The lazy continuation correctly stops at block-level elements (headings, list markers, blockquotes) via the existing `at_continuation_stop()` check.

**Bug fix:** The first token on a lazy continuation line has `has_preceding_line_break() == true`, which would cause an infinite loop if not handled. Fixed by unconditionally bumping the first token before entering the line-consumption loop.

### Nested Unordered Lists
Three key changes:
1. `parse_unordered_list_item` tracks the indent of the first marker and uses `at_bullet_start_at_indent` to only collect siblings at the **same** indent level (was: any indent < 4)
2. `parse_bullet` accepts a `list_indent` parameter, computes `content_indent = list_indent + 2`, and after parsing the paragraph content, checks for nested list markers at >= `content_indent`
3. Same changes applied to `parse_ordered_list_item` and `parse_order_bullet`

### Known Limitations
- **Ordered list detection broken (pre-existing):** The lexer tokenizes `1.` as two separate tokens `"1"` and `"."`, so `is_ordered_marker` never matches. This means `parse_ordered_list_item` is never called from `parse_any_block`. Ordered lists are parsed as flat paragraphs. This is a lexer-level issue that needs to be fixed separately.
- **Double nesting at indent >= 4:** `at_unordered_list` checks `before_whitespace_count() < 4`, so nested lists at indent 4+ are treated as indent code blocks. Only one level of nesting is supported.
- **Multi-line nested blockquotes:** `> > line1` followed by `> > line2` — the second line's inner `>` is consumed as textual content of the first-line inner blockquote's continuation, not as a proper nested structure.
- **Table cell parsing deferred:** Requires grammar changes to `MdTableRow` and codegen updates.

## Files Changed

| File | Change |
|------|--------|
| `crates/biome_markdown_parser/src/syntax.rs` | Blockquote nesting + lazy continuation, nested unordered lists |
| `crates/biome_markdown_parser/tests/md_test_suite/ok/blockquote_nested.md` | New test |
| `crates/biome_markdown_parser/tests/md_test_suite/ok/blockquote_lazy.md` | New test |
| `crates/biome_markdown_parser/tests/md_test_suite/ok/list_nested.md` | New test |

## Test Results

- Parser: 17/17 passed (14 existing unchanged + 3 new)
- Formatter: 10/10 passed
- Analyzer: 200/200 passed
- CLI: 13/13 passed
