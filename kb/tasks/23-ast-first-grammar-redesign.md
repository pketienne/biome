# Plan: Full AST-First Grammar Redesign

**Created:** 2026-02-10

---

## Goal

Complete the AST-first migration by redesigning the grammar so that structured nodes replace text-based shadow parser utilities. This enables migrating the remaining document-level rules to specific AST node queries.

This plan builds on top of the parser improvements (plan 22) and rule migrations (plan 21).

## Grammar Changes

### 1. Structured Link Definitions

**Current:** `MdLinkBlock = content: MdInlineItemList` (flat text)

**New:**
```ungram
MdLinkBlock =
    '['
    label: MdTextual
    ']'
    ':'
    url: MdTextual
    title: MdLinkBlockTitle?

MdLinkBlockTitle =
    delimiter: 'md_textual_literal'
    content: MdTextual
    closing_delimiter: 'md_textual_literal'
```

**Parser changes:** In `try_parse_link_definition()`, instead of consuming the whole line as flat content, parse `[label]:`, then URL, then optional title.

**New kinds needed:** `MD_LINK_BLOCK_TITLE`

**Rules enabled:**
- `no_duplicate_definitions` → `Ast<MdLinkBlock>`, compare `.label()` text
- `no_duplicate_defined_urls` → `Ast<MdLinkBlock>`, compare `.url()` text
- `no_definition_spacing_issues` → `Ast<MdLinkBlock>`, inspect spacing around `:` and URL
- `use_lowercase_definition_labels` → `Ast<MdLinkBlock>`, check `.label()` case
- `use_sorted_definitions` → `Ast<MdDocument>` (still needs all definitions, but uses structured access)
- `use_definitions_at_end` → `Ast<MdDocument>` (needs position info)

**Utility reduced:** `definition_utils.rs` — parsing functions replaced by AST accessors. Label normalization function retained.

### 2. Structured Code Block Info String

**Current:** `MdFencedCodeBlock` has `code_list: MdCodeNameList` for info string.

**New:**
```ungram
MdFencedCodeBlock =
    l_fence: 'md_textual_literal'
    language: MdTextual?
    meta: MdTextual?
    content: MdInlineItemList
    r_fence: 'md_textual_literal'
```

**Parser changes:** After consuming the opening fence, parse the first word as `language`, remaining text as `meta`.

**Rules enabled:**
- `no_missing_language` → `Ast<MdFencedCodeBlock>`, check `.language().is_none()`
- `no_shell_dollar_prompt` → `Ast<MdFencedCodeBlock>`, check `.language()` text + content
- `use_consistent_code_fence_marker` → `Ast<MdFencedCodeBlock>`, inspect `.l_fence_token()`
- `use_blanks_around_code_fences` → `Ast<MdFencedCodeBlock>`, check sibling spacing
- `use_consistent_code_block_style` → `Ast<MdDocument>` (needs to see both fenced and indented blocks)

**Utility reduced:** `fence_utils.rs` — `FenceTracker` no longer needed by these rules.

### 3. Table Row → Table Cells

(Covered in plan 22, Step 4)

**Rules enabled:**
- `no_hidden_table_cell` → `Ast<MdTable>`, compare cell counts per row
- `no_mismatched_table_column_count` → `Ast<MdTable>`, cell count consistency
- `use_consistent_table_cell_padding` → `Ast<MdTableCell>`, inspect padding
- `use_consistent_table_pipe_style` → `Ast<MdTableRow>`, inspect leading/trailing cells
- `no_table_indentation` → `Ast<MdTable>`, check leading whitespace

**Utility reduced:** `table_utils.rs` — cell splitting replaced by AST traversal.

## Implementation Steps

### Phase 1: Link Definition Structure
1. Add `MD_LINK_BLOCK_TITLE` kind
2. Update grammar for `MdLinkBlock`
3. Run codegen
4. Update parser: structured label/url/title parsing
5. Add formatter for `MdLinkBlockTitle`
6. Accept snapshots
7. Migrate definition rules to use structured accessors

### Phase 2: Code Block Info String
1. Update grammar for `MdFencedCodeBlock` (language/meta slots)
2. Remove `MdCodeNameList` from grammar
3. Run codegen
4. Update parser: parse language and meta separately
5. Update formatter to handle new slots
6. Accept snapshots
7. Migrate code block rules

### Phase 3: Rule Migration
1. Migrate definition rules (6 rules)
2. Migrate code block rules (5 rules)
3. Migrate table rules (5 rules, after plan 22 table cells)
4. Run full test suite

### Phase 4: Cleanup
1. Remove unused functions from `definition_utils.rs`
2. Remove unused functions from `table_utils.rs`
3. Evaluate `fence_utils.rs` for removal

## Files Changed

| File | Change |
|------|--------|
| `xtask/codegen/markdown.ungram` | Link definition, code block info string structure |
| `xtask/codegen/src/markdown_kinds_src.rs` | New kinds |
| `crates/biome_markdown_parser/src/syntax.rs` | Structured parsing for definitions and code blocks |
| `crates/biome_markdown_formatter/src/md/auxiliary/link_block.rs` | Updated formatter |
| `crates/biome_markdown_formatter/src/md/auxiliary/fenced_code_block.rs` | Updated formatter |
| ~16 rule files | Migrate to specific AST queries |
| Utility files | Remove unused functions |

## Risk: Medium-High

Grammar changes cascade through codegen. Each phase should be verified independently before proceeding to the next.
