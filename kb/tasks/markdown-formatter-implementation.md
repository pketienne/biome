# Markdown Formatter: Real Formatting Logic

Created: 2026-02-09
Status: **Completed**

---

## Context

The markdown formatter was a no-op passthrough — all node formatters used `format_verbatim_node()` except `FormatMdDocument` which added a final newline. This work implements real formatting logic for ATX headings and thematic breaks, plus a proper spec test infrastructure.

## What the parser gives us

- **MdHeader**: `before: MdHashList` (hash tokens), `content: Option<MdParagraph>`, `after: MdHashList` (trailing hashes — always empty, parser includes trailing hashes in content text)
- **MdThematicBreakBlock**: `value_token: MD_THEMATIC_BREAK_LITERAL` (e.g. `"***"`, `"- - -"`, `"___"`)
- **MdParagraph**: `list: MdInlineItemList`, `hard_line: MdHardLine` — inline content is flat text
- **Blank lines**: Stored as multiple `NEWLINE` trivia on the leading trivia of the next token — NOT explicit AST nodes

## Changes Made

### 1. Block-level formatting with blank line preservation

**File**: `src/md/lists/block_list.rs`

Changed from `f.join().entries(...)` to `f.join_nodes_with_hardline()` which uses `get_lines_before(node)` to detect blank lines in leading trivia and emit `empty_line()` (preserving blank lines) or `hard_line_break()` between blocks.

**File**: `src/md/auxiliary/document.rs`

Removed unused `biome_rowan::AstNode` import. Already delegated to `node.value().format()`.

### 2. Format MdThematicBreakBlock

**File**: `src/md/auxiliary/thematic_break_block.rs`

Normalizes any thematic break style (`***`, `___`, `- - -`) to `---` using `format_replaced(&token, &text("---", token.text_range().start()))`.

### 3. Format MdHeader (ATX headings)

**File**: `src/md/auxiliary/header.rs`

Formats `before` hash list + exactly one space + content (verbatim). The `after` hash list (trailing hashes) is intentionally skipped/not emitted.

**Parser limitation**: The parser does NOT separate trailing hashes from content — `### Heading ###` puts the trailing `###` inside the content paragraph text. Trailing hash removal would require parser changes.

**File**: `src/md/auxiliary/hash.rs`

Formats hash tokens via `node.hash_token().format()` instead of verbatim.

### 4. Verbatim range fix

**File**: `src/verbatim.rs`

Fixed `format_markdown_verbatim_node` to use `text_trimmed_range().len()` instead of `text_range_with_trivia().len()`. The `fmt` method only emits `text_trimmed()`, so the stored verbatim length must match. The old value caused "byte index N is out of bounds" panics during idempotency checks when multi-paragraph content was formatted.

### 5. Spec test infrastructure

Created following the JSON formatter pattern:
- `tests/language.rs` — `MarkdownTestFormatLanguage` implementing `TestFormatLanguage`
- `tests/spec_test.rs` — test runner using `SpecSnapshot`/`SpecTestFile`
- `tests/spec_tests.rs` — auto-discover fixtures via `tests_macros::gen_tests!`

### 6. Test fixtures

| Fixture | Tests |
|---------|-------|
| `headings/atx_basic.md` | Space normalization: `#Hello` → `# Hello`, `##  Title` → `## Title` |
| `thematic_break/normalize.md` | `***`, `___`, `- - -` all → `---` |
| `document/final_newline.md` | Missing newline gets added |
| `document/preserve_content.md` | Multi-paragraph content stays verbatim |
| `mixed/headings_and_content.md` | Combined heading + thematic break normalization |

### 7. Inline tests

Added 5 new inline tests in `src/lib.rs`:
- `normalizes_thematic_break_stars` — `***` → `---`
- `normalizes_thematic_break_underscores` — `___` → `---`
- `normalizes_heading_space` — `#Hello` → `# Hello`
- `normalizes_heading_extra_spaces` — `##  Title` → `## Title`
- `trailing_hashes_preserved_by_parser` — documents parser limitation

## Files modified (relative to `crates/biome_markdown_formatter/`)

| File | Change |
|------|--------|
| `src/md/auxiliary/document.rs` | Remove unused import |
| `src/md/auxiliary/header.rs` | Real heading formatting |
| `src/md/auxiliary/hash.rs` | Format hash tokens properly |
| `src/md/auxiliary/thematic_break_block.rs` | Normalize to `---` |
| `src/md/lists/block_list.rs` | `join_nodes_with_hardline` for blank line preservation |
| `src/verbatim.rs` | Fix verbatim length to use trimmed range |
| `src/lib.rs` | Add 5 inline tests |
| `Cargo.toml` | Add test dependencies |
| `tests/language.rs` | **New** — TestFormatLanguage impl |
| `tests/spec_test.rs` | **New** — spec test runner |
| `tests/spec_tests.rs` | **New** — test module |
| `tests/specs/md/**/*.md` | **New** — 5 test fixtures |
| `tests/specs/md/**/*.snap` | **New** — 5 snapshot files |

## Test results

- 11 inline tests: all pass
- 5 spec tests: all pass (with snapshot acceptance)

## Key utilities used

- `format_replaced(&token, &text("---", pos))` from `crate::trivia` — replaces token text preserving trivia
- `format_verbatim_node(node.syntax())` from `crate::verbatim` — keep node as-is
- `f.join_nodes_with_hardline()` — join blocks with newlines, preserving blank lines via trivia inspection
- `space()`, `text()` from `biome_formatter::prelude`
