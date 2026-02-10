# Remaining Parser Work

**Created:** 2026-02-10
**Status:** Implementation plan

---

## Current State (post b3e65d84)

### Fully Complete
- **Parser**: 12 block types, 8 inline types, nested inline parsing, HTML blocks
- **Formatter**: 32 formatters, all produce real output (zero verbatim)
- **Analyzer**: 100 lint rules (82% auto-fixable), all use text-based analysis
- **CLI tests**: 13 tests (lint, format, check, stdin, empty, CRLF, multi-file)
- **Parser tests**: 10 spec suites (110KB snapshots)
- **Formatter tests**: 10+ specs, 22 unit tests
- **Analyzer tests**: 200 spec snapshots

### Key Architectural Finding
All 100 lint rules use **text-based line-by-line analysis** via `list_utils.rs` (357 lines)
and `blockquote_utils.rs` (216 lines). They do NOT traverse AST nodes directly. This means
parser improvements to blockquotes/lists primarily benefit:
1. AST correctness for downstream tooling
2. The formatter (which walks the AST)
3. Future rules that might use AST traversal

---

## Remaining Parser Gaps

### 1. Multi-line Blockquotes
**Current**: `parse_blockquote` only consumes the `>` line and its content as a single line.
Multi-line blockquotes (consecutive `>` lines) produce separate `MdQuote` nodes.

**Goal**: Consume consecutive `>` lines into a single `MdQuote` node. The inner `MdParagraph`
spans multiple lines. Stop at blank lines or lines without `>`.

**Approach**: After first `>` line, loop checking if next line starts with `>`. If so, consume
the `>` and content into the same paragraph. No grammar change needed.

### 2. Multi-line List Items
**Current**: `parse_bullet`/`parse_order_bullet` only consume one line per item.
Indented continuation lines become separate paragraphs.

**Goal**: Consume indented continuation lines (indent >= 2) as part of the same list item's
content. Stop at blank lines, new list markers, or unindented lines.

**Approach**: Replace `parse_inline_list(p)` in bullet parsers with a multi-line variant
that continues across line breaks when indentation is sufficient. No grammar change needed.

### 3. Parser Spec Tests
**Missing**: No spec tests for blockquotes or lists. Add tests covering:
- Basic blockquotes and multi-line blockquotes
- Basic unordered and ordered lists
- Multi-item lists
- Mixed content (lists + blockquotes + paragraphs)

---

## Implementation Steps

### Step 1: Multi-line blockquote parsing
Extend `parse_blockquote` to consume consecutive `>` lines.

### Step 2: Multi-line list item parsing
Add `parse_multiline_inline_list` helper. Use in `parse_bullet` and `parse_order_bullet`.

### Step 3: Add parser spec tests
Create test files for blockquotes, lists, and mixed content.

### Step 4: Accept snapshots and verify all tests pass

### Step 5: Update remaining work document

---

## Files to Modify

| File | Change |
|------|--------|
| `crates/biome_markdown_parser/src/syntax.rs` | Multi-line blockquote + list parsing |
| `crates/biome_markdown_parser/tests/md_test_suite/ok/blockquote.md` | New test |
| `crates/biome_markdown_parser/tests/md_test_suite/ok/list_unordered.md` | New test |
| `crates/biome_markdown_parser/tests/md_test_suite/ok/list_ordered.md` | New test |
| `crates/biome_markdown_parser/tests/md_test_suite/ok/mixed_content.md` | New test |
| `kb/tasks/07-remaining-work-summary.md` | Update status |

## Files NOT Modified
- Grammar (`markdown.ungram`) — no changes needed
- Formatter — text-blob pattern handles multi-line content automatically
- Analyzer/lint rules — text-based analysis works regardless of AST structure
