# Biome Native Markdown Linter & Formatter

## Summary

What would be necessary to create a native markdown linter and formatter within biome. Biome's markdown AST parser is already in progress ([issue #3718](https://github.com/biomejs/biome/issues/3718), [discussion #3816](https://github.com/biomejs/biome/discussions/3816)), but the formatter and linter are not yet implemented.

## Status

Tracking upstream progress. Not actively being contributed to.

## Parser (in progress upstream)

1. **Grammar file** -- `markdown.ungram` defining the CommonMark AST (paragraphs, headings, lists, code blocks, links, tables, frontmatter, etc.)
2. **Codegen** -- Generate `biome_markdown_syntax` and `biome_markdown_factory` crates
3. **Lexer** -- Markdown lexing is notoriously context-sensitive (indentation-based lists, link reference resolution, emphasis parsing with left/right flanking rules). This is the hardest part.
4. **Token source** -- Lookahead and checkpoint support for ambiguous constructs
5. **Parser** -- Parse rules for every CommonMark + GFM construct (tables, strikethrough, task lists, autolinks)

## Formatter

6. **`biome_markdown_formatter` crate** -- Implement biome's `FormatLanguage` trait
7. **Format rules per node type** -- How to print each AST node (heading spacing, list indentation, code fence style, link wrapping, table alignment)
8. **Printer integration** -- Biome uses an IR-based printer (similar to Prettier's); each node emits formatting IR tokens

## Linter

9. **`biome_markdown_analyze` crate** -- Implement lint rules as analyzers
10. **Individual rules** -- Each rule (heading increment, no duplicate headings, blank lines around blocks, etc.) is a separate analyzer visiting specific AST nodes
11. **Rule categories** -- Organize under `correctness`, `style`, `suspicious`, etc. per biome's taxonomy
12. **Code actions/fixes** -- Autofixable rules need to emit fix suggestions

## Supporting Infrastructure

13. **File handling** -- Register `.md`/`.mdx` extensions in biome's file resolver
14. **Configuration** -- Add markdown section to `biome.json` schema for rule config
15. **LSP support** -- Wire up diagnostics, formatting, and code actions for the VS Code extension
16. **Embedded language support** -- Format code blocks by delegating to existing JS/TS/CSS/JSON formatters (biome already does this for other languages)

## What Makes It Hard

- **Markdown parsing is deceptively complex** -- CommonMark spec has ~300 edge cases. Emphasis/strong parsing, lazy continuation lines, link reference resolution, and nested list indentation are all tricky.
- **No existing Rust CommonMark parser fits biome's architecture** -- Biome needs its own CST (concrete syntax tree) that preserves every byte of the original source, including whitespace and trivia. Off-the-shelf parsers like `pulldown-cmark` or `markdown-rs` produce ASTs that discard formatting details.
- **GFM extensions** -- Tables, task lists, strikethrough, autolinks, footnotes all need to be handled on top of CommonMark.

## Rule Comparison: remark-lint vs markdownlint

For reference, the two main existing markdown linters have:

- **remark-lint**: 80 rules across 12 categories (headings, lists, links, code, tables, emphasis, definitions, file names, MDX JSX, directives, etc.). Plugin architecture -- each rule is a separate npm package.
- **markdownlint** (cli2): 53 built-in rules covering headings, lists, links, code, whitespace, emphasis, tables, accessibility. Single self-contained tool.

A biome implementation would likely target parity with markdownlint's rule set first, since those rules are self-contained and well-documented.

## References

- [Issue #3718 -- markdown support](https://github.com/biomejs/biome/issues/3718)
- [Discussion #3816 -- Markdown Support roadmap](https://github.com/biomejs/biome/discussions/3816)
- [Discussion #923 -- markdown formatting request](https://github.com/biomejs/biome/discussions/923)
- [Parser CONTRIBUTING.md](https://github.com/biomejs/biome/blob/main/crates/biome_parser/CONTRIBUTING.md)
