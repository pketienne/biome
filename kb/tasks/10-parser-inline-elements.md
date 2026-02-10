# Parser: Inline Element Parsing

**Status:** Planned
**Created:** 2026-02-09
**Effort:** Very High
**Impact:** 20+ lint rules benefit, unblocks inline formatting

---

## Context

Inline elements (code spans, emphasis, links, images) are not parsed — everything inside paragraphs is flattened to `MdTextual` tokens. Full AST definitions exist for all inline types. This is the most complex parser change, requiring CommonMark-spec delimiter matching.

## Grammar (already defined, no changes needed)

```ungram
MdInlineCode = l_tick: '`' content: MdInlineItemList r_tick: '`'
MdInlineEmphasis = l_fence: ('**' | '__') content: MdInlineItemList r_fence: ('**' | '__')
MdInlineItalic = l_fence: ('*' | '_') content: MdInlineItemList r_fence: ('*' | '_')
MdInlineLink = '[' text: MdInlineItemList ']' '(' source: MdInlineItemList ')'
MdInlineImage = '[' '!' alt: MdInlineImageAlt source: MdInlineImageSource ']' link: MdInlineImageLink?
```

## Token kinds available

- `BACKTICK` (`` ` ``), `TRIPLE_BACKTICK` (`` ``` ``)
- `STAR` (`*`), `DOUBLE_STAR` (`**`)
- `UNDERSCORE` (`_`), `DOUBLE_UNDERSCORE` (`__`)
- `L_BRACK` (`[`), `R_BRACK` (`]`)
- `L_PAREN` (`(`), `R_PAREN` (`)`)
- `BANG` (`!`)

## Implementation approach

### Phase 1: Code spans (simplest)

Code spans have clear boundaries — matching backtick delimiters. No nesting.

In `parse_paragraph` or a new `parse_inline_content`:
1. When encountering `` ` `` token, look ahead for matching `` ` ``
2. If found: create `MdInlineCode` node with content between ticks
3. If not: treat as regular `MdTextual`

### Phase 2: Links

Links have clear boundaries — `[text](url)` pattern.

1. When encountering `[`, look ahead for `](` sequence
2. If found: create `MdInlineLink` with text and source lists
3. Handle nested brackets for text content

### Phase 3: Images

Images follow links — `![alt](url)` pattern.

1. When encountering `[!`, look ahead for `](` sequence
2. Create `MdInlineImage` with alt and source sub-nodes

### Phase 4: Emphasis (most complex)

Emphasis requires CommonMark delimiter matching algorithm:
- `*text*` = italic, `**text**` = bold
- `_text_` = italic, `__text__` = bold
- Delimiter runs can be opening, closing, or both
- Left-flanking and right-flanking rules

This should be implemented last due to complexity.

## Lexer changes needed

Currently `*`, `_`, `[`, `]`, `(`, `)`, `!` are all consumed as `MD_TEXTUAL_LITERAL` by the lexer. They need to be recognized as their proper token kinds.

In `consume_token`, add dispatch for these characters:
```rust
b'[' => { self.advance(1); L_BRACK }
b']' => { self.advance(1); R_BRACK }
b'(' => { self.advance(1); L_PAREN }
b')' => { self.advance(1); R_PAREN }
b'!' => { self.advance(1); BANG }
```

For `*` and `_`: these already go through `consume_thematic_break_literal()`. Single `*`/`_` (not 3+ at line end) already fall back to `MD_TEXTUAL_LITERAL`. We'd need to detect `**`/`__` vs `*`/`_` and return the appropriate token kinds.

## Risk

This is the highest-risk parser change. Every inline element change affects how paragraphs are parsed, which affects ALL lint rules that scan paragraph text. Must be done incrementally with full test suite verification after each phase.

## Files to modify

| File | Change |
|------|--------|
| `crates/biome_markdown_parser/src/lexer/mod.rs` | Recognize `[`, `]`, `(`, `)`, `!` as tokens |
| `crates/biome_markdown_parser/src/syntax.rs` | Inline parsing within paragraphs |

## Verification

After EACH phase:
```bash
cargo test -p biome_markdown_parser
cargo test -p biome_markdown_analyze  # all 200 must pass
cargo test -p biome_markdown_formatter
```
