# Integration Testing

**Status:** Planned
**Created:** 2026-02-09
**Effort:** Medium
**Impact:** Confidence that biome CLI works correctly with markdown files

---

## Context

The markdown linter (100 rules) and formatter (heading/thematic break normalization) have unit-level and snapshot-level tests, but no CLI integration tests exist. We need to verify that `biome check`, `biome lint`, and `biome format` work end-to-end with `.md` files.

## Current Integration Test State

### What exists
- **Analyzer unit tests**: `crates/biome_markdown_analyze/tests/specs/` — 100 rule fixtures per group
- **Formatter unit tests**: `crates/biome_markdown_formatter/tests/specs/` — 5 spec fixtures
- **Formatter inline tests**: `crates/biome_markdown_formatter/src/lib.rs` — 11 tests
- **Parser tests**: `crates/biome_markdown_parser/tests/` — parser snapshot tests

### What does NOT exist
- **CLI integration tests**: No markdown fixtures under `crates/biome_cli/tests/`
- **End-to-end `biome check` tests for markdown**
- **End-to-end `biome lint` tests for markdown**
- **End-to-end `biome format` tests for markdown**

### CLI test pattern (from other languages)

The CLI tests in `crates/biome_cli/tests/` use:
- Snapshot-based testing with `insta`
- Test fixtures organized by command (`commands/check/`, `commands/lint/`, `commands/format/`)
- Memory filesystem (`biome_fs::MemoryFileSystem`) for isolated tests
- The `run_cli` helper to execute commands

### File source registration

`crates/biome_markdown_syntax/src/file_source.rs` shows:
- `.md` and `.markdown` extensions are recognized
- `.mdx` extension is recognized (MDX variant)
- Language ID `"markdown"` and `"mdx"` are registered

The CLI format command (`crates/biome_cli/src/commands/format.rs`) does NOT have a `markdown_formatter` option in `FormatCommandPayload` (unlike JS, JSON, CSS, GraphQL, HTML which all have their own). This may mean markdown formatting isn't exposed via CLI flags yet.

## Proposed Testing Plan

### Step 1: Manual smoke test

Before writing automated tests, verify basic CLI functionality:

```bash
# Build the binary
cargo build --bin biome

# Create test file
cat > /tmp/test.md << 'EOF'
#Hello World

Some paragraph text.

***

##  Second heading

- list item 1
- list item 2

[link](https://example.com)
EOF

# Test lint
./target/debug/biome lint /tmp/test.md

# Test format
./target/debug/biome format /tmp/test.md

# Test check
./target/debug/biome check /tmp/test.md
```

Document what works and what doesn't.

### Step 2: Check CLI markdown configuration

Verify that `biome.json` configuration works for markdown:

```json
{
  "linter": {
    "rules": {
      "style": {
        "noHardTabs": "error"
      }
    }
  }
}
```

Test that rules can be enabled/disabled via configuration.

### Step 3: Create CLI integration tests

Follow the pattern from `crates/biome_cli/tests/commands/`:

#### Lint integration tests
- `test_lint_markdown_basic` — lint a simple markdown file, verify diagnostics
- `test_lint_markdown_rule_filter` — test `--rule` flag with markdown rules
- `test_lint_markdown_fix` — test `--fix` applies code actions

#### Format integration tests
- `test_format_markdown_basic` — format markdown, verify output
- `test_format_markdown_heading_normalization` — verify heading space normalization
- `test_format_markdown_thematic_break` — verify `***` → `---`

#### Check integration tests
- `test_check_markdown` — combined lint + format check

### Step 4: Edge case testing

Test with:
- Large markdown files (1000+ lines)
- Files with various encodings (UTF-8 BOM, no BOM)
- Files with mixed line endings (CRLF, LF)
- Files with deeply nested structures
- Empty markdown files
- Markdown files with only whitespace

### Step 5: Real-world file testing

Test against real markdown files from the biome repository itself:
- `CONTRIBUTING.md`
- `README.md`
- Various documentation files

## Verification

```bash
# Run CLI tests (if created)
cargo test -p biome_cli -- markdown

# Manual smoke test
./target/debug/biome check /tmp/test.md
```

## Dependencies

- The CLI may need a `markdown_formatter` option added to `FormatCommandPayload` (like other languages have) if markdown-specific formatter CLI options are needed.
- Verify that the service layer correctly routes `.md` files to the markdown analyzer and formatter.
