# Markdown: Rule Promotion, Code Fixes, Tests, and Formatter

Created: 2026-02-09

## Context

All 100 markdown lint rules are implemented but sit in the `nursery` group. None have auto-fix actions. The formatter is verbatim-only (preserves source). This plan covers items 1-5 from `kb/tasks/markdown-remaining-work.md`:

1. Promote 100 rules from nursery to stable groups
2. Add code fix actions to 59 rules that have `fixKind != "none"` in the ontology
3. Improve test coverage with edge-case fixtures
4. Implement real markdown formatting logic
5. Add formatter tests

## Phase 1: Rule Promotion (100 rules)

### Approach
Use `just move-rule <ruleName> <group>` for each rule. This command:
- Moves the `.rs` file from `nursery/` to `<group>/`
- Moves the test directory from `tests/specs/nursery/<RuleName>/` to `tests/specs/<group>/<RuleName>/`
- Updates `categories.rs` with the new path
- Runs `just gen-analyzer` to regenerate registration

### Script
Write a bash script that calls `just move-rule` for each rule based on the ontology mapping. The 100 rules map to:

**Correctness (17 rules):** noBlockquoteBrokenContinuation, noDuplicateDefinitions, noDuplicateDirectiveAttributeName, noDuplicateMdxJsxAttributeName, noEmptyLinks, noHeadingLikeParagraph, noHiddenTableCell, noInvalidLinkFragments, noMdxJsxVoidChildren, noMismatchedTableColumnCount, noMissingSpaceAtxHeading, noMissingSpaceClosedAtxHeading, noReversedLinks, noSpaceInEmphasis, noUndefinedReferences, noUnusedDefinitions, useHeadingIncrement

**Style (75 rules):** noBareUrls, noCheckboxCharacterStyleMismatch, noCheckboxContentIndent, noConsecutiveBlankLines, noDefinitionSpacingIssues, noFileNameArticles, noFileNameConsecutiveDashes, noFileNameIrregularCharacters, noFileNameMixedCase, noFileNameOuterDashes, noHardTabs, noHeadingContentIndent, noHeadingIndent, noHeadingTrailingPunctuation, noInlineHtml, noListItemBulletIndent, noLongHeadings, noLongLines, noMissingLanguage, noMultipleSpaceAtxHeading, noMultipleSpaceClosedAtxHeading, noParagraphContentIndent, noShellDollarPrompt, noShortcutReferenceImage, noShortcutReferenceLink, noSpaceInCode, noSpaceInLinks, noTableIndentation, noTrailingHardBreakSpaces, noUnneededFullReferenceImage, noUnneededFullReferenceLink, useBlanksAroundCodeFences, useBlanksAroundHeadings, useBlanksAroundLists, useBlanksAroundTables, useBlanksBeforeBlockContent, useDirectiveCollapsedAttribute, useConsistentBlockquoteIndent, useConsistentCodeBlockStyle, useConsistentCodeFenceMarker, useConsistentDirectiveQuoteStyle, useConsistentEmphasisMarker, useFileExtension, useConsistentHeadingStyle, useConsistentHorizontalRuleStyle, useConsistentLinebreakStyle, useConsistentLinkStyle, useConsistentLinkTitleStyle, useConsistentListIndent, useConsistentListItemContentIndent, useConsistentListItemIndent, useConsistentListItemSpacing, useConsistentMdxJsxQuoteStyle, useConsistentMediaStyle, useConsistentOrderedListMarker, useConsistentOrderedListMarkerValue, useConsistentStrikethroughMarker, useConsistentStrongMarker, useConsistentTableCellPadding, useConsistentTablePipeAlignment, useConsistentTablePipeStyle, useConsistentUnorderedListIndent, useConsistentUnorderedListMarker, useDefinitionsAtEnd, useFinalNewline, useFirstLineHeading, useLowercaseDefinitionLabels, useMdxJsxSelfClosing, useProperNames, useRequiredHeadings, useDirectiveShortcutAttribute, useMdxJsxShorthandAttribute, useSortedDefinitions, useSortedDirectiveAttributes, useSortedMdxJsxAttributes

**Suspicious (6 rules):** noDuplicateDefinedUrls, noDuplicateHeadings, noDuplicateHeadingsInSection, noEmphasisAsHeading, noMultipleTopLevelHeadings, noReferenceLikeUrl

**A11y (2 rules):** noMissingAltText, useDescriptiveLinkText

### Important Note on Rule Names
Some rules in the ontology have different `biomeName` values than the actual Rust implementation names. The `just move-rule` command uses the actual Rust rule names (the `name` field in `declare_lint_rule!`). Known differences:
- Ontology `useConsistentFileExtension` = code `useFileExtension`
- Ontology `useCollapsedDirectiveAttribute` = code `useDirectiveCollapsedAttribute`
- Ontology `useShortcutDirectiveAttribute` = code `useDirectiveShortcutAttribute`
- Ontology `noDuplicateDirectiveAttributeName` = code `noDirectiveDuplicateAttribute`
- Ontology `useMdxJsxSelfClose` = code `useMdxJsxSelfClosing`
- Ontology `useShorthandMdxJsxAttribute` = code `useMdxJsxShorthandAttribute`
- Ontology `noDuplicateMdxJsxAttributeName` = code `noMdxJsxDuplicateAttribute`

Use the **code names** (from `declare_lint_rule!`) when calling `just move-rule`.

### Steps
1. Generate a shell script with 100 `just move-rule` calls
2. More efficient: call `cargo run -p xtask_codegen -- move-rule` 100 times, then `cargo run -p xtask_codegen -- analyzer` once at the end
3. Run `cargo test -p biome_markdown_analyze` to verify
4. Build: `cargo build --bin biome`

## Phase 2: Code Fix Actions (59 rules with fixKind != "none")

### Approach
Markdown rules use text-based scanning (not AST manipulation) because the parser flattens most content to `MdParagraph`/`MdTextual`. For code fixes, we use **token replacement**: replace the text content of syntax tokens with corrected versions.

### Pattern for text-based fixes
```rust
fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MdRuleAction> {
    let node = ctx.query();
    let mut mutation = ctx.root().begin();
    // Find the token containing the issue
    let token = node.syntax().first_token()?;
    // Build corrected text
    let new_text = /* corrected version */;
    let new_token = make_token(token.kind(), &new_text);
    mutation.replace_token_transfer_trivia(token, new_token);
    Some(RuleAction::new(
        ctx.metadata().action_category(ctx.category(), ctx.group()),
        ctx.metadata().applicability(),
        markup! { "Fix description" },
        mutation,
    ))
}
```

### Prioritized fix batches (by complexity)

**Batch A — Simple token fixes (25 rules):** Rules where the fix is trivial text manipulation on a single token.
- `noTrailingHardBreakSpaces` → trim trailing spaces
- `noHardTabs` → replace tabs with spaces
- `noConsecutiveBlankLines` → collapse to single blank line
- `useFinalNewline` → append newline
- `noHeadingTrailingPunctuation` → remove trailing punct from heading
- `noHeadingIndent` → remove leading whitespace before heading
- `noHeadingContentIndent` → normalize space after `#`
- `noMultipleSpaceAtxHeading` → collapse multiple spaces after `#`
- `noMultipleSpaceClosedAtxHeading` → collapse spaces before closing `#`
- `noMissingSpaceAtxHeading` → insert space after `#`
- `noMissingSpaceClosedAtxHeading` → insert space before closing `#`
- `noSpaceInCode` → trim spaces inside backticks
- `noSpaceInLinks` → trim spaces in link text
- `noSpaceInEmphasis` → trim spaces inside emphasis markers
- `noParagraphContentIndent` → remove leading indent from paragraphs
- `noListItemBulletIndent` → remove indent before list markers
- `noTableIndentation` → remove indent before table pipes
- `noShellDollarPrompt` → remove `$ ` prefix from code blocks
- `noDefinitionSpacingIssues` → normalize definition spacing
- `noCheckboxContentIndent` → normalize checkbox indentation
- `useLowercaseDefinitionLabels` → lowercase the label
- `useProperNames` → replace with proper capitalization
- `noReversedLinks` → swap `(text)[url]` to `[text](url)`
- `useConsistentLinebreakStyle` → normalize line endings
- `noUnusedDefinitions` → remove unused definition lines

**Batch B — Style normalization (20 rules):** Rules requiring marker/style replacement.
- `useConsistentHeadingStyle` (unsafe) → convert between ATX/setext
- `useConsistentEmphasisMarker` → swap `_` ↔ `*`
- `useConsistentStrongMarker` → swap `__` ↔ `**`
- `useConsistentCodeFenceMarker` → swap `` ` `` ↔ `~`
- `useConsistentOrderedListMarker` → change `.` ↔ `)`
- `useConsistentUnorderedListMarker` → change `-` ↔ `*` ↔ `+`
- `useConsistentHorizontalRuleStyle` → normalize `---`/`***`/`___`
- `useConsistentBlockquoteIndent` → normalize `> ` indent
- `useConsistentLinkTitleStyle` → swap `"` ↔ `'` ↔ `()`
- `useBlanksAroundHeadings` → insert blank lines
- `useBlanksAroundCodeFences` → insert blank lines
- `useBlanksAroundLists` → insert blank lines
- `useBlanksAroundTables` → insert blank lines
- `useBlanksBeforeBlockContent` → insert blank line
- `useConsistentListIndent` → normalize indentation
- `useConsistentListItemIndent` → normalize item indent
- `useConsistentListItemContentIndent` → normalize content indent
- `useConsistentUnorderedListIndent` → normalize indent
- `useConsistentOrderedListMarkerValue` → renumber list items
- `useConsistentTablePipeAlignment` → align pipes

**Batch C — Reference/shortcut fixes (6 rules):**
- `noShortcutReferenceImage` → expand `![alt]` to `![alt][]`
- `noShortcutReferenceLink` → expand `[text]` to `[text][]`
- `noUnneededFullReferenceImage` → collapse `![alt][alt]` to `![alt]`
- `noUnneededFullReferenceLink` → collapse `[text][text]` to `[text]`
- `useConsistentLinkStyle` → convert between inline/reference
- `useConsistentMediaStyle` → normalize image syntax

**Batch D — MDX/Directive fixes (8 rules):**
- `useMdxJsxSelfClosing` → convert `<C></C>` to `<C />`
- `useMdxJsxShorthandAttribute` → convert `prop={true}` to `prop`
- `useConsistentMdxJsxQuoteStyle` → normalize quotes
- `useSortedMdxJsxAttributes` (unsafe) → reorder attributes
- `useDirectiveCollapsedAttribute` → `.class` shorthand
- `useDirectiveShortcutAttribute` → `#id` shorthand
- `useConsistentDirectiveQuoteStyle` → normalize quotes
- `useSortedDirectiveAttributes` (unsafe) → reorder attributes

**Batch E — Unsafe/complex sorts (3 rules):**
- `useDefinitionsAtEnd` (unsafe) → move definitions to end
- `useSortedDefinitions` (unsafe) → reorder definitions
- `noBareUrls` → wrap bare URLs in `<>`

### Implementation steps per rule
1. Add `fix_kind: FixKind::Safe` (or `Unsafe`) to `declare_lint_rule!`
2. Ensure `State` captures enough info for the fix (position, replacement text)
3. Implement `fn action()` method
4. Add fix test cases to existing test fixtures
5. Accept updated snapshots

## Phase 3: Test Coverage

### Approach
Add edge-case test fixtures alongside the existing valid/invalid files. For each rule:
- Add more invalid cases covering boundary conditions
- Add more valid cases covering tricky non-violations
- For rules with fixes, test fixtures automatically validate the fix output in snapshots

### Priority
Focus on rules being promoted (Phase 1) and rules getting fixes (Phase 2). The snapshot tests from Phase 2 will inherently improve coverage.

## Phase 4: Real Markdown Formatting

### Constraint
The parser only produces structured AST for: `MdDocument`, `MdHeader`, `MdParagraph`, `MdTextual`, `MdThematicBreakBlock`. Everything else is flattened. So the formatter can only format what the parser gives it.

### Feasible formatting operations (no parser changes needed)
1. **Document** — Ensure final newline, normalize consecutive blank lines
2. **Header (ATX)** — Normalize space after `#`, remove trailing `#`
3. **Paragraph** — Preserve verbatim (no line wrapping without understanding inline structure)
4. **Thematic break** — Normalize to consistent style (e.g., `---`)
5. **All other nodes** — Keep verbatim until parser is improved

### Node formatters to implement
- `FormatMdDocument` → join blocks with blank lines, ensure trailing newline
- `FormatMdHeader` → format hash + space + content
- `FormatMdThematicBreakBlock` → emit consistent marker (e.g. `---`)
- `FormatMdBlockList` → join children with appropriate blank lines
- Everything else → keep `format_verbatim_node()` for now

### Files to modify
- `crates/biome_markdown_formatter/src/md/auxiliary/document.rs`
- `crates/biome_markdown_formatter/src/md/auxiliary/header.rs`
- `crates/biome_markdown_formatter/src/md/auxiliary/thematic_break_block.rs`
- `crates/biome_markdown_formatter/src/md/lists/block_list.rs`

## Phase 5: Formatter Tests

### Approach
Follow the JSON formatter test pattern:
- Create `crates/biome_markdown_formatter/tests/specs/` with test fixtures
- Each fixture: input markdown + expected output
- Use snapshot testing with `insta`

### Test cases
- Headings: various styles, spacing
- Blank line normalization
- Thematic break normalization
- Final newline enforcement
- Mixed content preservation (verbatim sections stay unchanged)

## Execution Order

1. **Phase 1** first — move all 100 rules to stable groups
2. **Phase 2 Batch A** — simple token fixes (25 rules, highest value)
3. **Phase 4** — formatter basics (small scope, independent of Phase 2)
4. **Phase 5** — formatter tests (validates Phase 4)
5. **Phase 2 Batches B-E** — remaining fixes (34 rules)
6. **Phase 3** — additional test coverage throughout

## Verification

```bash
# After Phase 1
cargo test -p biome_markdown_analyze
cargo build --bin biome

# After Phase 2 (each batch)
cargo test -p biome_markdown_analyze
cargo insta accept  # if snapshots changed

# After Phase 4-5
cargo test -p biome_markdown_formatter
cargo build --bin biome
```
