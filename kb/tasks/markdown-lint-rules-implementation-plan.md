# Implementation Plan: Extend Biome Markdown Lint Rules

**Status:** Planned
**Created:** 2026-02-08

## Context

This biome fork has 5 markdown lint rules implemented in nursery. The ontology at `kb/schemas/markdown-lint-rules.ttl` defines 100 canonical rules — 5 implemented, 76 planned, 19 deferred. After resolving the `noInvalidHeadingLevel`/`useHeadingIncrement` overlap (rename existing rule, drop `noInvalidHeadingLevel` from ontology), this plan covers implementing **75 new rules** in 6 phases.

**Key constraint:** The markdown parser only produces `MdHeader`, `MdParagraph`, `MdThematicBreakBlock`, and `MdDocument` AST nodes. All inline elements (links, emphasis, code spans) and block structures (lists, blockquotes, code blocks) are flattened to `MdTextual`/`MdParagraph`. Rules must use text-based scanning for anything beyond headings and thematic breaks.

**Existing rule patterns:**
- AST-based: Query `Ast<MdDocument>`, traverse descendants, cast to `MdHeader` (only works for headings)
- Text-based: Get raw text via `syntax().text_trimmed()`, line-by-line or byte-level scanning
- Hybrid: Find `MdParagraph` nodes, scan their text for inline patterns

---

## Phase 0: Prerequisite — Rename + Shared Utilities + Test Infrastructure

### Rename `noInvalidHeadingLevel` → `useHeadingIncrement`

The current `noInvalidHeadingLevel` implementation checks for heading level jumps (h1→h3), which is exactly what `useHeadingIncrement` describes. Rename the rule:
- Rename file: `no_invalid_heading_level.rs` → `use_heading_increment.rs`
- Update `declare_lint_rule!` name to `"useHeadingIncrement"`
- Update ontology: remove `noInvalidHeadingLevel`, mark `useHeadingIncrement` as `"implemented"`
- Update `options.rs` type alias
- Run `just gen-analyzer`

### Shared utility modules

Create `crates/biome_markdown_analyze/src/utils/mod.rs` with reusable scanning modules. Refactor existing rules to use them.

| Module | Purpose | Consumers |
|--------|---------|-----------|
| `line_utils` | `DocumentLines` struct, `is_blank_line()`, `leading_indent()`, line-to-`TextRange` mapping | Nearly all text-based rules |
| `fence_utils` | `FenceBlock` detection, `is_inside_code_fence()` — extracted from `noMissingLanguage` | Code block rules, any rule that must skip fenced content |
| `heading_utils` | Text-based heading detection (incl. setext), `heading_slug()` for fragment matching | Heading rules, `noInvalidLinkFragments` |
| `inline_utils` | `find_inline_links()`, `find_emphasis_spans()`, `find_code_spans()`, `find_matching_bracket()` — extracted from `noEmptyLinks`/`noReversedLinks` | Link, emphasis, code span rules |
| `table_utils` | GFM table detection: header/separator/data rows, column count, alignment | 7 table rules |
| `list_utils` | List block detection: markers, indentation, nesting, checkboxes | 12 list rules |
| `definition_utils` | Link reference definition detection: `[label]: url "title"` | 7 definition rules |
| `blockquote_utils` | Blockquote block detection via `> ` prefix | 2 blockquote rules |

### Test infrastructure

Create `crates/biome_markdown_analyze/tests/spec_tests.rs` with `tests_macros::gen_tests!` for `tests/specs/**/*.md`, following the CSS analyzer test pattern.

### Refactor existing rules

- `noMissingLanguage` → use `fence_utils`
- `noEmptyLinks` → use `inline_utils::find_matching_bracket`
- `noReversedLinks` → use `inline_utils::find_matching_bracket`, `looks_like_url`

---

## Phase 1: Document-Level + Heading Rules (11 rules)

Simple line scanning and `MdHeader` AST rules. High value, low complexity.

| # | Rule | Group | Approach | Fix | Options |
|---|------|-------|----------|-----|---------|
| 1 | `useFinalNewline` | style | Last char check | Safe | — |
| 2 | `noConsecutiveBlankLines` | style | Line scanning | Safe | `maxConsecutive: u32` (default 1) |
| 3 | `noHardTabs` | style | Byte scan, skip fences | Safe | `allowInCodeBlocks: bool` |
| 4 | `noLongLines` | style | Line length | — | `maxLength: u32` (80), `allowInCodeBlocks`, `allowInTables`, `allowUrls` |
| 5 | `noTrailingHardBreakSpaces` | style | Trailing space check | Safe | — |
| 6 | `noMissingSpaceAtxHeading` | correctness | AST + text | Safe | — |
| 7 | `noMultipleSpaceAtxHeading` | style | AST + text | Safe | — |
| 8 | `noHeadingTrailingPunctuation` | style | AST + text | — | `punctuation: String` (default `".,;:!?"`) |
| 9 | `noMultipleTopLevelHeadings` | suspicious | AST (`MdHeader`) | — | — |
| 10 | `useFirstLineHeading` | style | Line scanning | — | `level: u8` (default 1) |
| 11 | `useConsistentHorizontalRuleStyle` | style | AST (`MdThematicBreakBlock`) + text | Safe | `style: String` (default `"---"`) |

---

## Phase 2: Code Blocks + Inline Elements + Spacing (14 rules)

Builds on `fence_utils` and `inline_utils`. Introduces emphasis/code span detection.

| # | Rule | Group | Approach | Fix | Options |
|---|------|-------|----------|-----|---------|
| 12 | `useConsistentCodeFenceMarker` | style | `fence_utils` | Safe | `marker: "backtick"\|"tilde"` |
| 13 | `useConsistentCodeBlockStyle` | style | `fence_utils` | Unsafe | `style: "fenced"\|"indented"` |
| 14 | `noShellDollarPrompt` | style | `fence_utils` + content scan | Safe | — |
| 15 | `useBlanksAroundCodeFences` | style | `fence_utils` + `line_utils` | Safe | — |
| 16 | `useBlanksAroundHeadings` | style | `line_utils` + heading detection | Safe | — |
| 17 | `useConsistentEmphasisMarker` | style | `inline_utils` | Safe | `marker: "star"\|"underscore"\|"consistent"` |
| 18 | `useConsistentStrongMarker` | style | `inline_utils` | Safe | `marker: "star"\|"underscore"\|"consistent"` |
| 19 | `noSpaceInEmphasis` | correctness | `inline_utils` | Safe | — |
| 20 | `noSpaceInCode` | style | `inline_utils` | Safe | — |
| 21 | `noSpaceInLinks` | style | `inline_utils` | Safe | — |
| 22 | `noInlineHtml` | style | Line scan for `<tag` patterns | — | `allowedElements: Vec<String>` |
| 23 | `noHeadingIndent` | style | `line_utils` + heading detection | Safe | — |
| 24 | `noHeadingContentIndent` | style | AST + text | Safe | — |
| 25 | `useConsistentLinebreakStyle` | style | Byte scan for `\r\n` vs `\n` | Safe | `style: "lf"\|"crlf"` |

---

## Phase 3: Links + References + Definitions (16 rules)

Requires complete `inline_utils` link detection and `definition_utils`.

| # | Rule | Group | Approach | Fix | Options |
|---|------|-------|----------|-----|---------|
| 26 | `noBareUrls` | style | `inline_utils` + URL pattern scan | Safe | — |
| 27 | `noUndefinedReferences` | correctness | `inline_utils` + `definition_utils` | — | — |
| 28 | `noInvalidLinkFragments` | correctness | `inline_utils` + `heading_utils::heading_slug()` | — | — |
| 29 | `useConsistentLinkStyle` | style | `inline_utils` | Unsafe | `style: "inline"\|"reference"\|"consistent"` |
| 30 | `useConsistentLinkTitleStyle` | style | `inline_utils` | Safe | `style: "double-quote"\|"single-quote"\|"parentheses"` |
| 31 | `useConsistentMediaStyle` | style | `inline_utils` (image detection) | Unsafe | `style: "inline"\|"reference"` |
| 32 | `noShortcutReferenceImage` | style | `inline_utils` | Safe | — |
| 33 | `noShortcutReferenceLink` | style | `inline_utils` | Safe | — |
| 34 | `noUnneededFullReferenceImage` | style | `inline_utils` + `definition_utils` | Safe | — |
| 35 | `noUnneededFullReferenceLink` | style | `inline_utils` + `definition_utils` | Safe | — |
| 36 | `noDuplicateDefinitions` | correctness | `definition_utils` | — | — |
| 37 | `noDuplicateDefinedUrls` | suspicious | `definition_utils` | — | — |
| 38 | `noUnusedDefinitions` | correctness | `inline_utils` + `definition_utils` | Safe | — |
| 39 | `useLowercaseDefinitionLabels` | style | `definition_utils` | Safe | — |
| 40 | `useSortedDefinitions` | style | `definition_utils` | Unsafe | — |
| 41 | `noDefinitionSpacingIssues` | style | `definition_utils` | Safe | — |

---

## Phase 4: Tables + Lists (19 rules)

Requires `table_utils` and `list_utils`. Most complex utility work.

### Table rules (7)

| # | Rule | Group | Fix | Options |
|---|------|-------|-----|---------|
| 42 | `noMismatchedTableColumnCount` | correctness | — | — |
| 43 | `noHiddenTableCell` | correctness | — | — |
| 44 | `useConsistentTablePipeStyle` | style | Safe | `style: "leading"\|"trailing"\|"both"\|"consistent"` |
| 45 | `useConsistentTableCellPadding` | style | Safe | `style: "padded"\|"compact"\|"consistent"` |
| 46 | `useBlanksAroundTables` | style | Safe | — |
| 47 | `noTableIndentation` | style | Safe | — |
| 48 | `useConsistentTablePipeAlignment` | style | Safe | — |

### List rules (12)

| # | Rule | Group | Fix | Options |
|---|------|-------|-----|---------|
| 49 | `useConsistentUnorderedListMarker` | style | Safe | `marker: "-"\|"*"\|"+"\|"consistent"` |
| 50 | `useConsistentOrderedListMarker` | style | Safe | `delimiter: "."\|")"\|"consistent"` |
| 51 | `useConsistentListItemIndent` | style | Safe | `style: "tab"\|"space"\|"one"` |
| 52 | `useConsistentListIndent` | style | Safe | — |
| 53 | `useConsistentUnorderedListIndent` | style | Safe | — |
| 54 | `noListItemBulletIndent` | style | Safe | — |
| 55 | `useConsistentListItemContentIndent` | style | Safe | — |
| 56 | `useConsistentListItemSpacing` | style | — | `style: "compact"\|"loose"\|"consistent"` |
| 57 | `useConsistentOrderedListMarkerValue` | style | Safe | `style: "ordered"\|"one"` |
| 58 | `noCheckboxCharacterStyleMismatch` | style | Safe | `checked: "x"\|"X"\|"consistent"` |
| 59 | `noCheckboxContentIndent` | style | Safe | — |
| 60 | `useBlanksAroundLists` | style | Safe | — |

---

## Phase 5: Accessibility + Remaining Correctness (10 rules)

| # | Rule | Group | Approach | Fix | Options |
|---|------|-------|----------|-----|---------|
| 61 | `noMissingAltText` | a11y | `inline_utils` (image detection) | — | — |
| 62 | `useDescriptiveLinkText` | a11y | `inline_utils` | — | `minimumLength: u32` (1), `forbiddenTexts: Vec<String>` |
| 63 | `noEmphasisAsHeading` | suspicious | `inline_utils` + `line_utils` | — | — |
| 64 | `noDuplicateHeadingsInSection` | suspicious | AST (`MdHeader`) | — | — |
| 65 | `noReferenceLikeUrl` | suspicious | `inline_utils` | — | — |
| 66 | `noBlockquoteBrokenContinuation` | correctness | `blockquote_utils` | — | — |
| 67 | `noMissingSpaceClosedAtxHeading` | correctness | Text-based | Safe | — |
| 68 | `noMultipleSpaceClosedAtxHeading` | style | Text-based | Safe | — |
| 69 | `noLongHeadings` | style | AST + text | — | `maxLength: u32` (60) |
| 70 | `noParagraphContentIndent` | style | `line_utils` | Safe | — |

---

## Phase 6: Final Rules (5 rules)

| # | Rule | Group | Approach | Fix | Options |
|---|------|-------|----------|-----|---------|
| 71 | `useConsistentHeadingStyle` | style | `heading_utils` (needs setext detection) | Unsafe | `style: "atx"\|"setext"\|"consistent"` |
| 72 | `useConsistentBlockquoteIndent` | style | `blockquote_utils` | Safe | — |
| 73 | `useBlanksBeforeBlockContent` | style | `line_utils` (all block detectors) | Safe | — |
| 74 | `useConsistentStrikethroughMarker` | style | Inline scan for `~~` | Safe | `marker: "tilde"\|"double-tilde"\|"consistent"` |
| 75 | `useDefinitionsAtEnd` | style | `definition_utils` | — | — |

---

## Per-Rule Workflow

For each rule:

1. Create `crates/biome_markdown_analyze/src/lint/nursery/<snake_case_name>.rs`
2. Declare with `declare_lint_rule!` (version `"next"`, language `"md"`, metadata from ontology)
3. Implement `Rule` trait with `Query = Ast<MdDocument>`
4. If options needed: create options struct in `crates/biome_rule_options/src/`
5. If fixable: implement `fn action()` returning `MarkdownRuleAction`
6. Create test fixtures in `crates/biome_markdown_analyze/tests/specs/nursery/<rule_name>/`
7. Run `just gen-analyzer` to register
8. Update `crates/biome_markdown_analyze/src/options.rs` with type alias

## Group Promotion Strategy

All rules start in `nursery`. After 1 release cycle with no bugs, promote to target group via `just move-rule`. New group directories (`correctness/`, `style/`, `a11y/`, `suspicious/`) under `src/lint/` will be created as rules are promoted.

## Verification

```bash
cargo build -p biome_markdown_analyze          # Compiles
cargo test -p biome_markdown_analyze           # Unit + snapshot tests pass
cargo build --bin biome                        # Full binary builds
biome lint --rule=nursery/ruleName test.md     # Individual rule fires
biome check test.md                            # All rules run together
```

## Critical Files

- `crates/biome_markdown_analyze/src/lint/nursery/*.rs` — existing rule patterns
- `crates/biome_markdown_analyze/src/lib.rs` — analyzer entry point, `MarkdownRuleAction` type
- `crates/biome_markdown_analyze/src/options.rs` — rule option type aliases
- `crates/biome_markdown_analyze/src/suppression_action.rs` — `<!-- biome-ignore -->` handling
- `crates/biome_rule_options/src/` — options struct definitions
- `crates/biome_markdown_syntax/src/generated/nodes.rs` — available AST node types
- `crates/biome_markdown_parser/src/syntax.rs` — parser capabilities/limitations
- `kb/schemas/markdown-lint-rules.ttl` — canonical rule metadata (severity, fixKind, recommended, group)
