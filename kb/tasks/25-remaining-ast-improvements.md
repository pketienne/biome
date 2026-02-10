# Plan: Remaining Markdown AST Improvements

**Created:** 2026-02-10

---

## Context

The markdown parser, formatter, and 100 lint rules are functionally complete. However, 20 rules still use `Ast<MdDocument>` with text-based FenceTracker line scanning instead of querying specific AST nodes. Several parser structures are flat where they should be structured (link definitions, code block info strings, table cells), and ordered list tokenization is broken.

This plan covers all remaining items from `kb/tasks/` (tasks 19, 21, 22, 23) in a consolidated, dependency-ordered sequence.

## Phase 1: Quick Wins (No Grammar Changes)

### 1a. Fix Ordered List Lexer Tokenization

**Problem:** `consume_textual()` in the lexer consumes one character at a time, so `1.` becomes `"1"` + `"."` tokens. `is_ordered_marker()` in the parser expects a single token like `"1."`.

**File:** `crates/biome_markdown_parser/src/lexer/mod.rs`

**Change:** Add `consume_digit_sequence()` that:
1. Consumes consecutive ASCII digits
2. If followed by `.` or `)`, includes it in the same token
3. Returns `MD_TEXTUAL_LITERAL`

In `consume_token()` (~line 179), add a match arm before the default:
```rust
b'0'..=b'9' => self.consume_digit_sequence(),
```

**Verification:** Parser test with ordered lists should now produce `MdOrderListItem` nodes instead of flat paragraphs. Existing `list_ordered.md` test snapshot will change.

### 1b. Migrate `noShellDollarPrompt` to `Ast<MdFencedCodeBlock>`

**File:** `crates/biome_markdown_analyze/src/lint/style/no_shell_dollar_prompt.rs`

**Change:**
- `type Query = Ast<MdFencedCodeBlock>` (was `Ast<MdDocument>`)
- Access `node.code_list()` for language, `node.content()` for code content
- Remove FenceTracker usage
- Check if language is shell-like (`sh`, `bash`, `shell`, `zsh`, etc.)
- Scan content lines for `$ ` prefix

### 1c. Migrate `useBlanksAroundCodeFences` to `Ast<MdFencedCodeBlock>`

**File:** `crates/biome_markdown_analyze/src/lint/style/use_blanks_around_code_fences.rs`

**Change:**
- `type Query = Ast<MdFencedCodeBlock>`
- Check preceding/following siblings via `node.syntax().prev_sibling()` / `next_sibling()`
- Look for blank line in trivia between siblings
- Remove FenceTracker + line_utils usage

### 1d. Migrate `useConsistentMdxJsxQuoteStyle` to `Ast<MdMdxJsxElement>`

**File:** `crates/biome_markdown_analyze/src/lint/style/use_consistent_mdx_jsx_quote_style.rs`

**Change:**
- `type Query = Ast<MdMdxJsxElement>`
- Iterate `node.attributes()`, inspect `attr.value()?.delimiter_token()` for quote character
- Track first-seen style, flag mismatches
- Remove FenceTracker + mdx_utils usage

### 1e. Migrate `noMdxJsxVoidChildren` to `Ast<MdMdxJsxElement>`

**File:** `crates/biome_markdown_analyze/src/lint/correctness/no_mdx_jsx_void_children.rs`

**Change:**
- `type Query = Ast<MdMdxJsxElement>`
- Check if tag name is a void HTML element (`br`, `hr`, `img`, `input`, etc.)
- Check if element has no self-closing slash AND has content after closing `>`
- Remove FenceTracker + mdx_utils usage

### 1f. Delete `mdx_utils.rs`

After 1d and 1e, no rules import from `mdx_utils`. Delete it and remove from `utils/mod.rs`.

**Tests:** `cargo test -p biome_markdown_parser -p biome_markdown_analyze`

---

## Phase 2: Structured Link Definitions

### 2a. Grammar Change

**File:** `xtask/codegen/markdown.ungram`

Update `MdLinkBlock`:
```ungram
MdLinkBlock =
    '['
    label: MdInlineItemList
    ']'
    ':'
    url: MdInlineItemList
    title: MdLinkBlockTitle?

MdLinkBlockTitle =
    delimiter: 'md_textual_literal'
    content: MdInlineItemList
    closing_delimiter: 'md_textual_literal'
```

**File:** `xtask/codegen/src/markdown_kinds_src.rs` — add `MD_LINK_BLOCK_TITLE` node kind.

### 2b. Codegen + Formatter

Run codegen. Add formatter stub for `MdLinkBlockTitle`.

### 2c. Parser Update

**File:** `crates/biome_markdown_parser/src/syntax.rs`, function `try_parse_link_definition` (~line 1544)

Change slot 4 (url) to:
1. Parse URL tokens until whitespace or end of line -> `url: MdInlineItemList`
2. If remaining tokens on the line, check for `"`, `'`, or `(` as title delimiter
3. If title delimiter found, parse `MdLinkBlockTitle` (delimiter + content + closing delimiter)

### 2d. Migrate Definition Rules

6 rules to update:

| Rule | New Query | Change |
|------|-----------|--------|
| `no_duplicate_definitions` | `Ast<MdLinkBlock>` | Compare `node.label()` text directly |
| `no_duplicate_defined_urls` | `Ast<MdLinkBlock>` | Compare `node.url()` text directly |
| `no_definition_spacing_issues` | `Ast<MdLinkBlock>` | Inspect trivia around `:` and url tokens |
| `use_lowercase_definition_labels` | `Ast<MdLinkBlock>` | Check `node.label()` case |
| `use_sorted_definitions` | Keep `Ast<MdDocument>` | Walk all `MdLinkBlock` descendants, compare label order |
| `use_definitions_at_end` | Keep `Ast<MdDocument>` | Check position of `MdLinkBlock` nodes in block list |

### 2e. Reduce `definition_utils.rs`

Delete parsing functions (`collect_definitions`, `parse_definition`). Keep `normalize_label()` -- still used by cross-reference rules.

**Tests:** `cargo test -p biome_markdown_parser -p biome_markdown_formatter -p biome_markdown_analyze`

---

## Phase 3: Structured Code Block Info String

### 3a. Grammar Change

**File:** `xtask/codegen/markdown.ungram`

Update `MdFencedCodeBlock`:
```ungram
MdFencedCodeBlock =
    l_fence: '```'
    language: MdTextual?
    meta: MdTextual?
    content: MdInlineItemList
    r_fence: '```'
```

Remove `MdCodeNameList` from grammar.

### 3b. Codegen + Formatter

Run codegen. Update fenced code block formatter to handle language/meta slots. Remove `code_name_list.rs` formatter.

### 3c. Parser Update

**File:** `crates/biome_markdown_parser/src/syntax.rs`, function `parse_fenced_code_block` (~line 129)

After bumping the opening fence:
1. If next token is on same line and is not a fence char, parse it as `language: MdTextual`
2. If more tokens on same line, parse them as `meta: MdTextual`
3. Continue with content as before

### 3d. Migrate Code Block Rules

| Rule | New Query | Change |
|------|-----------|--------|
| `no_missing_language` | `Ast<MdFencedCodeBlock>` | Already migrated -- verify it uses `language()` accessor |
| `use_consistent_code_fence_marker` | `Ast<MdFencedCodeBlock>` | Check `l_fence_token()` text |
| `use_consistent_code_block_style` | Keep `Ast<MdDocument>` | Walk descendants for fenced vs indented |

**Tests:** `cargo test -p biome_markdown_parser -p biome_markdown_formatter -p biome_markdown_analyze`

---

## Phase 4: Table Cell Parsing

### 4a. Grammar Change

**File:** `xtask/codegen/markdown.ungram`

Update `MdTableRow`:
```ungram
MdTableRow =
    cells: MdTableCellList

MdTableCellList = MdTableCell*

MdTableCell =
    '|'?
    content: MdInlineItemList
```

`MD_TABLE_CELL` and `MD_TABLE_CELL_LIST` already exist in `markdown_kinds_src.rs`.

### 4b. Codegen + Formatter

Run codegen. Add formatters for `MdTableCell` and `MdTableCellList`.

### 4c. Parser Update

**File:** `crates/biome_markdown_parser/src/syntax.rs`, function `parse_table_row` (~line 1486)

Replace flat content loop with:
1. Start `MdTableCellList`
2. Loop: if current token is `|`, start `MdTableCell` with pipe, then parse inline content until next `|` or line break
3. If no leading `|`, still start cell and parse until `|` or line break
4. Complete each cell and the cell list

### 4d. Migrate Table Rules

| Rule | New Query | Change |
|------|-----------|--------|
| `no_hidden_table_cell` | `Ast<MdTable>` | Compare cell counts between rows |
| `no_mismatched_table_column_count` | `Ast<MdTable>` | Count cells per row |
| `use_consistent_table_cell_padding` | `Ast<MdTable>` | Inspect cell content spacing |
| `use_consistent_table_pipe_style` | `Ast<MdTable>` | Check leading/trailing pipe presence |
| `use_consistent_table_pipe_alignment` | `Ast<MdTable>` | Check pipe column positions |
| `use_blanks_around_tables` | `Ast<MdTable>` | Check sibling spacing |
| `no_table_indentation` | `Ast<MdTable>` | Check leading whitespace |

### 4e. Delete `table_utils.rs`

After migration, remove `table_utils.rs` and its import from `utils/mod.rs`.

**Tests:** `cargo test -p biome_markdown_parser -p biome_markdown_formatter -p biome_markdown_analyze`

---

## Phase 5: Cleanup

### 5a. Migrate `heading_utils` to AST

Rewrite `collect_heading_slugs()` in `heading_utils.rs` to walk `MdHeader` descendants instead of text-parsing. Used only by `noInvalidLinkFragments`.

### 5b. Audit utility usage

Check each utility file for unused functions after all migrations. Delete dead code.

### 5c. Update remaining-work doc

Update `kb/tasks/07-remaining-work-summary.md` to reflect final state.

### 5d. Full test suite

```bash
cargo test -p biome_markdown_parser
cargo test -p biome_markdown_formatter
cargo test -p biome_markdown_analyze
cargo test -p biome_cli
```

---

## Summary

| Phase | Rules Migrated | Grammar Changes | Utilities Reduced |
|-------|---------------|-----------------|-------------------|
| 1 | 4 rules + lexer fix | None | Delete mdx_utils.rs |
| 2 | 6 definition rules | MdLinkBlockTitle | Reduce definition_utils.rs |
| 3 | 2-3 code block rules | language/meta slots | -- |
| 4 | 7 table rules | MdTableCell/List | Delete table_utils.rs |
| 5 | heading_utils rewrite | None | Reduce heading_utils.rs |
| **Total** | **~20 rules** | **3 grammar changes** | **~1,000 lines deleted** |

**Rules remaining at `Ast<MdDocument>` after all phases (~16):** Consistency rules, cross-reference rules, and document-wide rules that legitimately need document scope.
