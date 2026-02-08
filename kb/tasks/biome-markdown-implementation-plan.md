# Implementation Plan: Biome Native Markdown Linter & Formatter

## Current State (as of Feb 2026)

Markdown support in biome is actively being developed by community contributors. Here's what exists:

| Component | Status | Crate |
|-----------|--------|-------|
| Grammar | Done | `xtask/codegen/markdown.ungram` |
| Syntax | Done | `biome_markdown_syntax` |
| Factory | Done | `biome_markdown_factory` |
| Parser | ~70% CommonMark compliance | `biome_markdown_parser` |
| Formatter | Boilerplate in [PR #8962](https://github.com/biomejs/biome/pull/8962) | `biome_markdown_formatter` |
| Linter | Not started | `biome_markdown_analyze` (does not exist) |
| Service integration | Not started | No file handler in `biome_service` |

Key contributors: @jfmcdowell (parser), @tidefield (formatter), @suxin2017 (initial grammar).

## Phase 1: Parser Completion

**Goal:** Reach >95% CommonMark spec compliance + GFM extensions.

Current compliance is ~70% (455/652 CommonMark examples after [PR #8525](https://github.com/biomejs/biome/pull/8525)). [PR #8908](https://github.com/biomejs/biome/pull/8908) improved structure and conformance further.

### Remaining parser work

- [ ] Emphasis/strong edge cases (left/right flanking delimiter rules)
- [ ] Nested list indentation (CommonMark's complex lazy continuation rules)
- [ ] Link reference resolution (case-insensitive matching, multiline labels)
- [ ] Setext heading edge cases
- [ ] HTML block types 1-7 (all variants)
- [ ] GFM extensions: tables, task lists, strikethrough, autolinks, footnotes
- [ ] YAML frontmatter parsing
- [ ] Conformance test coverage for all 652 CommonMark spec examples

### Reference

The parser lives at `crates/biome_markdown_parser/` with this structure:

```
src/
  lexer/mod.rs          -- lookup-table-based lexer
  syntax/
    mod.rs
    fenced_code_block.rs
    header.rs
    html_block.rs
    inline/             -- split from monolithic inline.rs in PR #8908
      code_span.rs
      emphasis.rs
      links.rs
      html.rs
      entities.rs
    link_block.rs
    list.rs
    parse_error.rs
    quote.rs
    reference.rs
    thematic_break_block.rs
  parser.rs
  token_source.rs
```

## Phase 2: Formatter

**Goal:** Format markdown documents while preserving semantics.

[PR #8962](https://github.com/biomejs/biome/pull/8962) by @tidefield sets up the `biome_markdown_formatter` crate with full boilerplate (comments, context, CST, generated format impls for all node types). Not yet merged or wired into `biome_service`.

### Formatter tasks

- [ ] Merge PR #8962 (formatter boilerplate)
- [ ] Implement format rules for each node type:
  - [ ] Headings (ATX style normalization, spacing)
  - [ ] Paragraphs (line wrapping / reflowing)
  - [ ] Lists (indentation, marker style, spacing)
  - [ ] Code blocks (fence style, language tag)
  - [ ] Block quotes (marker style, rewrapping)
  - [ ] Thematic breaks (normalize style)
  - [ ] Links and images (inline vs reference style)
  - [ ] Tables (column alignment, padding)
  - [ ] Emphasis/strong (marker normalization)
  - [ ] Horizontal rules
- [ ] Embedded code block formatting (delegate to existing JS/TS/CSS/JSON formatters)
- [ ] YAML frontmatter preservation
- [ ] Configuration options in `biome.json` markdown section

### Reference architecture

Follow `biome_html_formatter` as the template:

```
biome_markdown_formatter/
  html/           -- (markdown/ in our case)
  comments.rs
  context.rs
  cst.rs
  generated.rs    -- auto-generated format boilerplate per node
  separated.rs
  trivia.rs
  utils/
  verbatim.rs
```

## Phase 3: Linter

**Goal:** Implement markdown lint rules as biome analyzers.

No `biome_markdown_analyze` crate exists yet. Follow `biome_html_analyze` as reference.

### Linter scaffolding

- [ ] Create `biome_markdown_analyze` crate
- [ ] Implement rule registry and suppression actions
- [ ] Wire into `biome_service` analyzer pipeline

### Lint rules (prioritized by markdownlint parity)

#### Correctness rules

- [ ] `noInvalidHeadingLevel` -- heading increment by more than one (MD001)
- [ ] `noDuplicateHeadings` -- duplicate heading text (MD024)
- [ ] `noReversedLinks` -- `(text)[url]` instead of `[text](url)` (MD011)
- [ ] `noEmptyLinks` -- empty link destinations (MD042)
- [ ] `noInvalidLinkFragments` -- anchor references must match headings (MD051)
- [ ] `noUndefinedReferences` -- reference links to undefined definitions (MD052)
- [ ] `noUnusedDefinitions` -- definitions never referenced (MD053)
- [ ] `noMissingAltText` -- images without alt text (MD045)
- [ ] `noMissingLanguage` -- fenced code blocks without language (MD040)
- [ ] `noTableColumnMismatch` -- rows with wrong number of cells (MD056)

#### Style rules

- [ ] `useConsistentHeadingStyle` -- ATX vs setext (MD003)
- [ ] `useConsistentListMarker` -- `-` vs `*` vs `+` (MD004)
- [ ] `useConsistentEmphasis` -- `*` vs `_` (MD049, MD050)
- [ ] `useConsistentCodeFenceStyle` -- backtick vs tilde (MD048)
- [ ] `useConsistentOrderedListPrefix` -- `1.` vs sequential (MD029)
- [ ] `useConsistentHorizontalRule` -- `---` vs `***` (MD035)
- [ ] `useBlanksAroundHeadings` -- blank lines before/after headings (MD022)
- [ ] `useBlanksAroundCodeBlocks` -- blank lines before/after fences (MD031)
- [ ] `useBlanksAroundLists` -- blank lines before/after lists (MD032)
- [ ] `useBlanksAroundTables` -- blank lines before/after tables (MD058)
- [ ] `useConsistentListIndent` -- consistent indent width (MD005, MD007)
- [ ] `useProperNames` -- capitalization enforcement (MD044)

#### Suspicious rules

- [ ] `noTrailingSpaces` -- whitespace at line endings (MD009)
- [ ] `noHardTabs` -- tab characters (MD010)
- [ ] `noMultipleBlanks` -- consecutive blank lines (MD012)
- [ ] `noSpaceInEmphasis` -- spaces inside emphasis markers (MD037)
- [ ] `noSpaceInCode` -- spaces inside code spans (MD038)
- [ ] `noSpaceInLinks` -- spaces inside link text (MD039)
- [ ] `noInlineHtml` -- raw HTML elements (MD033)
- [ ] `noBareUrls` -- URLs without angle brackets or link syntax (MD034)
- [ ] `noEmphasisAsHeading` -- bold text used instead of heading (MD036)

#### Accessibility rules

- [ ] `useDescriptiveLinkText` -- avoid "click here" (MD059)

### Code actions

Each autofixable rule should emit `FixKind::Safe` or `FixKind::Unsafe` suggestions:

- Heading style conversion (safe)
- List marker normalization (safe)
- Blank line insertion (safe)
- Trailing space removal (safe)
- Bare URL wrapping in angle brackets (safe)
- Emphasis marker normalization (safe)

## Phase 4: Service Integration

**Goal:** Wire markdown into biome's CLI, LSP, and VS Code extension.

### Tasks

- [ ] Create file handler at `crates/biome_service/src/file_handlers/markdown.rs`
- [ ] Register `.md` and `.mdx` file extensions
- [ ] Implement `From<MdParse> for AnyParse` for parser integration
- [ ] Add `markdown` section to `biome.json` schema
- [ ] Wire formatter into `biome format` command
- [ ] Wire linter into `biome lint` and `biome check` commands
- [ ] LSP: diagnostics, formatting, code actions for markdown files
- [ ] VS Code extension: syntax-aware formatting and linting

### Configuration schema

```json
{
  "markdown": {
    "parser": {
      "allowGfm": true,
      "allowFrontmatter": true
    },
    "formatter": {
      "enabled": true,
      "lineWidth": 80,
      "headingStyle": "atx",
      "codeBlockStyle": "fenced",
      "emphasisMarker": "*",
      "strongMarker": "**",
      "listMarker": "-",
      "formatEmbeddedCode": true
    },
    "linter": {
      "enabled": true
    }
  }
}
```

## Phase 5: Stabilization

- [ ] Conformance test suite covering all CommonMark + GFM spec examples
- [ ] Integration tests for formatter round-tripping (format -> parse -> format = idempotent)
- [ ] Performance benchmarks against remark and markdownlint
- [ ] Documentation: rule descriptions, configuration options, migration guide from markdownlint/remark
- [ ] Create "Markdown Stabilization" milestone (following the HTML precedent)

## References

- [Issue #3718 -- markdown support tracking](https://github.com/biomejs/biome/issues/3718)
- [Discussion #3816 -- Markdown Support roadmap](https://github.com/biomejs/biome/discussions/3816)
- [PR #8525 -- parser implementation (~70% compliance)](https://github.com/biomejs/biome/pull/8525)
- [PR #8908 -- parser structure + conformance improvements](https://github.com/biomejs/biome/pull/8908)
- [PR #8962 -- formatter boilerplate (open)](https://github.com/biomejs/biome/pull/8962)
- [Parser CONTRIBUTING.md](https://github.com/biomejs/biome/blob/main/crates/biome_parser/CONTRIBUTING.md)
- [HTML implementation as reference](https://github.com/biomejs/biome/tree/main/crates/biome_html_parser)
