# Plan: MDX/Directive Grammar Support

**Created:** 2026-02-10

---

## Goal

Add AST-level parsing for MDX JSX elements and markdown directives, enabling 11 rules to migrate from text-based scanning to AST queries.

## Current State

- 5 directive rules use `directive_utils.rs` (306 lines) to parse `::name{attr="val"}` from text
- 6 MDX JSX rules use `mdx_utils.rs` (279 lines) to parse `<Component prop="val" />` from text
- All 11 rules query `Ast<MdDocument>` and use FenceTracker

After plan 21, these rules will query `Ast<MdParagraph>` but still use text-based utilities.

## Grammar Design

### Directive Nodes

Based on the [CommonMark Generic Directive Extension](https://github.com/micromark/micromark-extension-directive):

```ungram
// Text directive: :name{attrs}
// Leaf directive: ::name{attrs}
// Container directive: :::name{attrs}\n...\n:::
MdDirective =
    marker: 'md_textual_literal'   // ':', '::', or ':::'
    name: MdTextual
    attributes: MdDirectiveAttributeList?

MdDirectiveAttributeList = MdDirectiveAttribute*

MdDirectiveAttribute =
    name: MdTextual
    '='?
    value: MdDirectiveAttributeValue?

MdDirectiveAttributeValue =
    delimiter: 'md_textual_literal'   // '"' or "'"
    content: MdTextual
    closing_delimiter: 'md_textual_literal'
```

### MDX JSX Nodes

Based on MDX spec:

```ungram
MdMdxJsxElement =
    '<'
    name: MdTextual
    attributes: MdMdxJsxAttributeList?
    self_closing: '/'?
    '>'

MdMdxJsxAttributeList = MdMdxJsxAttribute*

MdMdxJsxAttribute =
    name: MdTextual
    '='?
    value: MdMdxJsxAttributeValue?

MdMdxJsxAttributeValue =
    delimiter: 'md_textual_literal'
    content: MdTextual
    closing_delimiter: 'md_textual_literal'
```

## Rules Enabled

### Directive rules → `Ast<MdDirective>`
| Rule | AST Access |
|------|-----------|
| `no_directive_duplicate_attribute` | `.attributes()` → compare names |
| `use_sorted_directive_attributes` | `.attributes()` → check order |
| `use_directive_shortcut_attribute` | `.attributes()` → find `id="val"` |
| `use_directive_collapsed_attribute` | `.attributes()` → find `class="val"` |
| `use_consistent_directive_quote_style` | `.attributes()` → compare delimiters |

### MDX JSX rules → `Ast<MdMdxJsxElement>`
| Rule | AST Access |
|------|-----------|
| `no_mdx_jsx_duplicate_attribute` | `.attributes()` → compare names |
| `no_mdx_jsx_void_children` | `.name()` → check void element list |
| `use_sorted_mdx_jsx_attributes` | `.attributes()` → check order |
| `use_mdx_jsx_shorthand_attribute` | `.attributes()` → find `prop={true}` |
| `use_mdx_jsx_self_closing` | `.self_closing()` presence |
| `use_consistent_mdx_jsx_quote_style` | `.attributes()` → compare delimiters |

## Implementation Steps

### Phase 1: Directive Grammar
1. Add directive kinds to `markdown_kinds_src.rs`
2. Add directive nodes to `markdown.ungram`
3. Run codegen
4. Implement directive parsing in `syntax.rs` (detect `:`, `::`, `:::` patterns in inline content)
5. Add directive formatters
6. Accept snapshots

### Phase 2: Directive Rule Migration
1. Migrate 5 directive rules to `Ast<MdDirective>`
2. Remove `directive_utils.rs` (306 lines)

### Phase 3: MDX JSX Grammar
1. Add MDX JSX kinds
2. Add MDX JSX nodes to grammar
3. Run codegen
4. Implement MDX JSX parsing (detect `<ComponentName` patterns)
5. Add MDX JSX formatters
6. Accept snapshots

### Phase 4: MDX JSX Rule Migration
1. Migrate 6 MDX JSX rules to `Ast<MdMdxJsxElement>`
2. Remove `mdx_utils.rs` (279 lines)

## New Kinds Needed

```
MD_DIRECTIVE
MD_DIRECTIVE_ATTRIBUTE_LIST
MD_DIRECTIVE_ATTRIBUTE
MD_DIRECTIVE_ATTRIBUTE_VALUE
MD_MDX_JSX_ELEMENT
MD_MDX_JSX_ATTRIBUTE_LIST
MD_MDX_JSX_ATTRIBUTE
MD_MDX_JSX_ATTRIBUTE_VALUE
```

## Risk: High

- Directive and MDX specs are complex with many edge cases
- Detection in inline content requires careful interaction with existing inline parsing
- MDX is not standard markdown — may need feature flags or MDX-specific mode
- These rules are least commonly used, so ROI may be low

## Dependencies

- Plan 22 (parser improvements) should be completed first for stable inline parsing
- Plan 23 (grammar redesign) should be completed first for the pattern

## Utility Eliminated

| File | Lines | Replaced By |
|------|-------|-------------|
| `directive_utils.rs` | 306 | `MdDirective` AST accessors |
| `mdx_utils.rs` | 279 | `MdMdxJsxElement` AST accessors |
| **Total** | **585** | |
