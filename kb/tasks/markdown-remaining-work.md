# Markdown Linter/Formatter/Parser — Remaining Work

Created: 2026-02-09
Updated: 2026-02-09

---

## Linter — 100 rules implemented, promoted to stable groups

**Done:**
- 100 lint rules covering all of markdownlint + remark-lint
- Rules promoted to target groups: `a11y/` (2), `correctness/` (17), `style/` (75), `suspicious/` (6)
- 62 rules have `fn action()` code fix implementations (safe/unsafe fixes)
- 32 rules have configurable options via `options.json`
- Text-based utility modules (fence, inline, heading, table, list, definition, blockquote, MDX JSX, directive)
- Markdown metadata wired into `to_analyzer_rules()` so options load correctly
- Test fixtures exist for all 100 rules (valid + invalid = 200 spec tests)
- 81 utility unit tests across all text-based scanning modules
- All 281 tests pass

**Remaining work:**
1. **Code fix actions for remaining 38 rules** — 62/100 rules have fixes. The remaining 38 emit diagnostics only. Adding `fn action()` implementations would improve auto-fix coverage.
2. **Edge case test coverage** — Each rule has basic valid/invalid fixtures, but some edge cases are thin. More robust test fixtures would improve confidence.

---

## Formatter — Heading + thematic break formatting, spec test infrastructure

**Done:**
- ATX heading normalization: ensures exactly one space after `#` hashes
- Thematic break normalization: all styles (`***`, `___`, `- - -`, etc.) normalized to `---`
- Block list uses `join_nodes_with_hardline()` for blank line preservation between blocks
- Document formatter delegates to child formatters (no longer a single verbatim passthrough)
- Spec test infrastructure in place (follows JSON formatter pattern)
- 16 tests pass (11 inline + 5 spec fixtures)
- Zero compiler warnings (cleaned up unused imports, dead code, deleted unused `separated.rs`)
- Configuration plumbing (indent style, line width, line endings) exists

**Remaining work:**
3. **Code fence normalization** — Normalize fence markers (backticks vs tildes), ensure consistent style. AST nodes (`l_fence_token`, `r_fence_token`) are available.
4. **Trailing whitespace removal** — Remove trailing whitespace from text lines (except intentional hard breaks with `  \n`).
5. **Setext heading normalization** — Normalize underline length or convert to ATX style.
6. **Inline whitespace normalization** — Needs parser improvements to handle emphasis, links, code spans at AST level (currently flattened to `MdTextual`).
7. **24 auxiliary formatters still verbatim** — Lists, blockquotes, code blocks, inline formatting, etc. all use `format_verbatim_node()`. Expanding these requires parser improvements first.

---

## Parser — Minimal, headings + paragraphs + thematic breaks + fenced code

**Done:**
- Parses `MdDocument`, `MdHeader` (ATX), `MdParagraph`, `MdThematicBreakBlock`, `MdFencedCodeBlock`
- Lexer handles basic markdown tokens
- Partial CommonMark compliance (tab handling, thematic breaks)
- AST node types defined for lists, blockquotes, inline elements (but parser doesn't populate them)

**Remaining work:**
8. **Lists** (highest impact — 12 lint rules benefit) — Parse unordered/ordered lists, create `MdBulletListItem`/`MdOrderListItem` nodes, handle indentation and nesting.
9. **Blockquotes** (2 lint rules benefit) — Parse `>` prefixed lines, create `MdQuote` nodes, handle nesting and lazy continuation.
10. **Inline elements** (20+ lint rules benefit) — Parse code spans, emphasis, strong, links, images. Most complex change — requires proper delimiter matching per CommonMark spec.
11. **GFM extensions** — Tables, task lists, strikethrough, autolinks are not parsed.
12. **Setext headings** — Only partially detected.

---

## CLI Integration

**Done:**
- `biome lint`, `biome format`, `biome check` all work with `.md` files
- Markdown enabled by default (`MarkdownFormatterEnabled = Bool<true>`, `MarkdownLinterEnabled = Bool<true>`)
- Configuration via `biome.json` `markdown.formatter`/`markdown.linter`/`markdown.assist` sections
- File source registration for `.md`, `.markdown`, `.mdx` extensions

**Remaining work:**
13. **CLI integration tests** — No markdown-specific tests exist in `crates/biome_cli/tests/`. Tests for `biome lint`, `biome format`, `biome check` with markdown files should follow the existing pattern (memory filesystem, snapshot testing with `insta`).

---

## Priority order

| Priority | Area | Status | Impact |
|----------|------|--------|--------|
| ~~1~~ | ~~Promote rules out of nursery~~ | **Done** | ~~Low effort, high credibility~~ |
| ~~2~~ | ~~Add code fix actions to key rules~~ | **Done (62/100)** | ~~High user value~~ |
| ~~3~~ | ~~Implement basic formatter~~ | **Done** | ~~Headings + thematic breaks~~ |
| ~~4~~ | ~~Clean up formatter warnings~~ | **Done** | ~~Zero warnings~~ |
| ~~5~~ | ~~Full test suite validation~~ | **Done (305 tests pass)** | ~~Confidence~~ |
| 6 | Expand the formatter (code fences, whitespace) | Planned | Incremental formatting improvements |
| 7 | Expand the parser (lists, blockquotes, inline) | Planned | Unblocks real formatting + better lint accuracy |
| 8 | CLI integration tests | Planned | End-to-end confidence |
| 9 | Remaining 38 code fix actions | Planned | Auto-fix coverage |

The linter is feature-complete for rule coverage with 62% code fix coverage. The parser and formatter are the main areas needing substantial work.
