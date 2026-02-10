# Plan: Nested Inline Element Parsing

## Context

The markdown parser already recognizes 6 inline element types (code spans, emphasis, italic, links, images, strikethrough) in `parse_inline_list()`. However, content inside these elements is parsed as flat `MdTextual` nodes — no nesting occurs. For example, `**bold *italic* more**` produces flat textual tokens instead of nesting `MdInlineItalic` inside `MdInlineEmphasis`.

This change adds nested inline parsing so the AST correctly represents inline structure.

## Scope

- **Add nesting** to emphasis, italic, strikethrough, link text, and image alt content loops
- **Add parser spec tests** for all inline types (none currently exist)
- **No grammar/codegen/lexer/formatter changes needed** — the grammar already supports nesting via `MdInlineItemList = AnyMdInline*`

## Key Design

### Extract reusable helpers

Two new functions in `syntax.rs`:

1. **`try_parse_one_inline(p) -> bool`** — Encapsulates the inline dispatch logic from `parse_inline_list`. Tries code/image/link/strikethrough/emphasis in order. Returns true if something was parsed, false if caller should bump as MdTextual.

2. **`parse_inline_content_until_delimiter(p, delimiter_text) -> bool`** — Parses inline content with nesting until the given delimiter text is found. Returns whether the delimiter was found. Used in content loops of emphasis/italic/strikethrough/link/image.

### Same-type nesting prevention

The stop condition `p.cur_text() == delimiter` is checked BEFORE the inline dispatch. So inside `**...**`, encountering `**` terminates the content loop rather than trying nested emphasis. This is correct behavior.

### Code spans stay flat

`parse_inline_code` is NOT changed — per CommonMark, code span content is raw text with no nested inline elements.

### Link/image source stays flat

URL content in `(...)` of links and images is NOT parsed for nested inlines. Only the text/alt slots get nesting.

## Implementation Steps

### Step 1: Add `try_parse_one_inline` helper

Extract from `parse_inline_list` dispatch:

```rust
fn try_parse_one_inline(p: &mut MarkdownParser) -> bool {
    if at_inline_code(p) {
        parse_inline_code(p);
        return true;
    }
    if at_inline_image_start(p) {
        return try_parse_inline_image(p);
    }
    if at_inline_link_start(p) {
        return try_parse_inline_link(p);
    }
    if at_inline_strikethrough_start(p) {
        return try_parse_inline_strikethrough(p);
    }
    if at_inline_emphasis_start(p) {
        return try_parse_inline_emphasis_or_italic(p);
    }
    false
}
```

Refactor `parse_inline_list` to use it (pure refactor, no behavior change).

### Step 2: Add `parse_inline_content_until_delimiter` helper

```rust
fn parse_inline_content_until_delimiter(
    p: &mut MarkdownParser,
    delimiter_text: &str,
) -> bool {
    let mut found = false;
    while !p.at(T![EOF]) && !p.has_preceding_line_break() {
        if p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == delimiter_text {
            found = true;
            break;
        }
        if !try_parse_one_inline(p) {
            let textual = p.start();
            p.bump_any();
            textual.complete(p, MD_TEXTUAL);
        }
    }
    found
}
```

### Step 3: Update 5 content loops

Replace the flat `MdTextual` content loops in:

| Function | Delimiter | Content slot |
|----------|-----------|-------------|
| `try_parse_inline_emphasis` | `"**"` or `"__"` | content (slot 1) |
| `try_parse_inline_italic` | `"*"` or `"_"` | content (slot 1) |
| `try_parse_inline_strikethrough` | `"~~"` | content (slot 1) |
| `try_parse_inline_link` | `"]"` | text (slot 1) only, NOT source (slot 4) |
| `try_parse_inline_image` | `"]"` | alt_content only, NOT src_content |

Each replacement follows this pattern:
```rust
// Before (flat):
while !p.at(T![EOF]) && !p.has_preceding_line_break() {
    if p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == delimiter { found_close = true; break; }
    let textual = p.start(); p.bump_any(); textual.complete(p, MD_TEXTUAL);
}

// After (nested):
let found_close = parse_inline_content_until_delimiter(p, &delimiter);
```

### Step 4: Add parser spec tests

New test files in `crates/biome_markdown_parser/tests/md_test_suite/ok/`:

**`inline_code.md`** — Basic inline code:
```
This has `inline code` in it.
```

**`inline_emphasis.md`** — Bold and italic:
```
**bold text** and *italic text*
```

**`inline_links.md`** — Links and images:
```
[link text](https://example.com) and ![alt](image.png)
```

**`inline_nesting.md`** — Nested inline elements:
```
**bold *italic* more**

*italic `code` more*

~~strike **bold** more~~

[**bold link**](url)

**bold [link](url) more**
```

### Step 5: Accept snapshots and verify

```bash
cargo test -p biome_markdown_parser     # Will fail with new snapshots
cargo insta accept --workspace          # Accept new snapshots
cargo test -p biome_markdown_parser     # Should pass
cargo test -p biome_markdown_formatter  # Should pass (formatters use text_trimmed)
cargo test -p biome_markdown_analyze    # Should pass (lint rules use text utilities)
cargo test -p biome_cli                 # Should pass
```

## Files to modify

| File | Change |
|------|--------|
| `crates/biome_markdown_parser/src/syntax.rs` | Add 2 helpers, refactor `parse_inline_list`, update 5 content loops |
| `crates/biome_markdown_parser/tests/md_test_suite/ok/inline_code.md` | New test |
| `crates/biome_markdown_parser/tests/md_test_suite/ok/inline_emphasis.md` | New test |
| `crates/biome_markdown_parser/tests/md_test_suite/ok/inline_links.md` | New test |
| `crates/biome_markdown_parser/tests/md_test_suite/ok/inline_nesting.md` | New test |

Files that should NOT change:
- Lexer — no changes needed
- Grammar/codegen — `MdInlineItemList` already supports nesting
- Formatter — text-blob pattern works regardless of AST nesting
- Lint rules — use text-based utilities, not AST inline nodes

## Nesting behavior

| Input | Result |
|-------|--------|
| `**bold *italic* more**` | `MdInlineEmphasis` containing `MdInlineItalic` |
| `*italic **bold** more*` | `MdInlineItalic` containing `MdInlineEmphasis` |
| `**bold `code` more**` | `MdInlineEmphasis` containing `MdInlineCode` |
| `[**bold**](url)` | `MdInlineLink` text slot containing `MdInlineEmphasis` |
| `` `code *not italic*` `` | `MdInlineCode` with flat content (correct) |
| `**bold ** not**` | Stop condition fires at inner `**` — same-type nesting prevented |
