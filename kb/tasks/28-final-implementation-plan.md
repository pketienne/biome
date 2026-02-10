# Implementation Plan: Final Incomplete Work

**Created:** 2026-02-10
**Source:** Comprehensive audit of all `kb/tasks/` files against actual codebase

---

## Audit Summary

A full codebase verification was performed for every item in all task files (01–27).
The overwhelming majority of planned work has been completed. Below is the
definitive status.

### Already Completed (verified in codebase)

| Source | Item | Evidence |
|--------|------|----------|
| 26 #1 | 7 table rules → `Ast<MdTable>` | All 7 rules use `Ast<MdTable>`; `table_utils.rs` deleted |
| 26 #2 | Definition functions gated | `#[cfg(test)]` on `collect_definitions`, `parse_definition`, `parse_definition_title` |
| 26 #3 | `DocumentLines` struct removed | Not in `line_utils.rs` |
| 26 #4 | Helpers privatized | `parse_list_item`, `heading_slug`, `extract_atx_heading_text` are `pub(crate)` |
| 26 #5 | Heading utils → AST | `collect_heading_slugs()` walks `MdHeader`/`MdSetextHeader` descendants |
| 26 #8 | Nested lists | Recursive `parse_bullet()`/`parse_order_bullet()` with test snapshots |
| 26 #9 | Nested blockquotes | Recursive `parse_blockquote()` |
| 26 #10 | Lazy continuation | `at_continuation_stop()` logic in blockquote parser |
| 26 #11 | Checkboxes | `try_parse_checkbox()` → `MdCheckbox` AST node |
| 26 #12 | Table formatter | Column-aligned formatting with width computation |
| 26 #14 | Dead code audit | All utility modules actively used |
| 25 1a | Ordered list lexer | `consume_digit_sequence()` implemented in lexer |
| 25 1b | `noShellDollarPrompt` → `Ast<MdFencedCodeBlock>` | Migrated |
| 25 1c | `useBlanksAroundCodeFences` → `Ast<MdFencedCodeBlock>` | Migrated |
| 25 1d–e | MDX JSX rules | All use `Ast<MdMdxJsxElement>` or `Ast<MdHtmlBlock>` |
| 25 1f | Delete `mdx_utils.rs` | File does not exist |
| 25 2a | Structured link definitions grammar | `MdLinkBlockTitle` exists; `MdLinkBlock` has label/url/title slots |
| 25 2d | Definition rule migrations | `no_definition_spacing_issues` and `use_lowercase_definition_labels` use `Ast<MdLinkBlock>` |
| 25 3a | Code block language/meta slots | Grammar has separate `language` and `meta` in `MdFencedCodeBlock` |
| 25 4 | Table cell parsing | `MdTableCell`/`MdTableCellList` fully implemented |
| 25 5a | Heading utils → AST | Done (same as 26 #5) |
| 21 | 9 directive/MDX rules | All migrated to `Ast<MdDirective>`, `Ast<MdMdxJsxElement>`, `Ast<MdHtmlBlock>` |
| 20 | Formatter warning cleanup | Dead formatter functions deleted |

### Correctly Deferred (text-based approach works, migration is complex)

| Source | Item | Reason |
|--------|------|--------|
| 26 #6 | List rules → AST (12 rules) | Need document-level cross-item consistency; `list_utils.rs` works correctly |
| 26 #7 | Blockquote rules → AST (2 rules) | Need document-level context; `blockquote_utils.rs` works correctly |
| 25 2–3 | Remaining definition/code block rule migrations | Rules at `Ast<MdDocument>` legitimately need document scope for duplicate/cross-ref detection |

### Remaining FenceTracker usage (17 rules)

17 rules use `Ast<MdDocument>` + `FenceTracker` for text-based line scanning. All are legitimate: consistency rules need first-seen tracking across all elements, cross-reference rules need document scope, and document-wide scanning rules (hard tabs, long lines, blank lines) inherently require full-document access. FenceTracker correctly skips fenced code blocks during text scanning.

---

## Completed Work

### Item A: Remove `MdInlineImageLink` from grammar — COMPLETED

**Problem:** The `MdInlineImage` grammar had an optional `link: MdInlineImageLink?` slot intended for `[![alt](img)](link)` patterns. In practice, linked images are naturally represented by nesting `MdInlineImage` inside `MdInlineLink`, so this slot was never used. The parser never populated it, no analyzer rules referenced it, and the formatter stub was dead code.

**Changes made:**

1. `xtask/codegen/markdown.ungram` — Removed `MdInlineImageLink` definition and `link` slot from `MdInlineImage`
2. `xtask/codegen/src/markdown_kinds_src.rs` — Removed `MD_INLINE_IMAGE_LINK` from node kinds
3. Ran `cargo codegen grammar` and `cargo codegen formatter` — regenerated 8 files
4. `crates/biome_markdown_parser/src/syntax.rs` — Removed slot 3 comment
5. Formatter stub `inline_image_link.rs` auto-deleted by codegen
6. Restored `FormatAnyCodeBlock`/`FormatAnyContainerBlock` in `block.rs` (codegen bug drops them)
7. Accepted 1 snapshot update (`inline_links.md.snap` — removed `link: missing (optional)` line)

**Test results:** All tests pass:
- Parser: 26 passed, 1 ignored
- Formatter: 61 passed
- Analyzer: 263 passed, 1 ignored

---

## Summary

All work from `kb/tasks/` files 01–27 is now either completed or correctly deferred. No further actionable items remain.
