# Implementation Plan: Extend Biome Markdown Lint Rules

**Status:** Completed
**Created:** 2026-02-08
**Completed:** 2026-02-09

## Summary

All 75 planned rules have been implemented, promoted to their target groups, and given test fixtures. An additional 24 rules were implemented beyond the original plan (MDX/directive support, filename validation, additional semantic checks), bringing the total to **100 rules**.

### Final Statistics

| Metric | Count |
|--------|-------|
| Total rules implemented | 100 |
| Rules from original plan | 76 (75 + renamed useHeadingIncrement) |
| Extra rules added | 24 |
| Rules with code fix actions | 62 (62%) |
| Rules with configurable options | 32 (32%) |
| Test fixture coverage | 100% |

### Rules by Group

| Group | Count |
|-------|-------|
| correctness | 17 |
| style | 75 |
| suspicious | 6 |
| a11y | 2 |

---

## Context

This biome fork had 5 markdown lint rules in nursery. The ontology at `kb/schemas/markdown-lint-rules.ttl` defined 100 canonical rules — 5 implemented, 76 planned, 19 deferred. After resolving the `noInvalidHeadingLevel`/`useHeadingIncrement` overlap (rename existing rule), this plan covered implementing **75 new rules** in 6 phases.

**Key constraint:** The markdown parser only produces `MdHeader`, `MdParagraph`, `MdThematicBreakBlock`, and `MdDocument` AST nodes. All inline elements (links, emphasis, code spans) and block structures (lists, blockquotes, code blocks) are flattened to `MdTextual`/`MdParagraph`. Rules use text-based scanning for anything beyond headings and thematic breaks.

**Existing rule patterns:**
- AST-based: Query `Ast<MdDocument>`, traverse descendants, cast to `MdHeader` (only works for headings)
- Text-based: Get raw text via `syntax().text_trimmed()`, line-by-line or byte-level scanning
- Hybrid: Find `MdParagraph` nodes, scan their text for inline patterns

---

## Phase 0: Prerequisite — Rename + Shared Utilities + Test Infrastructure [DONE]

### Rename `noInvalidHeadingLevel` → `useHeadingIncrement` [DONE]

### Shared utility modules [DONE]

Created `crates/biome_markdown_analyze/src/utils/mod.rs` with reusable scanning modules:

| Module | Purpose | Consumers |
|--------|---------|-----------|
| `line_utils` | `DocumentLines` struct, `is_blank_line()`, `leading_indent()`, line-to-`TextRange` mapping | Nearly all text-based rules |
| `fence_utils` | `FenceBlock` detection, `is_inside_code_fence()` | Code block rules, any rule that must skip fenced content |
| `heading_utils` | Text-based heading detection (incl. setext), `heading_slug()` for fragment matching | Heading rules, `noInvalidLinkFragments` |
| `inline_utils` | `find_inline_links()`, `find_emphasis_spans()`, `find_code_spans()`, `find_matching_bracket()` | Link, emphasis, code span rules |
| `table_utils` | GFM table detection: header/separator/data rows, column count, alignment | 7 table rules |
| `list_utils` | List block detection: markers, indentation, nesting, checkboxes | 12 list rules |
| `definition_utils` | Link reference definition detection: `[label]: url "title"` | 7 definition rules |
| `blockquote_utils` | Blockquote block detection via `> ` prefix | 2 blockquote rules |

### Test infrastructure [DONE]

Created `crates/biome_markdown_analyze/tests/spec_tests.rs` with `tests_macros::gen_tests!` for `tests/specs/**/*.md`, following the CSS analyzer test pattern.

---

## Phase 1: Document-Level + Heading Rules (11 rules) [DONE]

| # | Rule | Group | Fix | Options |
|---|------|-------|-----|---------|
| 1 | `useFinalNewline` | style | Safe | — |
| 2 | `noConsecutiveBlankLines` | style | Safe | `maxConsecutive: u32` |
| 3 | `noHardTabs` | style | Safe | `allowInCodeBlocks: bool` |
| 4 | `noLongLines` | style | — | `maxLength`, `allowInCodeBlocks`, `allowInTables`, `allowUrls` |
| 5 | `noTrailingHardBreakSpaces` | style | Safe | — |
| 6 | `noMissingSpaceAtxHeading` | correctness | Safe | — |
| 7 | `noMultipleSpaceAtxHeading` | style | Safe | — |
| 8 | `noHeadingTrailingPunctuation` | style | Safe | `punctuation: String` |
| 9 | `noMultipleTopLevelHeadings` | suspicious | — | — |
| 10 | `useFirstLineHeading` | style | — | `level: u8` |
| 11 | `useConsistentHorizontalRuleStyle` | style | Safe | `style: String` |

---

## Phase 2: Code Blocks + Inline Elements + Spacing (14 rules) [DONE]

| # | Rule | Group | Fix | Options |
|---|------|-------|-----|---------|
| 12 | `useConsistentCodeFenceMarker` | style | Safe | `marker: String` |
| 13 | `useConsistentCodeBlockStyle` | style | — | `style: String` |
| 14 | `noShellDollarPrompt` | style | Safe | — |
| 15 | `useBlanksAroundCodeFences` | style | Safe | — |
| 16 | `useBlanksAroundHeadings` | style | Safe | — |
| 17 | `useConsistentEmphasisMarker` | style | Safe | `marker: String` |
| 18 | `useConsistentStrongMarker` | style | Safe | `marker: String` |
| 19 | `noSpaceInEmphasis` | correctness | Safe | — |
| 20 | `noSpaceInCode` | style | Safe | — |
| 21 | `noSpaceInLinks` | style | Safe | — |
| 22 | `noInlineHtml` | style | — | `allowedElements: Vec<String>` |
| 23 | `noHeadingIndent` | style | Safe | — |
| 24 | `noHeadingContentIndent` | style | Safe | — |
| 25 | `useConsistentLinebreakStyle` | style | Safe | `style: String` |

---

## Phase 3: Links + References + Definitions (16 rules) [DONE]

| # | Rule | Group | Fix | Options |
|---|------|-------|-----|---------|
| 26 | `noBareUrls` | style | Safe | — |
| 27 | `noUndefinedReferences` | correctness | — | — |
| 28 | `noInvalidLinkFragments` | correctness | — | — |
| 29 | `useConsistentLinkStyle` | style | Safe | `style: String` |
| 30 | `useConsistentLinkTitleStyle` | style | Safe | `style: String` |
| 31 | `useConsistentMediaStyle` | style | Safe | `style: String` |
| 32 | `noShortcutReferenceImage` | style | Safe | — |
| 33 | `noShortcutReferenceLink` | style | Safe | — |
| 34 | `noUnneededFullReferenceImage` | style | Safe | — |
| 35 | `noUnneededFullReferenceLink` | style | Safe | — |
| 36 | `noDuplicateDefinitions` | correctness | — | — |
| 37 | `noDuplicateDefinedUrls` | suspicious | — | — |
| 38 | `noUnusedDefinitions` | correctness | Safe | — |
| 39 | `useLowercaseDefinitionLabels` | style | Safe | — |
| 40 | `useSortedDefinitions` | style | Safe | — |
| 41 | `noDefinitionSpacingIssues` | style | Safe | — |

---

## Phase 4: Tables + Lists (19 rules) [DONE]

### Table rules (7)

| # | Rule | Group | Fix | Options |
|---|------|-------|-----|---------|
| 42 | `noMismatchedTableColumnCount` | correctness | — | — |
| 43 | `noHiddenTableCell` | correctness | — | — |
| 44 | `useConsistentTablePipeStyle` | style | Safe | `style: String` |
| 45 | `useConsistentTableCellPadding` | style | Safe | `style: String` |
| 46 | `useBlanksAroundTables` | style | Safe | — |
| 47 | `noTableIndentation` | style | Safe | — |
| 48 | `useConsistentTablePipeAlignment` | style | Safe | — |

### List rules (12)

| # | Rule | Group | Fix | Options |
|---|------|-------|-----|---------|
| 49 | `useConsistentUnorderedListMarker` | style | Safe | `marker: String` |
| 50 | `useConsistentOrderedListMarker` | style | Safe | `delimiter: String` |
| 51 | `useConsistentListItemIndent` | style | Safe | `style: String` |
| 52 | `useConsistentListIndent` | style | Safe | — |
| 53 | `useConsistentUnorderedListIndent` | style | Safe | — |
| 54 | `noListItemBulletIndent` | style | Safe | — |
| 55 | `useConsistentListItemContentIndent` | style | Safe | — |
| 56 | `useConsistentListItemSpacing` | style | — | `style: String` |
| 57 | `useConsistentOrderedListMarkerValue` | style | Safe | `style: String` |
| 58 | `noCheckboxCharacterStyleMismatch` | style | Safe | `checked: String` |
| 59 | `noCheckboxContentIndent` | style | Safe | — |
| 60 | `useBlanksAroundLists` | style | Safe | — |

---

## Phase 5: Accessibility + Remaining Correctness (10 rules) [DONE]

| # | Rule | Group | Fix | Options |
|---|------|-------|-----|---------|
| 61 | `noMissingAltText` | a11y | — | — |
| 62 | `useDescriptiveLinkText` | a11y | — | `minimumLength: u32`, `forbiddenTexts: Vec<String>` |
| 63 | `noEmphasisAsHeading` | suspicious | — | — |
| 64 | `noDuplicateHeadingsInSection` | suspicious | — | — |
| 65 | `noReferenceLikeUrl` | suspicious | — | — |
| 66 | `noBlockquoteBrokenContinuation` | correctness | — | — |
| 67 | `noMissingSpaceClosedAtxHeading` | correctness | Safe | — |
| 68 | `noMultipleSpaceClosedAtxHeading` | style | Safe | — |
| 69 | `noLongHeadings` | style | — | `maxLength: u32` |
| 70 | `noParagraphContentIndent` | style | Safe | — |

---

## Phase 6: Final Rules (5 rules) [DONE]

| # | Rule | Group | Fix | Options |
|---|------|-------|-----|---------|
| 71 | `useConsistentHeadingStyle` | style | Safe | `style: String` |
| 72 | `useConsistentBlockquoteIndent` | style | Safe | — |
| 73 | `useBlanksBeforeBlockContent` | style | Safe | — |
| 74 | `useConsistentStrikethroughMarker` | style | Safe | `marker: String` |
| 75 | `useDefinitionsAtEnd` | style | Safe | — |

---

## Extra Rules (beyond original plan) [DONE]

24 additional rules were implemented beyond the original 75-rule plan:

### MDX/Directive support (11 rules)

| Rule | Group | Fix |
|------|-------|-----|
| `noDirectiveDuplicateAttribute` | correctness | — |
| `noMdxJsxDuplicateAttribute` | correctness | — |
| `noMdxJsxVoidChildren` | correctness | — |
| `useConsistentDirectiveQuoteStyle` | style | Safe |
| `useConsistentMdxJsxQuoteStyle` | style | Safe |
| `useDirectiveCollapsedAttribute` | style | Safe |
| `useDirectiveShortcutAttribute` | style | Safe |
| `useMdxJsxSelfClosing` | style | Safe |
| `useMdxJsxShorthandAttribute` | style | Safe |
| `useSortedDirectiveAttributes` | style | Safe |
| `useSortedMdxJsxAttributes` | style | Safe |

### Filename validation (6 rules)

| Rule | Group | Fix |
|------|-------|-----|
| `noFileNameArticles` | style | — |
| `noFileNameConsecutiveDashes` | style | — |
| `noFileNameIrregularCharacters` | style | — |
| `noFileNameMixedCase` | style | — |
| `noFileNameOuterDashes` | style | — |
| `useFileExtension` | style | — |

### Additional semantic checks (7 rules)

| Rule | Group | Fix |
|------|-------|-----|
| `noDuplicateHeadings` | suspicious | — |
| `noHeadingLikeParagraph` | correctness | — |
| `useProperNames` | style | Safe |
| `useRequiredHeadings` | style | — |

---

## Rules WITHOUT code fixes (38 rules)

These rules report diagnostics but don't offer automatic fixes. Some are inherently unfixable (require human judgment), others could potentially have fixes added:

**Could potentially add fixes:**
- `noEmptyLinks` (correctness) — could remove empty link
- `noMissingLanguage` (style) — could suggest common languages
- `noLongLines` (style) — complex wrapping logic
- `useHeadingIncrement` (correctness) — could adjust heading level

**Inherently diagnostic-only:**
- `noMissingAltText` (a11y) — needs human-written alt text
- `useDescriptiveLinkText` (a11y) — needs human-written text
- `noInvalidLinkFragments` (correctness) — ambiguous fix
- `noUndefinedReferences` (correctness) — needs human decision
- All suspicious group rules — need human review
- All filename rules — renaming files is destructive

---

## Verification

```bash
cargo build -p biome_markdown_analyze          # Compiles
cargo test -p biome_markdown_analyze           # Unit + snapshot tests pass
cargo build --bin biome                        # Full binary builds
biome lint --rule=correctness/ruleName test.md # Individual rule fires
biome check test.md                            # All rules run together
```

## Critical Files

- `crates/biome_markdown_analyze/src/lint/{a11y,correctness,style,suspicious}/*.rs` — rule implementations
- `crates/biome_markdown_analyze/src/utils/` — shared utility modules
- `crates/biome_markdown_analyze/src/lib.rs` — analyzer entry point, `MarkdownRuleAction` type
- `crates/biome_markdown_analyze/src/options.rs` — rule option type aliases
- `crates/biome_markdown_analyze/src/suppression_action.rs` — `<!-- biome-ignore -->` handling
- `crates/biome_rule_options/src/` — options struct definitions
- `crates/biome_markdown_analyze/tests/specs/` — test fixtures by group
- `kb/schemas/markdown-lint-rules.ttl` — canonical rule metadata
