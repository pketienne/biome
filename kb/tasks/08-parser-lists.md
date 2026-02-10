# Parser: List Parsing

**Status:** Done
**Created:** 2026-02-09
**Effort:** High
**Impact:** 12 lint rules benefit, unblocks list formatting

---

## Context

Lists are the highest-impact parser improvement. 12 lint rules use text-based `list_utils.rs` scanning. The grammar has fundamental issues that need fixing before implementation.

## Grammar Issues (must fix first)

Current broken grammar in `xtask/codegen/markdown.ungram`:
```ungram
MdBulletListItem = MdBulletList      # Both identical — wrong
MdOrderListItem = MdBulletList       # Should have ordered bullets
MdBulletList = MdBullet*
MdBullet =
    bullet: ('-' | '*')              # Missing '+' support
    space: 'md_textual_literal'
    content: MdInlineItemList
MdOrderList = AnyCodeBlock*          # Completely wrong
```

## Step 1: Fix grammar

Replace the list definitions with:
```ungram
MdBulletListItem = MdBulletList
MdOrderListItem = MdOrderList

MdBulletList = MdBullet*

MdBullet =
    bullet: ('-' | '*' | '+')
    content: MdInlineItemList

MdOrderList = MdOrderBullet*

MdOrderBullet =
    marker: 'md_textual_literal'
    content: MdInlineItemList
```

Changes:
- Add `+` to `MdBullet` bullet alternatives (need to add `PLUS` to `markdown_kinds_src.rs` punct: `("+", "PLUS")`)
- Remove `space` field from `MdBullet` (space is trivia on next token)
- Create `MdOrderBullet` node for ordered list items (marker is the `1.` or `2)` text)
- Fix `MdOrderList = MdOrderBullet*`
- Fix `MdOrderListItem = MdOrderList`

Also add to `markdown_kinds_src.rs`:
- `("+", "PLUS")` in punct array
- `"MD_ORDER_BULLET"` in nodes array

## Step 2: Run codegen

```bash
just gen-grammar
```

## Step 3: Update lexer

**File**: `crates/biome_markdown_parser/src/lexer/mod.rs`

The issue: `-` and `*` are routed to `consume_thematic_break_literal()` which tries to parse them as thematic breaks. If not 3+ markers at line end, it falls back to `consume_textual()` producing `MD_TEXTUAL_LITERAL`.

This actually works for list parsing — single `-`, `*`, or `+` followed by space will become `MD_TEXTUAL_LITERAL` tokens. We can detect them in the parser.

For `+`: it's currently not dispatched specially. The `_` case goes through `consume_thematic_break_literal()`, but `+` doesn't have a dispatch entry. It falls through to `consume_textual()`. This is fine — `+` becomes `MD_TEXTUAL_LITERAL`.

No lexer changes needed — the existing token types are sufficient.

## Step 4: Update parser

**File**: `crates/biome_markdown_parser/src/syntax.rs`

### Detection

```rust
fn at_unordered_list(p: &mut MarkdownParser) -> bool {
    if p.cur() != MD_TEXTUAL_LITERAL { return false; }
    let text = p.cur_text();
    (text == "-" || text == "*" || text == "+")
        && !p.has_preceding_line_break()  // not needed for first check
        && p.before_whitespace_count() < 4 // not indented code
}

fn at_ordered_list(p: &mut MarkdownParser) -> bool {
    if p.cur() != MD_TEXTUAL_LITERAL { return false; }
    let text = p.cur_text();
    // Check for pattern: digits followed by . or )
    let bytes = text.as_bytes();
    if bytes.is_empty() { return false; }
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() { i += 1; }
    i > 0 && i < bytes.len() && (bytes[i] == b'.' || bytes[i] == b')')
        && p.before_whitespace_count() < 4
}
```

### Disambiguation with thematic breaks

In `parse_any_block`, check for lists BEFORE thematic breaks:
```rust
pub(crate) fn parse_any_block(p: &mut MarkdownParser) {
    if at_indent_code_block(p) {
        parse_indent_code_block(p);
    } else if at_header(p) {
        parse_header(p);
    } else if at_fenced_code_block(p) {
        parse_fenced_code_block(p);
    } else if at_unordered_list(p) {
        parse_unordered_list_item(p);
    } else if at_ordered_list(p) {
        parse_ordered_list_item(p);
    } else if at_thematic_break_block(p) {
        // ... existing thematic break handling
    } else {
        parse_paragraph(p);
    }
}
```

### Parsing

```rust
fn parse_unordered_list_item(p: &mut MarkdownParser) {
    let m = p.start();
    let list = p.start();

    // Parse one or more bullets
    while !p.at(T![EOF]) && at_bullet_start(p) {
        parse_bullet(p);
    }

    list.complete(p, MD_BULLET_LIST);
    m.complete(p, MD_BULLET_LIST_ITEM);
}

fn at_bullet_start(p: &mut MarkdownParser) -> bool {
    if p.cur() != MD_TEXTUAL_LITERAL { return false; }
    let text = p.cur_text();
    text == "-" || text == "*" || text == "+"
}

fn parse_bullet(p: &mut MarkdownParser) {
    let m = p.start();

    // Slot 0: bullet marker — remap to MINUS, STAR, or PLUS
    let marker = p.cur_text().to_string();
    match marker.as_str() {
        "-" => p.bump_remap(MINUS),
        "*" => p.bump_remap(STAR),
        "+" => p.bump_remap(PLUS),
        _ => p.bump_any(),
    }

    // Slot 1: content — everything on the line after the marker
    let content = p.start();
    while !p.at(T![EOF]) && !p.has_preceding_line_break() {
        let textual = p.start();
        p.bump_any();
        textual.complete(p, MD_TEXTUAL);
    }
    // Continuation lines (indented, not a new list marker or blank line)
    // For now, only parse single-line items
    content.complete(p, MD_INLINE_ITEM_LIST);

    m.complete(p, MD_BULLET);
}
```

Similar for ordered lists with `parse_ordered_list_item` / `parse_order_bullet`.

## Step 5: Verify lint rules still work

The 12 list-related lint rules use `Ast<MdDocument>` + `list_utils.rs` text scanning. Since the CST is lossless, `document.syntax().text_with_trivia()` returns identical text. Rules should work unchanged.

**Critical**: Run `cargo test -p biome_markdown_analyze` — all 200 tests must pass.

## Files to modify

| File | Change |
|------|--------|
| `xtask/codegen/markdown.ungram` | Fix list grammar |
| `xtask/codegen/src/markdown_kinds_src.rs` | Add PLUS token, MD_ORDER_BULLET node |
| Generated files | Auto-regenerated |
| `crates/biome_markdown_parser/src/syntax.rs` | List detection + parsing |

## Verification

```bash
just gen-grammar
cargo build -p biome_markdown_parser -p biome_markdown_analyze
cargo test -p biome_markdown_parser
cargo test -p biome_markdown_analyze  # all 200 must pass
```
