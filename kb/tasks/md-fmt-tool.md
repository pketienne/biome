# Tool: `md-fmt` — Markdown Formatter with Multi-Formatter Code Block Support

## Summary

A markdown formatting tool using remark-stringify with pluggable formatter dispatch for embedded code blocks. Formats markdown structure and delegates code block formatting to appropriate language-specific formatters (biome, shfmt, rustfmt, etc.).

## Status

- [ ] Project setup
- [ ] Core remark pipeline
- [ ] Formatter dispatch plugin
- [ ] CLI interface
- [ ] Configuration file support
- [ ] Trunk plugin definition
- [ ] Testing
- [ ] Documentation

## Architecture

```
markdown → remark-parse → mdast tree → [remark-format-code-blocks] → remark-stringify → formatted markdown
                                                  ↓
                                         formatter dispatch
                                        ┌─────────┴─────────┐
                                        ↓         ↓         ↓
                                      biome    shfmt    rustfmt ...
```

## Tasks

### 1. Project Setup

- [ ] Create package directory (`tools/md-fmt/` or `packages/md-fmt/`)
- [ ] Initialize `package.json` with dependencies:
  - `unified`
  - `remark-parse`
  - `remark-stringify`
  - `unist-util-visit`
- [ ] Configure TypeScript (optional, can use plain JS)
- [ ] Set up build/bundle if needed

### 2. Core Remark Pipeline

- [ ] Create main processor combining remark-parse and remark-stringify
- [ ] Configure remark-stringify options:
  - `bullet`: `-`
  - `listItemIndent`: `one`
  - `emphasis`: `_`
  - `strong`: `*`
  - `fence`: `` ` ``
- [ ] Handle stdin/stdout for piping
- [ ] Handle file reading/writing

### 3. Formatter Dispatch Plugin (`remark-format-code-blocks`)

- [ ] Implement `remarkFormatCodeBlocks` plugin
- [ ] Default formatter mapping:

| Language         | Formatter                                     |
| ---------------- | --------------------------------------------- |
| js, ts, jsx, tsx | `biome format --stdin-file-path=file.{ext}`   |
| json, jsonc      | `biome format --stdin-file-path=file.json`    |
| css              | `biome format --stdin-file-path=file.css`     |
| graphql          | `biome format --stdin-file-path=file.graphql` |
| sh, bash, zsh    | `shfmt`                                       |
| go               | `gofmt`                                       |
| rust             | `rustfmt`                                     |
| python, py       | `ruff format -`                               |
| yaml, yml        | `yamlfmt -`                                   |
| toml             | `taplo fmt -`                                 |
| sql              | `sql-formatter`                               |

- [ ] Graceful failure handling (skip if formatter not installed or fails)
- [ ] Timeout handling (5s default per code block)
- [ ] Support for disabling specific languages

### 4. CLI Interface

- [ ] Parse command line arguments:
  - `md-fmt [files...]` — format files in place
  - `md-fmt --check [files...]` — check if formatted (exit 1 if not)
  - `md-fmt --stdin` — read from stdin, write to stdout
  - `md-fmt --config <path>` — specify config file
- [ ] Glob pattern support for file arguments
- [ ] Exit codes:
  - `0` — success (or all files formatted in check mode)
  - `1` — files need formatting (check mode) or error

### 5. Configuration File Support

- [ ] Look for `.md-fmt.json` in current directory or parents
- [ ] Configuration schema:

```json
{
  "formatters": {
    "python": { "cmd": "black -", "stdin": true }
  },
  "disabled": ["sql"],
  "remarkStringify": {
    "bullet": "-",
    "emphasis": "_",
    "listItemIndent": "one"
  }
}
```

- [ ] Allow overriding default formatters
- [ ] Allow disabling formatters by language
- [ ] Allow customizing remark-stringify options

### 6. Trunk Plugin Definition

- [ ] Add linter definition to `trunk/plugin.yaml`:

```yaml
lint:
    definitions:
        - name: md-fmt
          files: [markdown]
          commands:
              - name: format
                output: rewrite
                run: md-fmt ${target}
                success_codes: [0]
              - name: lint
                output: pass_fail
                run: md-fmt --check ${target}
                success_codes: [0, 1]
    enabled:
        - md-fmt
```

- [ ] Test integration with `trunk fmt`
- [ ] Test integration with `trunk check`

### 7. Testing

- [ ] Unit tests for formatter dispatch
- [ ] Unit tests for config loading
- [ ] Integration tests with sample markdown files
- [ ] Test graceful handling of missing formatters
- [ ] Test with various code block languages
- [ ] Test stdin/stdout mode
- [ ] Test check mode

### 8. Documentation

- [ ] README with usage examples
- [ ] Document configuration options
- [ ] Document default formatter mapping
- [ ] Add to ADR-002 implementation notes

## Dependencies

**Runtime:**

- Node.js (for unified/remark ecosystem)
- Optional formatters: biome, shfmt, gofmt, rustfmt, ruff, yamlfmt, taplo, sql-formatter

**Development:**

- unified, remark-parse, remark-stringify, unist-util-visit

## Open Questions

- [ ] Package location: `tools/md-fmt/` vs `packages/md-fmt/` vs standalone repo?
- [ ] Publish to npm or keep as local tool?
- [ ] Support parallel formatting of code blocks for performance?
- [ ] Cache formatter availability checks?

## Related

- [ADR-002: Markdown Formatting Strategy](../../architecture/ADR-002-markdown-formatting.md) — Decision record
- [remark-stringify options](https://github.com/remarkjs/remark/tree/main/packages/remark-stringify)
- [unified ecosystem](https://unifiedjs.com/)
- [mdast specification](https://github.com/syntax-tree/mdast)
