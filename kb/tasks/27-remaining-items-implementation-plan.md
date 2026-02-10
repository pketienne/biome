# Implementation Plan: Remaining Items 1–14

**Created:** 2026-02-10
**Source:** `kb/tasks/26-remaining-items.md`
**Last updated:** 2026-02-10

---

## Item 1: Migrate 7 table rules to `Ast<MdTable>` — COMPLETED

All 7 rules migrated from `Ast<MdDocument>` + `collect_tables()` to `Ast<MdTable>`:

| # | Rule file | Status |
|---|-----------|--------|
| 1a | `correctness/no_hidden_table_cell.rs` | Done — added `build_corrected_row()`, pre-computed fix text |
| 1b | `correctness/no_mismatched_table_column_count.rs` | Done — added `build_corrected_row()`, pre-computed fix text |
| 1c | `style/use_consistent_table_cell_padding.rs` | Done |
| 1d | `style/use_consistent_table_pipe_style.rs` | Done — fixed `MdTableRow: Copy` error using references |
| 1e | `style/use_consistent_table_pipe_alignment.rs` | Done |
| 1f | `style/use_blanks_around_tables.rs` | Done — uses `ctx.root()` for document text |
| 1g | `style/no_table_indentation.rs` | Done — uses `ctx.root()` for document text |

Post-migration: Deleted `table_utils.rs`, removed from `utils/mod.rs`.

**Known limitation:** The parser only recognizes tables with leading pipes as `MdTable` nodes. Tables without leading pipes are not parsed into the AST. This caused `useConsistentTablePipeStyle` to lose diagnostics for its "no leading pipe" test cases.

---

## Item 2: Clean up legacy definition functions — COMPLETED

Gated `collect_definitions()`, `parse_definition()`, and `parse_definition_title()` with `#[cfg(test)]`.

---

## Item 3: Remove unused `DocumentLines` struct — COMPLETED

Removed `DocumentLines` struct and its impl block from `line_utils.rs`. Kept `is_blank_line()` and `leading_indent()`.

---

## Item 4: Privatize internal helper functions — COMPLETED

Changed `parse_list_item()`, `heading_slug()`, and `extract_atx_heading_text()` to `pub(crate)`.

---

## Item 5: Migrate heading_utils to AST — COMPLETED

Rewrote `collect_heading_slugs()` to walk `MdHeader` and `MdSetextHeader` AST descendants instead of parsing text. Gated `extract_atx_heading_text()` with `#[cfg(test)]`. Updated `no_invalid_link_fragments.rs` call site.

---

## Item 6: Migrate list rules to AST — DEFERRED

12 list rules need document-level context for cross-item consistency checks (e.g., "all items in a list use the same marker style"). These don't cleanly map to per-node AST queries without significant architectural changes. Text-based `list_utils.rs` works correctly and is heavily used.

---

## Item 7: Migrate blockquote rules to AST — DEFERRED

2 blockquote rules need document-level context for continuation checking and cross-block consistency. Text-based `blockquote_utils.rs` works correctly.

---

## Item 8: Nested list grammar support — VERIFIED (already implemented)

Parser supports nested lists via recursive calls in `parse_bullet()`. AST correctly nests `MdBulletListItem`/`MdOrderListItem` nodes. Tests confirm.

---

## Item 9: Nested blockquote grammar support — VERIFIED (already implemented)

Parser handles `> > nested` blockquotes via recursive calls in `parse_blockquote()`. Tests confirm.

---

## Item 10: Lazy continuation in blockquotes — VERIFIED (already implemented)

Parser handles lazy continuation lines (without `>`) via `at_continuation_stop()` checks. Tests confirm.

---

## Item 11: GFM task list checkbox as AST node — VERIFIED (already implemented)

`MdCheckbox` AST node exists and `try_parse_checkbox()` handles `[ ]`, `[x]`, `[X]` formats. Tests confirm.

---

## Item 12: Table formatter improvements — COMPLETED

Enhanced `FormatMdTable` in `crates/biome_markdown_formatter/src/md/auxiliary/table.rs` to:
- Compute max column widths across all rows
- Format cells with aligned padding (`{:<width$}`)
- Normalize separator rows with matching dash counts
- Minimum column width of 3 characters

Used `std::format!` to avoid conflict with biome_formatter's custom `format!` macro.

---

## Item 13: `MdInlineImageLink` slot — DOCUMENTED (no action needed)

Investigation found:
- The parser **never constructs** `MdInlineImageLink` — the slot is always empty
- No analyzer rules reference it
- The formatter stub is dead code
- Linked images (`[![alt](img)](link)`) are naturally represented by nesting `MdInlineImage` inside `MdInlineLink`
- **Recommendation:** Remove the slot from the grammar in a future cleanup. No behavioral impact since it's never populated.

---

## Item 14: Further rule migrations / dead code cleanup — COMPLETED (no action needed)

Investigated all remaining utility modules:
- `list_utils.rs` — actively used by 12 rules, no dead code
- `blockquote_utils.rs` — actively used by 2 rules, no dead code
- `fence_utils.rs` — critical infrastructure used by 21+ files
- `inline_utils.rs` — extensively used by 22 rules, no dead code
- `heading_utils.rs` — used by 1 rule (migrated to AST), `extract_atx_heading_text` gated as test-only
- `table_utils.rs` — already deleted (Item 1)

All remaining `Ast<MdDocument>` rules legitimately need document scope. No additional migrations warranted.

---

## Summary

| Item | Status | Notes |
|------|--------|-------|
| 1. Table rules → `Ast<MdTable>` | COMPLETED | 7 rules migrated, `table_utils.rs` deleted |
| 2. Definition function cleanup | COMPLETED | Gated with `#[cfg(test)]` |
| 3. Remove `DocumentLines` | COMPLETED | Struct removed |
| 4. Privatize helpers | COMPLETED | Changed to `pub(crate)` |
| 5. Heading utils → AST | COMPLETED | `collect_heading_slugs` walks AST |
| 6. List rules → AST | DEFERRED | Complex; text-based utils work correctly |
| 7. Blockquote rules → AST | DEFERRED | Complex; text-based utils work correctly |
| 8. Nested lists | VERIFIED | Already implemented in parser |
| 9. Nested blockquotes | VERIFIED | Already implemented in parser |
| 10. Lazy continuation | VERIFIED | Already implemented in parser |
| 11. Checkboxes | VERIFIED | Already implemented in parser |
| 12. Table formatter | COMPLETED | Column-aligned formatting |
| 13. MdInlineImageLink | DOCUMENTED | Dead code; remove slot in future cleanup |
| 14. Dead code cleanup | COMPLETED | All utils actively used; no dead code found |
