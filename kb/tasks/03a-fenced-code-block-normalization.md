# Expand Formatter: Fenced Code Block Normalization

**Status:** Done (unblocked by 03b)
**Created:** 2026-02-09
**Effort:** Medium → High (grammar change + codegen needed)
**Impact:** First parser improvement + real fenced code block formatting

---

## Context

Plan 03 proposes expanding the markdown formatter beyond headings and thematic breaks. Phase A is fenced code block normalization (tilde fences → backtick fences). However, the parser currently creates `MdParagraph` nodes for fenced code blocks — not `MdFencedCodeBlock` nodes.

## What was accomplished

### Lexer improvement (KEPT)

**File**: `crates/biome_markdown_parser/src/lexer/mod.rs`

Added `consume_fence_or_textual()` that combines 3+ consecutive backticks or tildes into a single `MD_TEXTUAL_LITERAL` token. This follows the same pattern as `consume_thematic_break_literal()` for `---`/`***`/`___`. Updated `consume_token` dispatch for `b'`'` and `b'~'`.

This change is safe and doesn't affect any existing behavior because fenced code blocks are parsed as paragraphs regardless. The combined tokens still work with `FenceTracker` text scanning.

### Parser change (REVERTED)

Attempted to create proper `MdFencedCodeBlock` nodes. Discovered a fundamental blocker.

## Discovery: Grammar incompatibility

The `MdFencedCodeBlock` AST type has these slots:
- `l_fence_token`: expects `T!["```"]` (TRIPLE_BACKTICK kind)
- `code_list`: `MdCodeNameList`
- `l_hard_line`: `MdHardLine`
- `content`: `MdTextual`
- `r_hard_line`: `MdHardLine`
- `r_fence_token`: expects `T!["```"]`

**Problem 1**: The `MarkdownSyntaxFactory` (generated from the grammar) validates that fence tokens have kind `TRIPLE_BACKTICK`. Fixed with `p.bump_remap(TRIPLE_BACKTICK)`.

**Problem 2 (BLOCKER)**: The `content: MdTextual` slot expects a single node, and the `MdTextual` factory validates that it has exactly 1 child of kind `MD_TEXTUAL_LITERAL`. But fenced code block content spans multiple lines/tokens. The factory rejects multi-token content and creates `MD_BOGUS` nodes.

**Root cause**: In `xtask/codegen/markdown.ungram`:
```
MdTextual = value: 'md_textual_literal'
```
This grammar rule means `MdTextual` can only hold a single token. Fenced code block content needs a multi-token container.

## What's needed to unblock

1. **Update the grammar** (`xtask/codegen/markdown.ungram`):
   - Either change `content: MdTextual` to a multi-token type (e.g., `content: MdInlineItemList`)
   - Or add a new node type like `MdFencedCodeContent` that can hold multiple tokens
   - Consider: `MdFencedCodeContent = items: MdTextual*` or similar

2. **Regenerate syntax/factory code**:
   - Run `just gen-syntax` (or the equivalent codegen command)
   - This regenerates `biome_markdown_syntax/src/generated/` and `biome_markdown_factory/src/generated/`

3. **Update parser** (`syntax.rs`):
   - `at_fenced_code_block` — detect `MD_TEXTUAL_LITERAL` tokens with 3+ backticks/tildes
   - `parse_fenced_code_block` — create `MdFencedCodeBlock` nodes matching the updated grammar
   - Need `bump_remap(TRIPLE_BACKTICK)` for fence tokens

4. **Update formatter** (`fenced_code_block.rs`):
   - Replace `format_verbatim_node` with real normalization logic
   - Normalize tildes to backticks, preserve length

5. **Update/migrate lint rules**:
   - The 4 fence-related rules (`noMissingLanguage`, `noShellDollarPrompt`, `useBlanksAroundCodeFences`, `useConsistentCodeFenceMarker`) currently use `Ast<MdDocument>` + `FenceTracker` text scanning
   - Can be migrated to `Ast<MdFencedCodeBlock>` once the parser creates proper nodes
   - This would be more robust and eliminate the text-scanning approach

## Current state

- Lexer: combined fence tokens (KEPT, working)
- Parser: `at_fenced_code_block` returns `false`, fences parsed as paragraphs (REVERTED)
- Formatter: `format_verbatim_node` passthrough (REVERTED)
- Lint rules: `Ast<MdDocument>` + `FenceTracker` (REVERTED)
- All tests pass: parser (1), formatter (5), analyzer (200)

## Verification

```bash
cargo test -p biome_markdown_parser   # 1 passed
cargo test -p biome_markdown_formatter # 5 passed
cargo test -p biome_markdown_analyze   # 200 passed
```
