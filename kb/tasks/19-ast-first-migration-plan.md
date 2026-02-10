# AST-First Migration Plan: Gold Standard Markdown Architecture

**Created:** 2026-02-10

---

## Executive Summary

Migrate Biome's markdown analyzer from a text-reparsing architecture to a fully AST-based architecture, matching the gold standard set by remark-lint (mdast), markdownlint (micromark), and Biome's own JS/CSS/JSON/GraphQL analyzers.

**Current state:** 100 rules query `Ast<MdDocument>` and reparse source text line-by-line using 2,856 lines of shadow parser utilities across 10 files.

**Target state:** Rules query specific AST node types (`Ast<MdBullet>`, `Ast<MdQuote>`, `Ast<MdHeader>`, etc.) and traverse the tree. Shadow parser utilities deleted.

---

## Table of Contents

1. [Architecture Comparison](#1-architecture-comparison)
2. [Grammar Redesign](#2-grammar-redesign)
3. [Parser Changes](#3-parser-changes)
4. [Codegen & Formatter Cascade](#4-codegen--formatter-cascade)
5. [Rule Migration Strategy](#5-rule-migration-strategy)
6. [Utility Deprecation Schedule](#6-utility-deprecation-schedule)
7. [Phased Implementation](#7-phased-implementation)
8. [Risk Assessment](#8-risk-assessment)

---

## 1. Architecture Comparison

### Current Architecture (Anti-Pattern)

```
Source Text
    |
    v
Parser --> Flat AST (MdDocument > MdBlockList > MdParagraph/MdTextual...)
    |
    v
Analyzer Rule (type Query = Ast<MdDocument>)
    |
    v
Shadow Parser Utilities (2,856 lines)
    |-- list_utils.rs (357 lines) --> re-extracts lists from text
    |-- blockquote_utils.rs (216 lines) --> re-extracts blockquotes from text
    |-- fence_utils.rs (156 lines) --> re-tracks fence state from text
    |-- inline_utils.rs (704 lines) --> re-finds inline elements from text
    |-- table_utils.rs (265 lines) --> re-parses tables from text
    |-- definition_utils.rs (312 lines) --> re-parses definitions from text
    |-- directive_utils.rs (306 lines) --> re-parses directives from text
    |-- heading_utils.rs (138 lines) --> re-extracts headings from text
    |-- mdx_utils.rs (279 lines) --> re-parses MDX/JSX from text
    |-- line_utils.rs (113 lines) --> line splitting utilities
    v
Diagnostics
```

### Target Architecture (Gold Standard)

```
Source Text
    |
    v
Parser --> Rich CST (proper nesting, all constructs as typed nodes)
    |
    v
Analyzer Rule (type Query = Ast<MdBullet>)  // specific node type
    |
    v
AST node accessors (bullet.marker(), bullet.content(), bullet.checkbox()...)
    |
    v
Diagnostics
```

### mdast Reference Model

The mdast standard (used by remark-lint) defines the content model hierarchy:

```
Root (FlowContent[])
├── Paragraph (PhrasingContent[])
├── Heading { depth: 1-6 } (PhrasingContent[])
├── Blockquote (FlowContent[])           <-- recursive container
├── List { ordered, start, spread } (ListItem[])
│   └── ListItem { spread, checked } (FlowContent[])  <-- block container
├── Code { lang, meta, value }           <-- literal, no children
├── ThematicBreak                        <-- void node
├── Html { value }                       <-- literal
├── Definition { identifier, url, title }
├── Table { align[] } (TableRow[])
│   └── TableRow (TableCell[])
│       └── TableCell (PhrasingContent[])
├── FootnoteDefinition { identifier } (FlowContent[])
│
PhrasingContent (inline):
├── Text { value }
├── Emphasis (PhrasingContent[])         <-- recursive phrasing
├── Strong (PhrasingContent[])           <-- recursive phrasing
├── InlineCode { value }                 <-- literal, no nesting
├── Link { url, title } (PhrasingContent[])
├── Image { url, title, alt }            <-- void (alt is string, not children)
├── Break                                <-- void
├── Delete (PhrasingContent[])           <-- GFM strikethrough
├── LinkReference { identifier, referenceType } (PhrasingContent[])
├── ImageReference { identifier, referenceType, alt }
└── FootnoteReference { identifier }
```

**Key difference from mdast:** Biome uses a Concrete Syntax Tree (CST) that preserves all tokens (brackets, fences, hash marks, etc.) for formatting. mdast is an Abstract Syntax Tree that discards syntactic markers. Our grammar must preserve tokens while achieving the same structural nesting.

---

## 2. Grammar Redesign

### 2.1 Blockquote: Single Child -> Block Container

```ungram
// CURRENT (broken - single child, no nesting)
MdQuote = AnyMdBlock

// NEW (block container like mdast Blockquote)
MdQuote =
    '>'
    content: MdBlockList
```

**What this enables:**
- `MdQuote` can contain paragraphs, code blocks, lists, nested blockquotes
- Lazy continuation lines (lines without `>`) belong to the blockquote
- Rules can traverse `quote.content()` to find nested structure
- Replaces `blockquote_utils.rs` (216 lines)

### 2.2 Lists: Inline-Only -> Block Container

```ungram
// CURRENT (inline content only, no block children)
MdBullet =
    bullet: ('-' | '*' | '+')
    content: MdInlineItemList

MdOrderBullet =
    marker: 'md_textual_literal'
    content: MdInlineItemList

// NEW (block content, checkbox support, matches mdast ListItem)
MdBullet =
    bullet: ('-' | '*' | '+')
    checkbox: MdCheckbox?
    content: MdBlockList

MdOrderBullet =
    marker: 'md_textual_literal'
    checkbox: MdCheckbox?
    content: MdBlockList
```

**What this enables:**
- List items can contain paragraphs, code blocks, nested lists, blockquotes
- Checkbox is a first-class AST node (not regex-parsed from text)
- Rules can traverse `bullet.content()` to find nested structure
- Replaces `list_utils.rs` (357 lines)

### 2.3 New Node: Task List Checkbox

```ungram
// NEW
MdCheckbox =
    '['
    value: 'md_textual_literal'
    ']'
```

**AST accessor:** `checkbox.value_token()` returns `" "`, `"x"`, or `"X"`.

**What this enables:**
- `no_checkbox_character_style_mismatch` queries `Ast<MdCheckbox>` directly
- `no_checkbox_content_indent` inspects checkbox position in AST
- No regex parsing of `[ ]`/`[x]` from text

### 2.4 New Node: Link Definition (Structured)

```ungram
// CURRENT (flat inline content)
MdLinkBlock = content: MdInlineItemList

// NEW (structured with semantic slots)
MdLinkDefinition =
    '['
    label: MdTextual
    ']'
    ':'
    url: MdInlineItemList
    title: MdLinkDefinitionTitle?

MdLinkDefinitionTitle =
    delimiter: ('"' | "'" | '(')
    content: MdTextual
    closing_delimiter: ('"' | "'" | ')')
```

**What this enables:**
- `no_duplicate_definitions` queries `Ast<MdLinkDefinition>` and compares labels
- `no_unused_definitions` collects all definitions by label
- `use_sorted_definitions` sorts by `definition.label()`
- `use_definitions_at_end` checks position of definition nodes
- Replaces `definition_utils.rs` (312 lines)

### 2.5 Table Cells: Flat Row -> Cell List

```ungram
// CURRENT (flat inline content for entire row)
MdTableRow =
    content: MdInlineItemList

// NEW (structured cells, matches mdast TableRow > TableCell)
MdTableRow =
    cells: MdTableCellList

MdTableCellList = MdTableCell*

MdTableCell =
    '|'?
    content: MdInlineItemList
```

Note: `MD_TABLE_CELL` and `MD_TABLE_CELL_LIST` already exist in `markdown_kinds_src.rs` but are not in the grammar. This change adds them.

**What this enables:**
- `no_hidden_table_cell` compares cell counts between rows via AST
- `no_mismatched_table_column_count` counts cells directly
- `use_consistent_table_cell_padding` inspects individual cell text ranges
- Replaces `table_utils.rs` (265 lines)

### 2.6 Fenced Code Block: Structured Info String

```ungram
// CURRENT (flat token list for info string)
MdFencedCodeBlock =
    l_fence: '```'
    code_list: MdCodeNameList
    content: MdInlineItemList
    r_fence: '```'

// NEW (structured language and metadata)
MdFencedCodeBlock =
    l_fence: '```'
    language: MdTextual?
    meta: MdTextual?
    content: MdInlineItemList
    r_fence: '```'
```

**What this enables:**
- `no_missing_language` checks `code_block.language().is_none()`
- `no_shell_dollar_prompt` checks `code_block.language()` text
- `use_consistent_code_fence_marker` inspects `l_fence_token()` directly

### 2.7 Update AnyLeafBlock

```ungram
// CURRENT
AnyLeafBlock =
    MdThematicBreakBlock
    | MdHeader
    | MdSetextHeader
    | AnyCodeBlock
    | MdHtmlBlock
    | MdLinkBlock          // <-- old flat link block
    | MdTable
    | MdParagraph

// NEW
AnyLeafBlock =
    MdThematicBreakBlock
    | MdHeader
    | MdSetextHeader
    | AnyCodeBlock
    | MdHtmlBlock
    | MdLinkDefinition     // <-- new structured node
    | MdTable
    | MdParagraph
```

### 2.8 Summary of Grammar Changes

| Change | New Nodes | Removed Nodes | Impact |
|--------|-----------|---------------|--------|
| Blockquote nesting | - | - | `MdQuote` gains `>` token + `MdBlockList` content |
| List block content | `MdCheckbox` | - | `MdBullet`/`MdOrderBullet` gain checkbox + `MdBlockList` content |
| Link definitions | `MdLinkDefinition`, `MdLinkDefinitionTitle` | `MdLinkBlock` | Structured label/url/title |
| Table cells | - (use existing kinds) | - | `MdTableRow` gains `MdTableCellList` |
| Code block info | - | `MdCodeNameList` | `MdFencedCodeBlock` gains language/meta slots |

---

## 3. Parser Changes

### 3.1 Blockquote Parser

**Current:** Parses `>` marker + inline content as flat paragraph child.

**New behavior:**
1. Consume `>` marker token
2. Strip `>` prefix from continuation lines (including lazy continuation)
3. Recursively call `parse_block_list()` on stripped content
4. Handle nested `> >` by recursive descent (each `>` level creates a new `MdQuote`)

```
> Paragraph one
>
> - List item
> > Nested quote

becomes:

MdQuote
  '>'
  MdBlockList
    MdParagraph("Paragraph one")
    MdBulletListItem
      MdBulletList
        MdBullet('-', "List item")
    MdQuote
      '>'
      MdBlockList
        MdParagraph("Nested quote")
```

**Complexity:** This is the hardest parser change. The `>` stripping must happen at the token/lexer level since the block parser expects clean tokens. Two approaches:

- **Approach A (token rewriting):** Before parsing blockquote content, create a sub-token-source that strips leading `>` and whitespace from each line's tokens. Feed this to a recursive `parse_block_list()` call.
- **Approach B (marker tracking):** Track the current blockquote nesting depth. At each line start, consume the expected number of `>` markers. If a line has fewer markers, it's either lazy continuation (if preceding content allows) or end of blockquote.

Approach B is simpler and matches how CommonMark specifies blockquote parsing.

### 3.2 List Item Parser

**Current:** Parses marker + inline content. Multi-line items consume continuation lines with sufficient indentation.

**New behavior:**
1. Consume marker token (-, *, +, or 1., 2., etc.)
2. Optionally consume checkbox `[ ]`/`[x]`/`[X]` and create `MdCheckbox` node
3. Calculate content indentation (marker width + spaces after marker)
4. Call `parse_block_list()` for item content, where each block must be indented at least to content column
5. Blank lines within the item are allowed (loose list items)
6. Item ends when a line with insufficient indentation is found

```
- First paragraph
  with continuation

  Second paragraph

  ```
  code block
  ```

  - Nested list

becomes:

MdBullet
  '-'
  MdBlockList
    MdParagraph("First paragraph with continuation")
    MdParagraph("Second paragraph")
    MdFencedCodeBlock("code block")
    MdBulletListItem
      MdBulletList
        MdBullet('-', "Nested list")
```

**Key challenge:** Indentation tracking. The parser must know the "content column" (indent level where content starts after the marker) and only consume blocks that are indented at least that much.

### 3.3 Table Cell Parser

**Current:** Entire row is `MdInlineItemList`.

**New behavior:**
1. Split row content by unescaped `|` characters
2. Create `MdTableCell` node for each cell
3. Parse cell content as inline elements

### 3.4 Link Definition Parser

**Current:** Parses `[label]: url` into flat `MdLinkBlock`.

**New behavior:**
1. Parse `[`, label text, `]`, `:`
2. Parse URL (until whitespace or end of line)
3. Optionally parse title in `"..."`, `'...'`, or `(...)`
4. Create structured `MdLinkDefinition` node

### 3.5 Checkbox Parser

**New:**
1. After list marker and space, check for `[`, space/x/X, `]`
2. Create `MdCheckbox` node with value token
3. Remaining content follows as normal

### 3.6 Parser Change Summary

| Parser Area | Complexity | Lines (est.) | Blocks |
|-------------|-----------|-------------|--------|
| Blockquote nesting | High | ~150 | Phase 1 |
| List item block content | High | ~200 | Phase 1 |
| Checkbox parsing | Low | ~30 | Phase 1 |
| Table cell splitting | Medium | ~80 | Phase 2 |
| Link definition structure | Medium | ~60 | Phase 2 |
| Code block info string | Low | ~20 | Phase 2 |

---

## 4. Codegen & Formatter Cascade

Grammar changes propagate through the build system:

### 4.1 Files Auto-Generated by Codegen

After grammar changes, run `just gen-analyzer` (or the markdown codegen subset) to regenerate:

| File | Contents |
|------|----------|
| `crates/biome_markdown_syntax/src/generated/kind.rs` | `MdSyntaxKind` enum with new node kinds |
| `crates/biome_markdown_syntax/src/generated/nodes.rs` | Typed AST node structs with accessor methods |
| `crates/biome_markdown_syntax/src/generated/nodes_mut.rs` | Mutable AST node variants |
| `crates/biome_markdown_factory/src/generated/node_factory.rs` | Node construction functions |
| `crates/biome_markdown_factory/src/generated/syntax_factory.rs` | Syntax tree factory |
| `crates/biome_markdown_formatter/src/generated.rs` | Formatter trait impls for new nodes |

### 4.2 Formatter Updates

Each new/changed node needs a formatter implementation:

| Node | Formatter File | Strategy |
|------|---------------|----------|
| `MdQuote` (updated) | `md/auxiliary/quote.rs` | Format `>` prefix, indent nested block list |
| `MdBullet` (updated) | `md/auxiliary/bullet.rs` | Format marker, optional checkbox, indent block content |
| `MdOrderBullet` (updated) | `md/auxiliary/order_bullet.rs` | Same as bullet with numbered marker |
| `MdCheckbox` (new) | `md/auxiliary/checkbox.rs` | Format `[ ]` or `[x]` |
| `MdLinkDefinition` (new) | `md/auxiliary/link_definition.rs` | Format `[label]: url "title"` |
| `MdLinkDefinitionTitle` (new) | `md/auxiliary/link_definition_title.rs` | Format title with delimiters |
| `MdTableCell` (new) | `md/auxiliary/table_cell.rs` | Format cell content with padding |
| `MdTableCellList` (new) | `md/lists/table_cell_list.rs` | Format cells separated by `\|` |
| `MdTableRow` (updated) | `md/auxiliary/table_row.rs` | Use cell list instead of flat inline |

### 4.3 Build Verification

After each grammar change + codegen:
```bash
cargo test -p biome_markdown_parser     # Parser produces valid new nodes
cargo test -p biome_markdown_formatter  # Formatters handle new structure
cargo test -p biome_markdown_analyze    # Rules still pass (initially text-based)
cargo test -p biome_cli                 # CLI integration tests pass
```

---

## 5. Rule Migration Strategy

### 5.1 Migration Pattern

Each rule migration follows this template:

**Before:**
```rust
impl Rule for SomeRule {
    type Query = Ast<MdDocument>;
    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        // ... text-based utility calls ...
        let items = collect_list_items(&text);
        for item in items {
            if /* condition */ {
                signals.push(/* diagnostic with text offset */);
            }
        }
    }
}
```

**After:**
```rust
impl Rule for SomeRule {
    type Query = Ast<MdBullet>;  // specific node type
    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let bullet = ctx.query();
        // ... direct AST property access ...
        if let Some(checkbox) = bullet.checkbox() {
            // direct AST traversal
        }
    }
}
```

**Key changes:**
1. `type Query` changes from `Ast<MdDocument>` to specific node type
2. Remove `text_with_trivia().to_string()` call
3. Remove utility function calls (`collect_list_items`, `collect_tables`, etc.)
4. Replace with AST accessor calls on the queried node
5. Diagnostic ranges come from AST node `text_range()` instead of manual offset calculation

### 5.2 Rules by Target AST Node Type

#### Group A: Heading Rules (16 rules) -> `Ast<MdHeader>` / `Ast<MdSetextHeader>`

| Rule | Target Query | AST Info Needed |
|------|-------------|-----------------|
| `no_heading_like_paragraph` | `Ast<MdParagraph>` | Check if text starts with 7+ `#` |
| `no_multiple_top_level_headings` | `Ast<MdHeader>` | Already partially AST-based |
| `use_heading_increment` | `Ast<MdHeader>` | `before().len()` for level |
| `no_missing_space_atx_heading` | `Ast<MdHeader>` | Inspect trivia between hash and content |
| `no_missing_space_closed_atx_heading` | `Ast<MdHeader>` | Inspect trailing hash trivia |
| `no_heading_indent` | `Ast<MdHeader>` | Leading whitespace in trivia |
| `no_heading_content_indent` | `Ast<MdHeader>` | Space count between `#` and text |
| `no_long_headings` | `Ast<MdHeader>` | Content text length |
| `no_heading_trailing_punctuation` | `Ast<MdHeader>` | Last char of content text |
| `no_multiple_space_atx_heading` | `Ast<MdHeader>` | Trivia between hash and content |
| `no_multiple_space_closed_atx_heading` | `Ast<MdHeader>` | Trailing hash trivia |
| `no_duplicate_headings` | `Ast<MdDocument>` | Collect all headers, compare text |
| `no_duplicate_headings_in_section` | `Ast<MdDocument>` | Collect headers by section scope |
| `use_blanks_around_headings` | `Ast<MdHeader>` | Check preceding/following siblings |
| `use_consistent_heading_style` | `Ast<MdDocument>` | Compare ATX vs Setext across doc |
| `use_first_line_heading` | `Ast<MdDocument>` | Check first child of block list |
| `use_required_headings` | `Ast<MdDocument>` | Collect all header levels |

**Utility eliminated:** `heading_utils.rs` (138 lines)

#### Group B: List Rules (16 rules) -> `Ast<MdBullet>` / `Ast<MdOrderBullet>` / `Ast<MdBulletListItem>`

| Rule | Target Query | AST Info Needed |
|------|-------------|-----------------|
| `use_consistent_unordered_list_marker` | `Ast<MdDocument>` | Compare marker tokens across bullets |
| `use_consistent_ordered_list_marker` | `Ast<MdDocument>` | Compare marker tokens across ordered bullets |
| `use_consistent_ordered_list_marker_value` | `Ast<MdOrderBullet>` | Marker number value |
| `no_list_item_bullet_indent` | `Ast<MdBullet>` | Leading whitespace before marker |
| `use_consistent_list_item_indent` | `Ast<MdBullet>` | Content indentation |
| `use_consistent_unordered_list_indent` | `Ast<MdBullet>` | Content indentation |
| `use_consistent_list_indent` | `Ast<MdDocument>` | Compare indentation across all lists |
| `use_consistent_list_item_spacing` | `Ast<MdBulletListItem>` | Sibling spacing (loose/tight) |
| `use_blanks_around_lists` | `Ast<MdBulletListItem>` | Check preceding/following siblings |
| `use_blanks_before_block_content` | `Ast<MdBullet>` | Block children spacing |
| `use_consistent_list_item_content_indent` | `Ast<MdBullet>` | Content column alignment |
| `no_checkbox_character_style_mismatch` | `Ast<MdCheckbox>` | Checkbox value token |
| `no_checkbox_content_indent` | `Ast<MdCheckbox>` | Spacing after checkbox |

**Utility eliminated:** `list_utils.rs` (357 lines)

#### Group C: Blockquote Rules (2 rules) -> `Ast<MdQuote>`

| Rule | Target Query | AST Info Needed |
|------|-------------|-----------------|
| `no_blockquote_broken_continuation` | `Ast<MdQuote>` | Lazy continuation detection (missing `>` lines) |
| `use_consistent_blockquote_indent` | `Ast<MdQuote>` | Indentation after `>` |

**Utility eliminated:** `blockquote_utils.rs` (216 lines)

#### Group D: Table Rules (7 rules) -> `Ast<MdTable>` / `Ast<MdTableRow>` / `Ast<MdTableCell>`

| Rule | Target Query | AST Info Needed |
|------|-------------|-----------------|
| `no_hidden_table_cell` | `Ast<MdTable>` | Compare cell counts between rows |
| `no_mismatched_table_column_count` | `Ast<MdTable>` | Cell count consistency |
| `use_consistent_table_cell_padding` | `Ast<MdTableCell>` | Cell content spacing |
| `use_consistent_table_pipe_style` | `Ast<MdTableRow>` | Leading/trailing pipe tokens |
| `use_consistent_table_pipe_alignment` | `Ast<MdTable>` | Pipe position alignment |
| `use_blanks_around_tables` | `Ast<MdTable>` | Preceding/following siblings |
| `no_table_indentation` | `Ast<MdTable>` | Leading whitespace |

**Utility eliminated:** `table_utils.rs` (265 lines)

#### Group E: Code Block Rules (6 rules) -> `Ast<MdFencedCodeBlock>` / `Ast<MdIndentCodeBlock>`

| Rule | Target Query | AST Info Needed |
|------|-------------|-----------------|
| `use_consistent_code_block_style` | `Ast<MdDocument>` | Compare fenced vs indented across doc |
| `use_blanks_around_code_fences` | `Ast<MdFencedCodeBlock>` | Preceding/following siblings |
| `use_consistent_code_fence_marker` | `Ast<MdFencedCodeBlock>` | `l_fence_token()` character |
| `no_missing_language` | `Ast<MdFencedCodeBlock>` | `language()` presence |
| `no_shell_dollar_prompt` | `Ast<MdFencedCodeBlock>` | Language + content text |
| `no_hard_tabs` | `Ast<MdDocument>` | Document-wide tab detection |

**Utility eliminated:** `fence_utils.rs` (156 lines)

#### Group F: Inline Element Rules (19 rules) -> Various inline node types

| Rule | Target Query | AST Info Needed |
|------|-------------|-----------------|
| `no_empty_links` | `Ast<MdInlineLink>` | `source()` emptiness |
| `no_reversed_links` | `Ast<MdParagraph>` | Text pattern `(text)[url]` |
| `no_bare_urls` | `Ast<MdParagraph>` | URL patterns in text nodes |
| `use_consistent_link_style` | `Ast<MdDocument>` | Compare inline vs reference links |
| `no_space_in_links` | `Ast<MdInlineLink>` | Spacing in text/source slots |
| `use_consistent_link_title_style` | `Ast<MdInlineLink>` | Title quote style |
| `no_shortcut_reference_link` | `Ast<MdDocument>` | Reference link form detection |
| `no_unneeded_full_reference_link` | `Ast<MdDocument>` | Reference link simplification |
| `no_shortcut_reference_image` | `Ast<MdDocument>` | Reference image form |
| `no_unneeded_full_reference_image` | `Ast<MdDocument>` | Reference image simplification |
| `no_missing_alt_text` | `Ast<MdInlineImage>` | `alt().content()` emptiness |
| `use_descriptive_link_text` | `Ast<MdInlineLink>` | Link text quality |
| `use_consistent_emphasis_marker` | `Ast<MdInlineItalic>` | `l_fence_token()` character |
| `use_consistent_strong_marker` | `Ast<MdInlineEmphasis>` | `l_fence_token()` character |
| `no_emphasis_as_heading` | `Ast<MdParagraph>` | Single emphasis child |
| `no_space_in_emphasis` | `Ast<MdInlineEmphasis>` | Leading/trailing spaces in content |
| `no_space_in_code` | `Ast<MdInlineCode>` | Leading/trailing spaces in content |
| `no_inline_html` | `Ast<MdHtmlBlock>` | Presence detection |
| `use_consistent_strikethrough_marker` | `Ast<MdInlineStrikethrough>` | Marker style |

**Utility eliminated:** `inline_utils.rs` (704 lines)

#### Group G: Definition Rules (8 rules) -> `Ast<MdLinkDefinition>`

| Rule | Target Query | AST Info Needed |
|------|-------------|-----------------|
| `no_duplicate_definitions` | `Ast<MdDocument>` | Collect definitions, compare labels |
| `no_duplicate_defined_urls` | `Ast<MdDocument>` | Collect definitions, compare URLs |
| `no_undefined_references` | `Ast<MdDocument>` | Match references to definitions |
| `no_unused_definitions` | `Ast<MdDocument>` | Match definitions to references |
| `no_reference_like_url` | `Ast<MdDocument>` | URL text matching reference pattern |
| `use_sorted_definitions` | `Ast<MdDocument>` | Definition label ordering |
| `use_definitions_at_end` | `Ast<MdDocument>` | Definition position in block list |
| `use_lowercase_definition_labels` | `Ast<MdLinkDefinition>` | Label case |
| `no_definition_spacing_issues` | `Ast<MdLinkDefinition>` | Spacing around `:` |

**Utility eliminated:** `definition_utils.rs` (312 lines)

#### Group H: MDX/JSX Rules (6 rules) -> Future MDX nodes

These rules depend on MDX parsing support which is out of scope for this migration. They will continue to use text-based analysis until MDX grammar nodes are added.

| Rule | Current Status |
|------|---------------|
| `no_mdx_jsx_duplicate_attribute` | Text-based (deferred) |
| `no_mdx_jsx_void_children` | Text-based (deferred) |
| `use_sorted_mdx_jsx_attributes` | Text-based (deferred) |
| `use_consistent_mdx_jsx_quote_style` | Text-based (deferred) |
| `use_mdx_jsx_self_closing` | Text-based (deferred) |
| `use_mdx_jsx_shorthand_attribute` | Text-based (deferred) |

**Utility retained (for now):** `mdx_utils.rs` (279 lines)

#### Group I: Directive Rules (5 rules) -> Future directive nodes

Same as MDX - depends on directive parsing support.

| Rule | Current Status |
|------|---------------|
| `no_directive_duplicate_attribute` | Text-based (deferred) |
| `use_sorted_directive_attributes` | Text-based (deferred) |
| `use_consistent_directive_quote_style` | Text-based (deferred) |
| `use_directive_collapsed_attribute` | Text-based (deferred) |
| `use_directive_shortcut_attribute` | Text-based (deferred) |

**Utility retained (for now):** `directive_utils.rs` (306 lines)

#### Group J: Document-Level Rules (12 rules) -> `Ast<MdDocument>`

These rules legitimately query the document node for document-wide analysis. They don't need migration but may benefit from AST improvements.

| Rule | Notes |
|------|-------|
| `no_file_name_articles` | File path analysis, not AST |
| `no_file_name_consecutive_dashes` | File path analysis |
| `no_file_name_irregular_characters` | File path analysis |
| `no_file_name_mixed_case` | File path analysis |
| `no_file_name_outer_dashes` | File path analysis |
| `use_file_extension` | File path analysis |
| `use_final_newline` | Document text analysis |
| `no_long_lines` | Line length analysis |
| `no_consecutive_blank_lines` | Blank line detection |
| `no_trailing_hard_break_spaces` | Trailing whitespace |
| `use_consistent_linebreak_style` | CRLF/LF detection |
| `use_proper_names` | Text pattern matching |
| `use_consistent_media_style` | Cross-document style |
| `use_consistent_horizontal_rule_style` | Thematic break style |
| `no_paragraph_content_indent` | Paragraph indentation |
| `no_invalid_link_fragments` | Cross-reference validation |

**Utility retained:** `line_utils.rs` (113 lines) — general-purpose line splitting

---

## 6. Utility Deprecation Schedule

| Utility File | Lines | Eliminated In | Replaced By |
|-------------|-------|---------------|-------------|
| `list_utils.rs` | 357 | Phase 2 (list rules) | `MdBullet`/`MdOrderBullet` accessors + `MdCheckbox` |
| `inline_utils.rs` | 704 | Phase 3 (inline rules) | `MdInlineCode`/`MdInlineEmphasis`/etc. accessors |
| `blockquote_utils.rs` | 216 | Phase 2 (blockquote rules) | `MdQuote` nesting traversal |
| `definition_utils.rs` | 312 | Phase 3 (definition rules) | `MdLinkDefinition` accessors |
| `table_utils.rs` | 265 | Phase 3 (table rules) | `MdTableCell` accessors |
| `fence_utils.rs` | 156 | Phase 2 (code block rules) | `MdFencedCodeBlock` node queries |
| `heading_utils.rs` | 138 | Phase 2 (heading rules) | `MdHeader` accessors |
| `directive_utils.rs` | 306 | Deferred | Future directive grammar |
| `mdx_utils.rs` | 279 | Deferred | Future MDX grammar |
| `line_utils.rs` | 113 | Retained | General-purpose, used by document-level rules |
| **Total deletable** | **2,148** | | |
| **Total deferred** | **585** | | |
| **Total retained** | **113** | | |

---

## 7. Phased Implementation

### Phase 1: Grammar & Parser Foundation

**Goal:** Establish the new AST structure. All existing rules continue to work (they read text, so AST changes don't affect them).

**Steps:**
1. Update `markdown.ungram` with grammar changes from Section 2
2. Update `markdown_kinds_src.rs` with new node kinds
3. Run codegen (`just gen-analyzer` or targeted markdown codegen)
4. Update parser (`syntax.rs`) for:
   - Blockquote nesting (recursive block parsing)
   - List item block content (indentation-based block parsing)
   - Checkbox parsing
   - Link definition structure
   - Table cell splitting
   - Code block info string
5. Update all formatter files for new/changed nodes
6. Accept new parser spec test snapshots
7. Verify all existing tests pass

**Files changed:** ~15 files
**Estimated lines:** ~600 new/modified parser + formatter code
**Risk:** Medium — grammar changes cascade through codegen

**Verification:**
```bash
cargo test -p biome_markdown_parser      # New AST structure correct
cargo test -p biome_markdown_formatter   # Formatters handle new nodes
cargo test -p biome_markdown_analyze     # Rules still pass (text-based)
cargo test -p biome_cli                  # CLI integration passes
```

### Phase 2: Core Rule Migration (Headings, Lists, Blockquotes, Code Blocks)

**Goal:** Migrate rules that depend on the most complex shadow parser utilities.

**Steps:**
1. **Heading rules (16):** Change Query to `Ast<MdHeader>`, use `before().len()` for level, `content()` for text
2. **List rules (13 + 3 checkbox):** Change Query to `Ast<MdBullet>`/`Ast<MdBulletListItem>`, use block content traversal
3. **Blockquote rules (2):** Change Query to `Ast<MdQuote>`, traverse nested blocks
4. **Code block rules (6):** Change Query to `Ast<MdFencedCodeBlock>`, use language/meta accessors
5. Delete `list_utils.rs`, `blockquote_utils.rs`, `fence_utils.rs`, `heading_utils.rs` (867 lines)
6. Update all affected rule spec test snapshots

**Files changed:** ~40 rule files + 4 utility deletions
**Risk:** Medium — each rule must produce identical diagnostics

**Verification:** Compare diagnostic output before/after for each rule's spec tests.

### Phase 3: Inline & Definition Rule Migration

**Goal:** Migrate rules that depend on inline element and definition parsing utilities.

**Steps:**
1. **Inline rules (19):** Change Query to specific inline node types
2. **Definition rules (9):** Change Query to `Ast<MdLinkDefinition>`
3. **Table rules (7):** Change Query to `Ast<MdTable>`/`Ast<MdTableCell>`
4. Delete `inline_utils.rs`, `definition_utils.rs`, `table_utils.rs` (1,281 lines)
5. Update all affected rule spec test snapshots

**Files changed:** ~35 rule files + 3 utility deletions
**Risk:** Medium — inline parsing edge cases

### Phase 4: Cleanup & Optimization

**Goal:** Remove all remaining dead code, verify complete migration.

**Steps:**
1. Audit all remaining rules for text-based patterns
2. Remove any unused utility functions
3. Verify no rule calls `text_with_trivia().to_string()` for structure parsing
4. Update documentation
5. Run full test suite

**Files changed:** ~5 files
**Risk:** Low

### Phase 5 (Future): MDX & Directive Grammar

**Goal:** Add MDX/JSX and directive grammar nodes to enable migration of remaining 11 rules.

**Deferred** — requires MDX specification analysis and grammar design.

---

## 8. Risk Assessment

### High Risk

| Risk | Mitigation |
|------|-----------|
| Grammar changes break codegen | Run codegen immediately after grammar changes; fix compile errors before anything else |
| Blockquote nesting parser complexity | Implement Approach B (marker tracking) first; extensive spec tests |
| List nesting parser complexity | Follow CommonMark spec algorithm; test with edge cases from CommonMark spec examples |

### Medium Risk

| Risk | Mitigation |
|------|-----------|
| Rule migration produces different diagnostics | Compare before/after spec test snapshots line by line |
| Formatter breaks with new AST structure | Format-then-parse round-trip tests |
| Performance regression (deeper AST traversal) | Benchmark before/after; AST traversal should be faster than text reparsing |

### Low Risk

| Risk | Mitigation |
|------|-----------|
| Remaining utility code after migration | Audit step in Phase 4 |
| MDX/directive rules break | These rules are text-based and unchanged |
| New parser spec test failures | Accept snapshots after verifying AST correctness |

---

## Appendix A: Current vs Target Node Count

| Metric | Current | After Phase 1 |
|--------|---------|---------------|
| Grammar node types | 35 | 39 (+4 new) |
| Shadow parser utility lines | 2,856 | 2,856 (unchanged) |
| Rules using `Ast<MdDocument>` | 100 | 100 (unchanged) |

| Metric | After Phase 3 | After Phase 4 |
|--------|---------------|---------------|
| Shadow parser utility lines | 698 (mdx + directive + line) | 698 |
| Rules using `Ast<MdDocument>` | ~25 (document-level only) | ~25 |
| Lines deleted | ~2,148 | ~2,148 |

## Appendix B: mdast Naming Convention Note

Biome's naming convention differs from mdast:
- mdast `Emphasis` (single `*`/`_`) = Biome `MdInlineItalic`
- mdast `Strong` (double `**`/`__`) = Biome `MdInlineEmphasis`

This is an existing convention in the codebase and should **not** be changed during this migration to avoid unnecessary churn. Document the mapping but preserve existing names.
