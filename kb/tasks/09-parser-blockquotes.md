# Parser: Blockquote Parsing

**Status:** Done
**Created:** 2026-02-09
**Effort:** Medium
**Impact:** 2 lint rules benefit, unblocks blockquote formatting

---

## Context

Blockquotes (`> text`) are not parsed — they become `MdParagraph` nodes. The grammar defines `MdQuote = AnyMdBlock` (single child). 2 lint rules use text-based `blockquote_utils.rs`.

## Grammar

Current grammar is usable as-is:
```ungram
MdQuote = AnyMdBlock
```

The `MdQuote` node wraps a single `AnyMdBlock` child. For blockquote content that spans multiple blocks, the parser can create a `MdParagraph` as the child.

**Factory validation** (`syntax_factory.rs`): Expects exactly one `AnyMdBlock` child (slot 0). If extra elements present, creates bogus node.

## Step 1: Update parser

**File**: `crates/biome_markdown_parser/src/syntax.rs`

### Detection

The `>` character is lexed as `R_ANGLE` token... actually let me check. Looking at the kinds_src, `>` maps to `R_ANGLE`. But in the lexer, `>` would go through `consume_textual()` and become `MD_TEXTUAL_LITERAL`.

Detection:
```rust
fn at_blockquote(p: &mut MarkdownParser) -> bool {
    if p.cur() != MD_TEXTUAL_LITERAL { return false; }
    p.cur_text() == ">"
}
```

### Dispatch

Add to `parse_any_block` before the paragraph fallback:
```rust
} else if at_blockquote(p) {
    parse_blockquote(p);
} else {
    parse_paragraph(p);
}
```

### Parsing

```rust
fn parse_blockquote(p: &mut MarkdownParser) {
    let m = p.start();

    // Consume the `>` marker
    p.bump_any();

    // Parse content as a paragraph (simplification — handles one line)
    // Content is everything on the current line after `>`
    let content_m = p.start();
    let list = p.start();
    while !p.at(T![EOF]) && !p.has_preceding_line_break() {
        let textual = p.start();
        p.bump_any();
        textual.complete(p, MD_TEXTUAL);
    }
    list.complete(p, MD_INLINE_ITEM_LIST);
    content_m.complete(p, MD_PARAGRAPH);

    m.complete(p, MD_QUOTE);
}
```

This is a minimal implementation — one blockquote line = one MdQuote node. Nested blockquotes and multi-line blockquotes would need a more sophisticated parser (context stack).

## Step 2: Verify lint rules

The 2 blockquote lint rules use `Ast<MdDocument>` + `blockquote_utils.rs` text scanning on the full document text. Since the CST is lossless, they should work unchanged.

## Files to modify

| File | Change |
|------|--------|
| `crates/biome_markdown_parser/src/syntax.rs` | Blockquote detection + parsing |

## Verification

```bash
cargo test -p biome_markdown_parser
cargo test -p biome_markdown_analyze  # all 200 must pass
```
