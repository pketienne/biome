# Full Test Suite Validation

**Status:** Completed
**Created:** 2026-02-09
**Effort:** Low
**Impact:** Confidence that all 100 rules + formatter work correctly

---

## Context

The markdown analyzer has 100 lint rules and the formatter has 16 tests. A full test suite run validates everything works together before further development. Previous test fixes (commit `e53e7ef6fb`) resolved bugs in `noSpaceInEmphasis`, `noUnusedDefinitions`, and `find_emphasis_markers`, plus fixed 15 test fixtures with placeholder content.

## Steps

### 1. Run analyzer tests

```bash
cargo test -p biome_markdown_analyze 2>&1
```

Expected: ~200 tests pass (100 rules x valid/invalid fixtures).

If failures occur:
- Check if the failure is a snapshot mismatch (run `cargo insta accept` if formatting changed)
- Check if it's a rule logic bug (fix the rule)
- Check if it's a test fixture issue (fix the fixture content)

### 2. Run formatter tests

```bash
cargo test -p biome_markdown_formatter 2>&1
```

Expected: 16 tests pass (11 inline + 5 spec).

### 3. Run parser tests

```bash
cargo test -p biome_markdown_parser 2>&1
```

### 4. Build full binary

```bash
cargo build --bin biome 2>&1
```

Expected: Builds successfully (warnings acceptable for now, see plan #1).

### 5. Smoke test with real markdown

Create a test file and run biome on it:

```bash
echo '# Hello World

Some paragraph text.

***

## Second heading

- list item 1
- list item 2

[link](https://example.com)
' > /tmp/test.md

./target/debug/biome check /tmp/test.md
./target/debug/biome lint /tmp/test.md
./target/debug/biome format /tmp/test.md
```

### 6. Document results

Record pass/fail counts, any failures found, and whether they need fixing.

## Results (2026-02-09)

### Parser tests
- **8 passed, 0 failed** (7 lexer unit tests + 1 spec test, 1 ignored quick_test)
- Note: 1 snapshot in legacy format (cosmetic only)

### Formatter tests
- **16 passed, 0 failed** (11 inline + 5 spec)
- 9 compiler warnings (tracked in plan #01)

### Analyzer tests
- **281 passed, 0 failed** (81 unit + 200 spec, 2 ignored)
- All 100 rules' valid + invalid fixtures pass

### Binary build
- Builds successfully
- 9 warnings from `biome_markdown_formatter` (tracked in plan #01)

### Smoke test (CLI)
- `biome lint test.md` — works, detects `useConsistentHorizontalRuleStyle` with code fix
- `biome format test.md` — works, detects thematic break normalization (`***` → `---`)
- `biome check test.md` — works, reports both lint and format findings
- Note: requires a `biome.json` in the project directory (markdown files outside a project are ignored)

## Success Criteria

- [x] All analyzer tests pass
- [x] All formatter tests pass
- [x] All parser tests pass
- [x] Full binary builds
- [x] `biome check` runs without crashes on sample markdown
