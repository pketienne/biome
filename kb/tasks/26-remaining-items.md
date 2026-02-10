# Remaining Items

**Created:** 2026-02-10

---

## Easy Wins (No Grammar Changes)

### 1. Migrate 7 table rules to `Ast<MdTable>`

The AST now has structured `MdTableCell`/`MdTableCellList` nodes. These rules can stop using `table_utils.rs` text scanning and query the AST directly.

| Rule | New Query | Change |
|------|-----------|--------|
| `no_hidden_table_cell` | `Ast<MdTable>` | Compare `row.cells().len()` between header and data rows |
| `no_mismatched_table_column_count` | `Ast<MdTable>` | Count cells per row |
| `use_consistent_table_cell_padding` | `Ast<MdTable>` | Inspect cell content spacing |
| `use_consistent_table_pipe_style` | `Ast<MdTable>` | Check leading/trailing pipe presence |
| `use_consistent_table_pipe_alignment` | `Ast<MdTable>` | Check pipe column positions |
| `use_blanks_around_tables` | `Ast<MdTable>` | Check sibling spacing |
| `no_table_indentation` | `Ast<MdTable>` | Check leading whitespace |

After migration, delete `table_utils.rs` and remove from `utils/mod.rs`.

### 2. Clean up legacy definition functions

In `definition_utils.rs`, `collect_definitions()` and `parse_definition()` are only used in unit tests. They are superseded by `collect_definitions_from_ast()`. Either delete them or mark them `#[cfg(test)]`.

### 3. Remove unused `DocumentLines` struct

In `line_utils.rs`, the `DocumentLines` struct is never used by any lint rule — only in its own unit tests. Remove it.

### 4. Privatize internal helper functions

Several public functions are only used internally or in tests:
- `parse_list_item()` in `list_utils.rs` — only used by `collect_list_items()` and tests
- `heading_slug()` and `extract_atx_heading_text()` in `heading_utils.rs` — only used by `collect_heading_slugs()` and tests
- `is_separator_row()` and `is_table_row()` in `table_utils.rs` — only used by `collect_tables()`

---

## Medium Effort

### 5. Migrate `heading_utils` to AST

Rewrite `collect_heading_slugs()` in `heading_utils.rs` to walk `MdHeader` and `MdSetextHeader` descendants instead of text-parsing. Only used by `no_invalid_link_fragments`.

### 6. Migrate list rules to AST

~8 rules use `list_utils.rs` text-based parsing. Would benefit from structured list item nodes but currently works fine.

| Rule | Current Utility |
|------|----------------|
| `use_consistent_list_item_spacing` | `collect_list_blocks()` |
| `use_consistent_list_item_indent` | `collect_list_items()` |
| `use_consistent_list_item_content_indent` | `collect_list_items()` |
| `use_consistent_list_indent` | `collect_list_blocks()` |
| `use_consistent_ordered_list_marker` | `collect_list_items()` |
| `use_consistent_ordered_list_marker_value` | `collect_list_items()` |
| `use_consistent_unordered_list_marker` | `collect_list_items()` |
| `use_consistent_unordered_list_indent` | `collect_list_items()` |
| `no_list_item_bullet_indent` | `collect_list_items()` |
| `use_blanks_around_lists` | `collect_list_blocks()` |

### 7. Migrate blockquote rules to AST

2 rules use `blockquote_utils.rs` text-based parsing:
- `no_blockquote_broken_continuation`
- `use_consistent_blockquote_indent`

---

## Larger Items

### 8. Nested list grammar support

Sub-items at deeper indentation levels are currently flat in the AST. Adding nested `MdBulletList`/`MdOrderList` inside `MdBulletListItem`/`MdOrderListItem` would allow rules to query list depth directly.

### 9. Nested blockquote grammar support

`> > nested` blockquotes are parsed as a single `MdQuote` node. Structuring as nested `MdQuote` nodes would improve AST fidelity.

### 10. Lazy continuation in blockquotes

Lines without `>` after a blockquote line are valid continuation lines per CommonMark spec. Parser currently doesn't handle this — lint rules work around it via text analysis.

### 11. GFM task list checkbox as AST node

`- [ ] task` items have `MdCheckbox` in the grammar but the parser doesn't always produce it correctly for all checkbox formats. Improvement would help checkbox-related lint rules.

### 12. Table formatter improvements

The `table_cell.rs` and `table_cell_list.rs` formatters are currently stubs that emit raw text. Could be enhanced to:
- Normalize cell padding
- Align columns
- Normalize separator rows

### 13. `MdInlineImageLink` slot

The `MdInlineImageLink` slot in `MdInlineImage` is never populated by the parser. Either implement it (for `[![alt](img)](link)` patterns) or remove from grammar.

### 14. Further rule migrations from `Ast<MdDocument>`

~27 rules still use `Ast<MdDocument>` with text-based scanning. Many legitimately need document scope (consistency rules, cross-reference rules), but some could be narrowed:
- Cross-paragraph consistency rules (emphasis/strong/strikethrough markers)
- Whole-document rules (hard tabs, long lines, final newline, first-line heading)
- Heading style rules
