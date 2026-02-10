# AST-First Architecture & Analyzer Technical Debt

**Created:** 2026-02-10

---

## Problem Statement

All 100 markdown lint rules in Biome bypass the AST and reparse the source text line-by-line. This is an architectural anomaly — every other language in Biome (JS, CSS, JSON, GraphQL) has rules that query specific AST node types and traverse the tree.

The 730 lines of shadow parser code in the analyzer utilities exist because the real parser didn't produce the structure the rules needed:
- `list_utils.rs` (357 lines) — re-extracts list items, markers, indentation, checkboxes from text
- `blockquote_utils.rs` (216 lines) — re-extracts blockquote blocks, markers, nesting from text
- `fence_utils.rs` (156 lines) — re-tracks fenced code block state to skip content

All 100 rules follow this anti-pattern:
```rust
impl Rule for SomeRule {
    type Query = Ast<MdDocument>;
    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let text = ctx.query().syntax().text_with_trivia().to_string();
        // ... line-by-line text parsing, ignoring the AST entirely
    }
}
```

The correct pattern (used by every other Biome language) is:
```rust
impl Rule for SomeRule {
    type Query = Ast<MdBulletListItem>;  // specific node type
    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let node = ctx.query();
        // ... traverse AST children directly
    }
}
```

---

## Industry Standard: AST-First

### remark-lint (gold standard)
- Fully AST-based. Parses into mdast (markdown abstract syntax tree) with proper nesting for blockquotes, lists, and all container structures.
- Rules traverse the AST using unist visitor utilities.
- No text reparsing.
- Source: https://github.com/remarkjs/remark-lint

### markdownlint
- Dual approach: micromark produces a hierarchical token tree; rules receive both tokens and raw lines.
- Rules declare which parser they need (`"micromark"`, `"markdownit"`, or `"none"`).
- Started as line-based, migrated to AST-based (micromark) as the primary path.
- Helper functions like `filterByTypes()` enable AST traversal without parsing details.
- Source: https://github.com/DavidAnson/markdownlint

### @eslint/markdown
- Parses into mdast via `mdast-util-from-markdown`.
- Rules use visitor-pattern AST traversal (`heading()`, `image()`, `text()` visitors).
- Identical architecture to ESLint JS rules.
- Source: https://github.com/eslint/markdown

### Biome (all other languages)
- JS rules query `Ast<JsVariableDeclaration>`, CSS rules query `Ast<CssDeclaration>`, etc.
- Rules pattern-match on specific node types and traverse the tree.
- Never call `.text_with_trivia().to_string()` to reparse source.

---

## Root Cause

The markdown parser was initially incomplete — it didn't produce proper AST structures for blockquotes, lists, or inline elements. The lint rules were written against a flat AST where most content was `MdTextual` tokens. The text-based utilities were a pragmatic workaround to ship 100 rules quickly.

The parser has since been significantly expanded (12 block types, 8 inline types, nested inline parsing, multi-line blockquotes/lists), but the analyzer rules were never updated to use the improved AST.

---

## Remaining Parser Prerequisites

Before rules can migrate to AST-based analysis, the parser must produce correct, complete structure for all constructs the rules inspect:

| Parser Gap | Required By Rules | Current Workaround |
|------------|-------------------|-------------------|
| Nested blockquotes (`> > nested`) | `noBlockquoteBrokenContinuation`, `useConsistentBlockquoteIndent` | `blockquote_utils.rs` counts `>` markers per line |
| Nested lists (sub-items at deeper indent) | 9 list rules | `list_utils.rs` infers nesting from indentation |
| Lazy continuation (lines without `>` in blockquotes) | `noBlockquoteBrokenContinuation` | `blockquote_utils.rs` detects missing markers |
| Task list checkbox AST node (`- [ ] task`) | `noCheckboxCharacterStyleMismatch`, `noCheckboxContentIndent` | `list_utils.rs` regex-parses `[ ]`/`[x]` from text |
| Multi-paragraph list items (blank line within item) | `useConsistentListItemSpacing` | Text scanning for blank lines between items |

### Grammar Changes Required

The current grammar uses simplified models that prevent proper nesting:

```
// Current — single child, no nesting
MdQuote = AnyMdBlock

// Needed — block list allows nested content (paragraphs, code, sub-blockquotes)
MdQuote = '>' content: MdBlockList
```

```
// Current — flat inline content only
MdBullet = bullet: ('-' | '*' | '+') content: MdInlineItemList

// Needed — block content allows nested lists, paragraphs, code blocks
MdBullet = bullet: ('-' | '*' | '+') content: MdBlockList
```

These grammar changes propagate through codegen (4 generated files), formatter, and all rules that reference these nodes.

---

## Migration Path

### Phase 1: Complete Parser AST (prerequisites)
1. Grammar changes for `MdQuote` and `MdBullet`/`MdOrderBullet` to support block-level content
2. Nested blockquote parsing (recursive `>` stripping)
3. Nested list parsing (indentation-based nesting)
4. Lazy continuation in blockquotes
5. Task list checkbox AST node
6. Multi-paragraph list items (blank line handling)
7. Update formatters for new AST structure
8. Verify all 200 analyzer spec tests still pass (rules read text, so AST changes shouldn't break them)

### Phase 2: Migrate Rules to AST-Based Analysis
1. Migrate blockquote rules (2 rules) — replace `blockquote_utils.rs` with AST traversal
2. Migrate list rules (9 rules) — replace `list_utils.rs` with AST traversal
3. Migrate checkbox rules (2 rules) — use new checkbox AST node
4. Migrate remaining rules that scan for structure already in the AST
5. Remove shadow parser utilities once no rules depend on them

### Phase 3: Cleanup
1. Delete `list_utils.rs`, `blockquote_utils.rs`, `fence_utils.rs` (730 lines)
2. Update rule `Query` types from `Ast<MdDocument>` to specific node types
3. Verify all 200 analyzer spec tests still produce identical diagnostics

---

## Scope & Effort

- **Phase 1** (parser): ~1000 lines of parser changes, grammar codegen, formatter updates
- **Phase 2** (rules): ~100 rules to audit, ~30 rules to rewrite (those using text utilities)
- **Phase 3** (cleanup): ~730 lines deleted, net reduction in code

The parser work is the critical path. Once the AST is correct, rule migration can be incremental — each rule can be migrated independently without affecting others.

---

## Key Insight

The remaining parser gaps aren't "nice-to-have" — they're prerequisites for the analyzer to work the way every other Biome language works. The text-based approach is technical debt, not a design choice. The end goal is eliminating the shadow parser entirely and having rules that pattern-match on AST nodes.
