# Plan: Additional Block Parsing (Indented Code, Link Definitions, HTML Blocks)

## Context

With GFM extensions done, the next priority from the remaining work summary is "additional blocks" — indented code blocks, link reference definitions, and HTML blocks. These complete block-level parser coverage.

- **Indented code blocks**: Stub exists (`parse_indent_code_block` just calls `parse_paragraph`), detection has a bug (`> 4` should be `>= 4`). The `use_consistent_code_block_style` lint rule already detects them via text scanning.
- **Link reference definitions**: Grammar exists (`MdLinkBlock`) but parser never creates these nodes. 12+ lint rules use text-based `collect_definitions()` from `definition_utils.rs`, which works well. Adding parser support makes the AST more complete.
- **HTML blocks**: Low impact — only `no_inline_html` lint rule touches HTML, and it handles inline tags. CommonMark specifies 7 complex types. **Defer.**

## Scope

Two block types, in order:

1. **Indented code blocks** — fix bug + implement parser (~30 lines)
2. **Link reference definitions** — add parser detection + AST nodes (~60 lines)
3. ~~HTML blocks~~ — deferred (high complexity, low impact)

## Key Constraint: Indentation as Trivia

The lexer puts whitespace (spaces/tabs) into leading trivia of tokens. For `    code`, the lexer produces `MD_TEXTUAL_LITERAL "c"` with leading trivia `[Whitespace("    ")]`. The parser's `before_whitespace_count()` reads this trivia to determine indentation. This means **`MD_INDENT_CHUNK_LITERAL` tokens cannot be produced** without lexer changes.

**Solution**: Simplify `MdIndentCodeBlock` grammar from `lines: MdIndentedCodeLineList` to `content: MdInlineItemList` (flat content, same pattern as blockquotes/tables). Indentation info is preserved in trivia.

## Key Constraint: Per-Character Tokenization

Each character is a separate `MD_TEXTUAL_LITERAL` token. `[foo]: url` is 12+ tokens. The current `MdLinkBlock` grammar expects `label: MdTextual url: MdTextual title: MdTextual?` — but `MdTextual` holds exactly one token. Multi-character fields can't fit.

**Solution**: Simplify `MdLinkBlock` grammar to `content: MdInlineItemList` (flat content). Lint rules continue using text-based `collect_definitions()`.

## Implementation Steps

### Step 1: Grammar changes (`xtask/codegen/markdown.ungram`)

Change:
```ungram
MdIndentCodeBlock =
    lines: MdIndentedCodeLineList
```
To:
```ungram
MdIndentCodeBlock =
    content: MdInlineItemList
```

Change:
```ungram
MdLinkBlock = label: MdTextual
              url: MdTextual
              title: MdTextual?
```
To:
```ungram
MdLinkBlock = content: MdInlineItemList
```

Keep `MdIndentedCodeLineList`, `MdIndentedCodeLine`, `MdIndent` in grammar (they may be useful later but aren't used by the parser now).

### Step 2: Run codegen

```bash
cargo codegen grammar
```

### Step 3: Parser — Indented code blocks (`syntax.rs`)

Fix detection bug (line 341):
```rust
// Before:
p.before_whitespace_count() > 4
// After:
p.before_whitespace_count() >= 4
```

Implement `parse_indent_code_block` (replace stub at line 344-347):
```rust
pub(crate) fn parse_indent_code_block(p: &mut MarkdownParser) {
    let m = p.start();
    let content = p.start();
    let mut first = true;
    while !p.at(T![EOF]) {
        if !first && p.has_preceding_line_break() {
            // Blank lines are allowed inside indented code blocks,
            // but stop if next non-blank line has < 4 indent
            if p.has_preceding_blank_line() {
                if p.before_whitespace_count() < 4 {
                    break;
                }
            } else if p.before_whitespace_count() < 4 {
                break;
            }
        }
        first = false;
        let textual = p.start();
        p.bump_any();
        textual.complete(p, MD_TEXTUAL);
    }
    content.complete(p, MD_INLINE_ITEM_LIST);
    m.complete(p, MD_INDENT_CODE_BLOCK);
}
```

### Step 4: Parser — Link reference definitions (`syntax.rs`)

Add detection before paragraph fallback in `parse_any_block()`:
```rust
} else if at_link_definition(p) {
    if !try_parse_link_definition(p) {
        let para = parse_paragraph(p);
        maybe_wrap_setext_header(p, para);
    }
} else {
```

Detection function:
```rust
fn at_link_definition(p: &mut MarkdownParser) -> bool {
    p.cur() == MD_TEXTUAL_LITERAL
        && p.cur_text() == "["
        && p.before_whitespace_count() <= 3
}
```

Lookahead check (non-destructive, always rewinds):
```rust
fn is_link_definition_line(p: &mut MarkdownParser) -> bool {
    let mut found = false;
    let _ = try_parse(p, |p| {
        // Skip past [
        p.bump_any();
        let mut first = true;
        // Find ] on same line
        while !p.at(T![EOF]) && (first || !p.has_preceding_line_break()) {
            first = false;
            if p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == "]" {
                p.bump_any();
                // Must be followed by :
                if !p.at(T![EOF]) && !p.has_preceding_line_break()
                    && p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == ":" {
                    found = true;
                }
                break;
            }
            p.bump_any();
        }
        Err::<(), ()>(()) // Always rewind
    });
    found
}
```

Parse function:
```rust
fn try_parse_link_definition(p: &mut MarkdownParser) -> bool {
    try_parse(p, |p| {
        if !is_link_definition_line(p) {
            return Err(());
        }
        let m = p.start();
        let content = p.start();
        let mut first = true;
        while !p.at(T![EOF]) && (first || !p.has_preceding_line_break()) {
            first = false;
            let textual = p.start();
            p.bump_any();
            textual.complete(p, MD_TEXTUAL);
        }
        content.complete(p, MD_INLINE_ITEM_LIST);
        m.complete(p, MD_LINK_BLOCK);
        Ok(())
    })
    .is_ok()
}
```

### Step 5: Update formatter (if needed)

The existing text-blob formatters in `indent_code_block.rs` and `link_block.rs` already work correctly — they use `node.syntax().text_trimmed()` which captures all content regardless of internal AST structure. After the grammar change, codegen may update the factory but the formatter code itself should not need changes.

If codegen changes `MdIndentCodeBlock` from 1 slot (list) to 1 slot (inline item list), the factory will validate correctly.

### Step 6: Tests

Parser spec tests:
- `crates/biome_markdown_parser/tests/md_test_suite/ok/indent_code_block.md`:
  ```
      code line 1
      code line 2
  ```
- `crates/biome_markdown_parser/tests/md_test_suite/ok/link_definition.md`:
  ```
  [foo]: https://example.com
  [bar]: https://example.com "Title"
  ```

## Files to modify

| File | Change |
|------|--------|
| `xtask/codegen/markdown.ungram` | Simplify MdIndentCodeBlock and MdLinkBlock grammars |
| Generated files (7) | Auto-regenerated by codegen |
| `crates/biome_markdown_parser/src/syntax.rs` | Fix indent detection bug, implement both parsers |
| Test files (2) | New parser spec tests |

Files that should NOT change:
- Lexer, token source — no changes needed
- `definition_utils.rs` — lint rules keep using text-based approach
- Formatter files — text-blob pattern works unchanged

## Verification

```bash
cargo codegen grammar                          # Regenerate syntax code
cargo test -p biome_markdown_parser            # Parser tests
cargo test -p biome_markdown_formatter         # Formatter tests
cargo test -p biome_markdown_analyze           # Lint rule tests (should still pass)
cargo test -p biome_cli                        # CLI integration tests
```
