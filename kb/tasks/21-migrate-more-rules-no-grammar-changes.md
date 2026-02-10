# Plan: Migrate More Rules Without Grammar Changes

**Created:** 2026-02-10

---

## Goal

Migrate 9 more lint rules from `Ast<MdDocument>` + FenceTracker to specific AST node queries using the existing grammar. These rules use FenceTracker purely to skip fenced code blocks — since the parser already creates `MdFencedCodeBlock` nodes, paragraphs inside fences are not `MdParagraph` nodes, so FenceTracker is unnecessary.

## Rules to Migrate

### Group A: Directive rules → `Ast<MdParagraph>` (4 rules)

These rules scan paragraph text for directive syntax (`::name{attrs}`). Directives only appear in paragraph content, never inside fenced code blocks.

| Rule | Current Query | New Query | Notes |
|------|--------------|-----------|-------|
| `no_directive_duplicate_attribute` | `MdDocument` | `MdParagraph` | Scans for duplicate attrs in directives |
| `use_sorted_directive_attributes` | `MdDocument` | `MdParagraph` | Checks attr ordering |
| `use_directive_shortcut_attribute` | `MdDocument` | `MdParagraph` | `id="val"` → `#val` |
| `use_directive_collapsed_attribute` | `MdDocument` | `MdParagraph` | `class="val"` → `.val` |

### Group B: MDX JSX rules → `Ast<MdParagraph>` (5 rules)

These rules scan paragraph text for MDX JSX elements. Like directives, JSX only appears in paragraph content.

| Rule | Current Query | New Query | Notes |
|------|--------------|-----------|-------|
| `no_mdx_jsx_duplicate_attribute` | `MdDocument` | `MdParagraph` | Duplicate JSX attrs |
| `no_mdx_jsx_void_children` | `MdDocument` | `MdParagraph` | Void elements with children |
| `use_sorted_mdx_jsx_attributes` | `MdDocument` | `MdParagraph` | Attr ordering |
| `use_mdx_jsx_shorthand_attribute` | `MdDocument` | `MdParagraph` | `prop={true}` → `prop` |
| `use_mdx_jsx_self_closing` | `MdDocument` | `MdParagraph` | `<X></X>` → `<X />` |

## Migration Pattern

Each rule follows this transformation:

**Before:**
```rust
type Query = Ast<MdDocument>;
fn run(ctx: &RuleContext<Self>) -> Self::Signals {
    let document = ctx.query();
    let text = document.syntax().text_trimmed().to_string();
    let base = document.syntax().text_trimmed_range().start();
    let mut tracker = FenceTracker::new();
    for (line_idx, line) in text.lines().enumerate() {
        tracker.process_line(line_idx, line);
        if tracker.is_inside_fence() { continue; }
        // ... scan line ...
    }
}
```

**After:**
```rust
type Query = Ast<MdParagraph>;
fn run(ctx: &RuleContext<Self>) -> Self::Signals {
    let paragraph = ctx.query();
    let text = paragraph.syntax().text_trimmed().to_string();
    let base = paragraph.syntax().text_trimmed_range().start();
    for line in text.lines() {
        // ... scan line ...
    }
}
```

The key change: replace document-level iteration with paragraph-level iteration, remove FenceTracker entirely.

## Implementation Steps

1. Migrate 4 directive rules (batch)
2. Migrate 5 MDX JSX rules (batch)
3. Run tests, accept any snapshot changes
4. Verify all 200 analyzer spec tests pass

## Rules That Cannot Migrate (remain at MdDocument)

These 22 rules legitimately need document-level state:

- **Consistency rules (8):** emphasis/strong/strikethrough markers, link/media style, link title style, directive/MDX quote style — need first-seen style tracking
- **Cross-reference rules (4):** unused/undefined definitions, invalid link fragments, required headings
- **Document-wide scanning (6):** hard tabs, long lines, consecutive blank lines, blanks around headings/lists/tables
- **Code block rules (3):** blanks around code fences, shell dollar prompt, consistent code fence marker — query `MdFencedCodeBlock` once grammar supports it better
- **Other (1):** `no_paragraph_content_indent` — needs FenceTracker for indentation context

## Risk: Low

Same pattern as the 16 rules already migrated. FenceTracker removal is safe because `MdParagraph` nodes don't exist inside `MdFencedCodeBlock`.
