# Remaining Work Summary

**Created:** 2026-02-09
**Updated:** 2026-02-10

---

## Work Done (Committed)

### Bug Fixes
- **Fixed infinite loop in `parse_inline_list`** — `parse_block_list` looped forever because `parse_paragraph` → `parse_inline_list` consumed zero tokens when the current token had a preceding line break. Added a `first` flag so the first token is always consumed. (`2efc1e0`)
- **Fixed marker panics in `try_parse` closures** — 24 analyzer tests panicked with "Marker must either be completed or abandoned" because `Marker` objects weren't abandoned before returning `Err(())`. Added `m.abandon(p)` calls in `try_parse_inline_link`, `try_parse_inline_image`, `try_parse_inline_emphasis`, `try_parse_inline_italic`. (`2efc1e0`)
- **Updated 4 CLI snapshots** — Pre-existing mismatches from markdown lint rules added in prior commits. (`7e25c38`)
- **Fixed `has_preceding_blank_line()` false positives** — Rewrote to examine source text directly instead of relying on trivia list boundaries that broke when tokens lacked trailing whitespace trivia. (`b3e65d8`)

### Parser Expansion
- **Lists** — Ordered and unordered list parsing with proper indentation detection (grammar fix + parser) (`93fdccd`)
- **Multi-line list items** — Continuation lines with sufficient indentation consumed into same list item
- **Blockquotes** — Block quote parsing with `>` prefix handling (`93fdccd`)
- **Multi-line blockquotes** — Consecutive `>` lines consumed into a single MdQuote node
- **Setext headings** — Detection via `=`/`-` underlines with blank line guard, formatter converts to ATX
- **Inline elements** — Code spans, emphasis (`**bold**`), italic (`*italic*`), links (`[text](url)`), images (`![alt](src)`)
- **Nested inline parsing** — Content inside emphasis/italic/strikethrough/link/image recursively dispatches to inline parsers (`b3e65d8`)
- **Fenced code blocks** — Grammar fix + parser + formatter with tilde→backtick normalization (`95ae631`)
- **HTML blocks** — Multi-line HTML content starting with `<` until blank line (`b3e65d8`)
- **GFM tables** — Pipe-delimited tables with header/separator/data rows
- **GFM strikethrough** — `~~text~~` parsing with nested inline support
- **Link reference definitions** — `[label]: url` parsing
- **Indented code blocks** — 4+ space indentation detection

### Formatter Expansion (`93fdccd`, `c7739ad`)
- **All 32 auxiliary formatters properly implemented** — Zero files use `format_verbatim_node`
- **Trailing whitespace removal** — Paragraph formatter strips trailing whitespace, preserves hard breaks
- **Setext→ATX heading conversion** — Formatter converts setext headings to ATX style
- **Code fence normalization** — Tilde fences converted to backtick fences
- **Inline whitespace normalization** (`c7739ad`) — Collapses runs of spaces/tabs to a single space in paragraphs, list items, blockquotes. Preserves whitespace inside backtick-delimited code spans. 22 unit tests.

### Linter (`93fdccd`)
- **82 rules now have code fix actions** (up from 68) — Added fixes to 14 more rules across correctness, style, and suspicious categories
- **100 total lint rules** across correctness (17), style (75), suspicious (6), a11y (2)

### CLI Integration Tests
- **13 integration tests** — lint, format, check, stdin lint/format, config, disabled formatter, empty files, CRLF, multiple files

### Test Results (Latest Run)
- **746 biome_cli tests passed** (0 failed, 3 ignored)
- **281 analyzer tests passed** (200 spec + 81 unit)
- **61 formatter tests passed** (51 unit + 10 spec)
- **14 parser spec tests passed** + 7 unit tests

---

## Work Done (Uncommitted — Current Session)

### Grammar & Parser Improvements

#### Phase 1a: Ordered List Lexer Fix
- Added `consume_digit_sequence()` in lexer to produce `1.` / `1)` as single tokens instead of `"1"` + `"."` separately
- Ordered lists now parse correctly as `MdOrderListItem` nodes

#### Phase 1b-c: Code Block Rule Migration
- Migrated `no_shell_dollar_prompt` to `Ast<MdFencedCodeBlock>` — accesses language via `.language().syntax().text_trimmed()`
- Migrated `no_missing_language` to `Ast<MdFencedCodeBlock>` — uses `.language().is_empty()`

#### Phase 1d-f: MDX Rule Migration
- Migrated `use_consistent_mdx_jsx_quote_style` to `Ast<MdMdxJsxElement>`
- Migrated `no_mdx_jsx_void_children` to `Ast<MdMdxJsxElement>`
- Deleted `mdx_utils.rs` utility (no longer needed)

#### Phase 2: Structured Link Definitions
- **Grammar:** Added `MdLinkBlockTitle` node (delimiter + content + closing_delimiter)
- **Parser:** Updated `try_parse_link_definition` to produce structured title nodes
- **Rules migrated (5):** `no_undefined_references`, `no_unused_definitions`, `use_consistent_link_style`, `use_consistent_link_title_style`, `use_consistent_media_style` — all now use `collect_definitions_from_ast()`
- **`definition_utils.rs`:** Added `collect_definitions_from_ast()` that walks `MdLinkBlock` descendants with proper line-based field computation

#### Phase 3: Structured Code Block Info String
- **Grammar:** Replaced `code_list: MdCodeNameList` with `language: MdInlineItemList` and `meta: MdInlineItemList`
- **Parser:** Updated `parse_fenced_code_block` to populate language/meta slots
- **Removed:** `MdCodeNameList` node type and its formatter
- Updated `no_shell_dollar_prompt` and `no_missing_language` to use new accessors

#### Phase 4: Table Cell Parsing
- **Grammar:** Added `MdTableCell` (pipe? + content) and `MdTableCellList` nodes; changed `MdTableRow` to use `cells: MdTableCellList`
- **Kinds:** Added `PIPE` token to punct list
- **Parser:** Updated `parse_table_row` to create cell nodes separated by `|` pipes
- **Bug fix:** Fixed infinite loop when data rows start on a new line (first token has newline trivia causing inner content loop to exit immediately without consuming tokens)
- **Formatter stubs:** Added `table_cell.rs` and `table_cell_list.rs`

---

## Implementation Status

### Parser — All Major Features Complete

| Item | Status |
|------|--------|
| ATX headings | **Done** |
| Setext headings | **Done** |
| Paragraphs | **Done** |
| Fenced code blocks | **Done** (language/meta slots) |
| Indented code blocks | **Done** |
| Blockquotes (multi-line) | **Done** |
| Unordered lists (multi-line items) | **Done** |
| Ordered lists (multi-line items) | **Done** (digit sequence lexer fix) |
| Thematic breaks | **Done** |
| GFM tables | **Done** (structured cells) |
| GFM strikethrough | **Done** |
| HTML blocks | **Done** |
| Link reference definitions | **Done** (structured titles) |
| Inline code spans | **Done** |
| Inline emphasis/italic | **Done** |
| Inline links | **Done** |
| Inline images | **Done** |
| Nested inline elements | **Done** |

### Formatter — Complete

| Item | Status |
|------|--------|
| All auxiliary formatters | **Done** (zero verbatim) |
| Code fence normalization | **Done** |
| Setext→ATX conversion | **Done** |
| Trailing whitespace removal | **Done** |
| Inline whitespace normalization | **Done** |
| Table cell formatters | **Done** (stubs) |

### Linter — Complete

| Item | Status |
|------|--------|
| 100 lint rules implemented | **Done** |
| 82 rules with code fix actions | **Done** |
| ~18 rules not auto-fixable | N/A (file name rules, ambiguous transforms) |

### Integration — Complete

| Item | Status |
|------|--------|
| 13 CLI integration tests | **Done** |

---

## AST-First Migration Status

### Fully Migrated to Specific AST Nodes (23 rules)

**`Ast<MdParagraph>` (14 rules)**
- `no_trailing_hard_break_spaces`, `no_bare_urls`, `no_space_in_code`, `no_space_in_links`
- `no_space_in_emphasis`, `no_reference_like_url`, `use_proper_names`, `no_inline_html`
- `use_descriptive_link_text`, `no_missing_alt_text`, `no_shortcut_reference_link`
- `no_shortcut_reference_image`, `no_unneeded_full_reference_link`, `no_unneeded_full_reference_image`

**`Ast<MdHeader>` (2 rules)**
- `no_missing_space_closed_atx_heading`, `no_multiple_space_closed_atx_heading`

**`Ast<MdFencedCodeBlock>` (2 rules)**
- `no_shell_dollar_prompt`, `no_missing_language`

**`Ast<MdMdxJsxElement>` (2 rules)**
- `use_consistent_mdx_jsx_quote_style`, `no_mdx_jsx_void_children`

**`Ast<MdDocument>` using AST-based collection (5 rules)**
- `no_undefined_references`, `no_unused_definitions`, `use_consistent_link_style`
- `use_consistent_link_title_style`, `use_consistent_media_style`
- (These use `collect_definitions_from_ast()` which walks `MdLinkBlock` descendants)

### Shared Utilities
- `fix_utils::make_text_replacement` — range-based token replacement (used by 29 rules)
- `definition_utils::collect_definitions_from_ast` — walks MdLinkBlock AST nodes (used by 5 rules)
- `definition_utils::normalize_label` — label normalization (used by 11 rules)

### Rules Remaining at `Ast<MdDocument>` with Text-Based Scanning (~27 rules)
These use FenceTracker/line-based utilities:
- **Table rules (7):** Could migrate to `Ast<MdTable>` now that cells are structured
- **List rules (~8):** Use `list_utils.rs` text-based parsing
- **Blockquote rules (2):** Use `blockquote_utils.rs` text-based parsing
- **Cross-paragraph consistency rules:** emphasis/strong/strikethrough markers, heading style
- **Whole-document rules:** hard tabs, long lines, final newline, first-line heading

---

## Remaining Low-Priority Items

| Item | Impact | Notes |
|------|--------|-------|
| Migrate 7 table rules to `Ast<MdTable>` | Low | Grammar/parser now supports cells; rules work fine with text-based approach |
| Migrate list rules | Low | Would require further grammar changes for nested lists |
| Migrate blockquote rules | Low | Would require grammar changes for nested blockquotes |
| `heading_utils` AST migration | Low | `collect_heading_slugs()` works fine; only used by 1 rule |
| Lazy continuation in blockquotes | Low | Lines without `>` after a blockquote. Lint rules handle via text analysis. |
| Nested blockquotes | Low | `> > nested`. Lint rules parse nesting from text. |
| Nested lists | Low | Sub-items at deeper indentation. Lint rules detect via indentation analysis. |
| GFM task list checkbox AST node | Low | `- [ ] task`. Lint rules detect via `list_utils.rs` text parsing. |
| `MdInlineImageLink` slot | Low | Never populated; no use case identified. |
| Clean up legacy `collect_definitions()` / `parse_definition()` | Low | Only used in tests; superseded by `collect_definitions_from_ast()` |
| Remove unused `DocumentLines` struct | Low | In `line_utils.rs`; only used in tests |
