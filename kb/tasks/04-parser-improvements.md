# Parser Improvements

**Status:** Planned
**Created:** 2026-02-09
**Effort:** High
**Impact:** Unblocks real formatting + more accurate lint rules

---

## Context

The markdown parser (`crates/biome_markdown_parser/`) produces a minimal AST. Most markdown constructs are flattened to `MdParagraph` containing `MdTextual` tokens. The 100 lint rules work around this with text-based scanning utilities (`fence_utils`, `inline_utils`, `table_utils`, `list_utils`, etc.), but proper AST nodes would enable:

1. More accurate lint rules (no regex false positives)
2. Real formatter support for lists, blockquotes, code blocks
3. AST-based code actions that manipulate structure

## Current Parser Capabilities

### What IS parsed to proper AST nodes
- `MdDocument` — root node
- `MdHeader` — ATX headings (`# Heading`), with `MdHashList` (before/after) and content `MdParagraph`
- `MdParagraph` — text blocks, with `MdInlineItemList` of `MdTextual` tokens
- `MdThematicBreakBlock` — horizontal rules (`---`, `***`, `___`)
- `MdFencedCodeBlock` — fenced code blocks (``` ``` ```), with fence tokens, language identifier, and content

### What is NOT parsed (flattened to MdParagraph/MdTextual)
- **Lists** — `MdBulletListItem`, `MdOrderListItem`, `MdBullet` nodes exist in syntax but parser doesn't produce them
- **Blockquotes** — `MdQuote` node exists but parser doesn't produce it
- **Inline formatting** — `MdInlineCode`, `MdInlineEmphasis`, `MdInlineItalic`, `MdInlineLink`, `MdInlineImage` nodes all exist in syntax but parser doesn't populate them
- **Setext headings** — `MdSetextHeader` exists but partially implemented
- **HTML blocks** — `MdHtmlBlock` exists but not parsed
- **Link reference definitions** — `MdLinkBlock` exists but not parsed
- **Indented code blocks** — `MdIndentCodeBlock` exists but not parsed
- **Tables** — No AST node types defined at all

### Parser architecture

- **Lexer**: `crates/biome_markdown_parser/src/lexer/mod.rs` — produces tokens (HASH, MD_TEXTUAL_LITERAL, STAR, etc.)
- **Parser**: `crates/biome_markdown_parser/src/syntax.rs` — main parsing logic
  - `parse_document()` → `parse_block_list()` → dispatches based on first token
  - Recognizes: ATX headers (by HASH), thematic breaks, fenced code blocks (by backtick/tilde)
  - Everything else → `parse_paragraph()`
- **Token source**: `crates/biome_markdown_parser/src/token_source.rs` — tracks whitespace for indentation
- **Factory**: `crates/biome_markdown_factory/` — generated node construction helpers
- **Syntax kinds**: `crates/biome_markdown_syntax/src/generated/kind.rs` — all token/node kind enums

## Proposed Improvements (ordered by impact)

### Phase 1: Lists (highest impact — 12 lint rules benefit)

Parse unordered and ordered lists:
- Detect list markers (`-`, `*`, `+` for unordered; `1.`, `2)` for ordered)
- Create `MdBulletListItem` / `MdOrderListItem` nodes
- Handle indentation for nested lists
- Handle list item continuation lines

**Lint rules that benefit**: `useConsistentUnorderedListMarker`, `useConsistentOrderedListMarker`, `useConsistentListItemIndent`, `useConsistentListIndent`, `noListItemBulletIndent`, `useConsistentListItemContentIndent`, `useConsistentListItemSpacing`, `useConsistentOrderedListMarkerValue`, `noCheckboxCharacterStyleMismatch`, `noCheckboxContentIndent`, `useBlanksAroundLists`, `useConsistentUnorderedListIndent`

### Phase 2: Blockquotes (2 lint rules benefit)

Parse blockquotes:
- Detect `> ` prefix
- Create `MdQuote` nodes
- Handle nested blockquotes
- Handle lazy continuation lines

**Lint rules that benefit**: `noBlockquoteBrokenContinuation`, `useConsistentBlockquoteIndent`

### Phase 3: Inline elements (20+ lint rules benefit)

Parse inline formatting:
- Code spans (`` ` ``)
- Emphasis (`*text*`, `_text_`)
- Strong (`**text**`, `__text__`)
- Links (`[text](url)`, `[text][ref]`)
- Images (`![alt](url)`)

This is the most complex change as it requires proper delimiter matching per the CommonMark spec.

**Lint rules that benefit**: Most link, emphasis, code span, and image rules

### Phase 4: GFM extensions

- Tables (pipe tables with headers, alignment)
- Task list items (`- [ ]`, `- [x]`)
- Strikethrough (`~~text~~`)
- Autolinks

### Phase 5: Additional block elements

- Setext headings (properly)
- HTML blocks
- Link reference definitions
- Indented code blocks (4-space indent)

## Key reference documents

- [CommonMark spec](https://spec.commonmark.org/) — the parsing algorithm
- `crates/biome_parser/CONTRIBUTING.md` — how biome parsers work
- `crates/biome_markdown_syntax/src/generated/nodes.rs` — defined AST nodes (many unused)
- `crates/biome_markdown_syntax/src/generated/kind.rs` — all syntax kinds

## Verification

```bash
# Parser tests
cargo test -p biome_markdown_parser

# Ensure lint rules still work after parser changes
cargo test -p biome_markdown_analyze

# Ensure formatter still works
cargo test -p biome_markdown_formatter
```

## Risk

This is the highest-risk and highest-effort item. Parser changes can break lint rules that rely on the current text-based scanning approach. Each parser improvement should be accompanied by a full test suite run of the analyzer to catch regressions.
