# Parser: GFM Extensions

**Status:** Planned (research phase)
**Created:** 2026-02-09
**Effort:** Very High
**Impact:** Table lint rules benefit, unblocks table formatting

---

## Context

GitHub Flavored Markdown (GFM) extensions include tables, task lists, strikethrough, and autolinks. No AST node types are defined for tables. Text-based `table_utils.rs` exists for lint rules.

## Tables

### Current state

- NO grammar definitions for table nodes
- `table_utils.rs` provides `Table`, `TableRow` structs and `collect_tables()` for text scanning
- 5+ lint rules use table_utils: `noMismatchedTableColumnCount`, `noHiddenTableCell`, `noTableIndentation`, `useConsistentTableCellPadding`, `useConsistentTablePipeAlignment`, `useConsistentTablePipeStyle`

### Grammar needed (new definitions)

```ungram
MdTable =
    header: MdTableRow
    separator: MdTableSeparator
    body: MdTableRowList

MdTableRowList = MdTableRow*

MdTableRow =
    leading_pipe: '|'?
    cells: MdTableCellList
    trailing_pipe: '|'?

MdTableCellList = (MdTableCell ('|' MdTableCell)*)

MdTableCell = content: MdInlineItemList

MdTableSeparator = 'md_textual_literal'
```

### Implementation

1. Add grammar definitions to `markdown.ungram`
2. Add node kinds to `markdown_kinds_src.rs`
3. Run codegen
4. Add table detection in parser: check for `|` at line start followed by separator row on next line
5. Parse header row, separator row, and data rows

### Complexity

Tables are medium complexity — the format is rigid (pipe-separated cells) but need to handle:
- Optional leading/trailing pipes
- Escaped pipes in content
- Alignment markers in separator (`:---`, `:---:`, `---:`)

## Task Lists

Task list items are list items with checkbox: `- [ ] task` or `- [x] task`. These build on top of list parsing (plan 08). Once lists are parsed, task list detection is straightforward:
- Check if first content of a list item matches `[ ]` or `[x]` pattern
- Could be a property on `MdBullet` rather than a separate node type

## Strikethrough

GFM strikethrough: `~~text~~`. Similar to emphasis parsing. Grammar would be:
```ungram
MdInlineStrikethrough =
    l_fence: '~~'
    content: MdInlineItemList
    r_fence: '~~'
```

Need to add `~~` as a token kind and add to `AnyMdInline` union.

## Autolinks

GFM autolinks: bare URLs like `https://example.com` are automatically linked. This is primarily a rendering concern — the parser would need URL detection heuristics.

## Priority

1. Tables (highest — 5+ lint rules)
2. Task lists (depends on list parsing)
3. Strikethrough (low — similar to emphasis)
4. Autolinks (lowest — mostly rendering concern)

## Files to modify

| File | Change |
|------|--------|
| `xtask/codegen/markdown.ungram` | Add table grammar |
| `xtask/codegen/src/markdown_kinds_src.rs` | Add table node kinds |
| Generated files | Auto-regenerated |
| `crates/biome_markdown_parser/src/syntax.rs` | Table detection + parsing |
| `crates/biome_markdown_parser/src/lexer/mod.rs` | Pipe token handling |

## Verification

```bash
just gen-grammar
cargo test -p biome_markdown_parser
cargo test -p biome_markdown_analyze
```
