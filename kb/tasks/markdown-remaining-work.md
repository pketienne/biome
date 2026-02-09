# Markdown Linter/Formatter/Parser — Remaining Work

Created: 2026-02-09

---

## Linter — 100 rules implemented, all in nursery

**Done:**
- 100 lint rules covering all of markdownlint + remark-lint
- Text-based utility modules (fence, inline, heading, table, list, definition, blockquote, MDX JSX, directive)
- Options support for configurable rules (useFileExtension, useConsistentMdxJsxQuoteStyle, etc.)
- Markdown metadata wired into `to_analyzer_rules()` so options load correctly

**Remaining work:**
1. **Rule promotion** — All 100 rules sit in `nursery/`. After a release cycle with no bugs, they should be promoted to stable groups (correctness, style, suspicious, a11y) using `just move-rule`. The ontology already maps each rule to its target group.
2. **Code fix actions** — Nearly zero rules implement auto-fix. Most just emit diagnostics. Adding `fn action()` implementations (safe/unsafe fixes) would be a large effort but high value for rules like `useConsistentHeadingStyle`, `useSortedMdxJsxAttributes`, etc.
3. **Test coverage** — Each rule has basic valid/invalid fixtures, but edge cases are thin. More robust test fixtures would improve confidence before promotion.

---

## Formatter — Verbatim only (no real formatting)

**Done:**
- Complete skeleton with format implementations for every markdown syntax node
- All nodes route to `format_verbatim_node()` — preserves original source as-is
- Configuration plumbing (indent style, line width, line endings) exists but is unused

**Remaining work:**
4. **Actual formatting logic** — This is the biggest gap. Currently the formatter is a no-op passthrough. Real formatting would include:
   - Consistent heading style (ATX vs setext)
   - Line wrapping at configured width
   - Consistent list marker style
   - Blank line normalization
   - Code fence normalization
   - Trailing whitespace removal
5. **Tests** — Only one smoke test exists. A full formatter needs spec-level test suites.

---

## Parser — Minimal, headings + paragraphs only

**Done:**
- Parses `MdDocument`, `MdHeader` (ATX), `MdParagraph`, `MdThematicBreakBlock`
- Lexer handles basic markdown tokens
- Partial CommonMark compliance (tab handling, thematic breaks)

**Remaining work:**
6. **Structural parsing for most elements** — Lists, blockquotes, code blocks, links, emphasis, images, and inline code are all flattened to `MdParagraph`/`MdTextual`. The analyzer works around this with text-based scanning utilities, but a proper parser would:
   - Enable the formatter to do real work
   - Make lint rules more accurate (no regex false positives)
   - Support code actions that manipulate the AST
7. **GFM extensions** — Tables, task lists, strikethrough, autolinks are not parsed
8. **Setext headings** — Only partially detected

---

## Priority order

| Priority | Area | Impact |
|----------|------|--------|
| 1 | Promote stable rules out of nursery | Low effort, high credibility |
| 2 | Add code fix actions to key rules | High user value |
| 3 | Expand the parser | Unblocks real formatting + better lint accuracy |
| 4 | Implement real formatter | Requires parser improvements first |

The linter is feature-complete for rule coverage. The parser and formatter are the main areas needing substantial work.
