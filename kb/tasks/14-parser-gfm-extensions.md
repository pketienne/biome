# Plan: GFM Extensions (Tables, Strikethrough, Task Lists)

## Context

GFM (GitHub Flavored Markdown) extensions are the highest-impact remaining parser work. 10 lint rules currently use text-based scanning (`table_utils.rs`, `list_utils.rs`) because no AST nodes exist for tables, strikethrough, or task list checkboxes. Adding parser-level support creates proper syntax tree nodes, enables structural formatting, and improves lint accuracy.

## Scope

Three GFM features, in implementation order:

1. **Strikethrough** (inline, ~50 lines parser) — follows emphasis/italic pattern
2. **Tables** (block, ~200 lines parser) — most impactful, 7+ lint rules
3. **Task list checkboxes** (extends list items, ~30 lines parser) — 2 lint rules

## Implementation Steps

### Step 1: Grammar changes (`xtask/codegen/markdown.ungram`)

Add to `AnyLeafBlock`:
```ungram
AnyLeafBlock = ... | MdTable
```

Add to `AnyMdInline`:
```ungram
AnyMdInline = ... | MdInlineStrikethrough
```

Add new node definitions:
```ungram
// ~~strikethrough~~
MdInlineStrikethrough =
  l_fence: '~'
  content: MdInlineItemList
  r_fence: '~'

// GFM Table
MdTable =
  header: MdTableRow
  separator: MdTextual
  rows: MdTableRowList

MdTableRowList = MdTableRow*

MdTableRow =
  cells: MdTableCellList

MdTableCellList = MdTableCell*

MdTableCell = content: MdInlineItemList
```

### Step 2: Syntax kinds (`xtask/codegen/src/markdown_kinds_src.rs`)

Add to `punct`: `("~~", "DOUBLE_TILDE")`

Add to `nodes`:
```
"MD_INLINE_STRIKETHROUGH",
"MD_TABLE",
"MD_TABLE_ROW",
"MD_TABLE_ROW_LIST",
"MD_TABLE_CELL",
"MD_TABLE_CELL_LIST",
```

### Step 3: Run codegen

```bash
just gen-grammar
```

### Step 4: Parser — Strikethrough (`syntax.rs`)

Inline strikethrough parsing follows emphasis/italic pattern. Integrate into `parse_inline_list()` dispatch before the emphasis check.

### Step 5: Parser — Tables (`syntax.rs`)

Table detection uses `try_parse` with lookahead. Integrate into `parse_any_block()` before the paragraph fallback. Key challenge: table detection requires peeking at next line via checkpoint/rewind.

### Step 6: Parser — Task Lists (`syntax.rs`)

Existing list parsing already consumes `[ ]`/`[x]` as inline content. Ensure parser doesn't break on checkbox syntax.

### Step 7: Formatter stubs

Text-blob pattern for new nodes (consistent with existing formatters).

### Step 8: Tests

Parser and formatter spec tests for tables and strikethrough.

## Verification

```bash
just gen-grammar
cargo test -p biome_markdown_parser
cargo test -p biome_markdown_formatter
cargo test -p biome_markdown_analyze
cargo test -p biome_cli
```
