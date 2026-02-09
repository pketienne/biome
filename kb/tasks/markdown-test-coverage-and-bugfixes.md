# Markdown Lint Rules — Test Coverage & Bug Fixes

Created: 2026-02-09

---

## Summary

Improved test coverage for 15 markdown lint rules that had placeholder or minimal test fixtures, and fixed 3 rule-level bugs discovered during the process.

## Bug Fixes

### 1. `noSpaceInEmphasis` — Rewrote detection with paired-marker scanner

**Problem:** The rule relied on `find_emphasis_markers()` from `inline_utils.rs`, which correctly skips markers with spaces on both sides per the CommonMark spec. But `noSpaceInEmphasis` specifically needs to detect those markers (patterns like `* text *` where the user intended emphasis but spaces prevent it).

An initial attempt to modify `find_emphasis_markers` to include spaced markers caused panics in the `action()` method because the markdown lexer tokenizes `*` as a separate `STAR` token (1 byte), but the action assumed the diagnostic range was within a single token.

**Fix:** Rewrote `noSpaceInEmphasis` with its own paired-marker scanner that:
- Collects all `*`/`_` runs on each line (skipping code spans and escaped chars)
- Finds matched pairs where the opening has a trailing space and closing has a leading space
- Stores the exact position of each space character in the state
- In `action()`, finds the token containing the space (using `token_at_offset`) and trims just that character

**Files changed:**
- `crates/biome_markdown_analyze/src/lint/correctness/no_space_in_emphasis.rs`

### 2. `noUnusedDefinitions` — Definition lines counted as self-references

**Problem:** `find_reference_links()` matched `[label]` from definition lines (e.g., `[example]: url`) as shortcut references, making every definition appear "used" by its own label. This caused the rule to never report any unused definitions.

**Fix:** Collect definition line indices and skip them when scanning for references:
```rust
let definition_lines: std::collections::HashSet<usize> =
    definitions.iter().map(|d| d.line_index).collect();
// ... in the scanning loop:
if !tracker.is_inside_fence() && !definition_lines.contains(&line_idx) {
```

**Files changed:**
- `crates/biome_markdown_analyze/src/lint/correctness/no_unused_definitions.rs`

### 3. `find_emphasis_markers` — Reverted to correctly skip spaced markers

**Problem:** A previous change made `find_emphasis_markers` classify markers with spaces on both sides as "opening" markers. This broke the function's contract for other rules that depend on it (e.g., `useConsistentEmphasisMarker`, `useConsistentStrongMarker`).

**Fix:** Reverted to the original behavior: skip markers where `preceded_by_space && followed_by_space` since these are not emphasis markers per the CommonMark spec.

**Files changed:**
- `crates/biome_markdown_analyze/src/utils/inline_utils.rs`

---

## Test Fixtures Written/Enhanced (15 rules)

Each rule below had its `invalid.md` and/or `valid.md` test fixtures rewritten with proper content that triggers actual violations (invalid) or demonstrates correct usage (valid).

### Correctness (4 rules)

| Rule | What was fixed |
|------|---------------|
| `noSpaceInEmphasis` | Rewrote both invalid.md (`* spaced *`, `** spaced **`, `_ spaced _`, `__ spaced __`) and valid.md (proper emphasis, code spans, escaped markers) |
| `noUnusedDefinitions` | Rewrote invalid.md (3 unused definitions with no references) and valid.md (definitions with collapsed, full, shortcut, and image references) |
| `noHiddenTableCell` | Enhanced valid.md with actual table content (2-col, 3-col, single-col tables) |
| `noMismatchedTableColumnCount` | Enhanced valid.md with matching table content |

### Suspicious (1 rule)

| Rule | What was fixed |
|------|---------------|
| `noEmphasisAsHeading` | Rewrote invalid.md with `**bold heading**` and `__underscore heading__` on standalone lines; rewrote valid.md with proper headings and inline bold/italic |

### Style (10 rules)

| Rule | What was fixed |
|------|---------------|
| `useConsistentStrongMarker` | Rewrote both files: invalid has mixed `**` and `__`; valid uses consistent `**` |
| `useConsistentEmphasisMarker` | Rewrote both files: invalid has mixed `*` and `_`; valid uses consistent `*` |
| `noInlineHtml` | Rewrote both files: invalid has `<em>`, `<strong>`, `<br>` tags; valid uses markdown syntax |
| `useDefinitionsAtEnd` | Rewrote both files: invalid has definitions followed by content; valid has definitions at document end |
| `noShortcutReferenceImage` | Rewrote both files: invalid has `![image]` shortcut form; valid has `![image][]` collapsed form |
| `noUnneededFullReferenceImage` | Rewrote both files: invalid has `![image][image]` redundant full form; valid has `![image][]` and `![alt][different]` |
| `useConsistentMediaStyle` | Rewrote both files: invalid mixes inline `![](url)` and reference `![alt][ref]`; valid uses consistent inline style |
| `useConsistentCodeFenceMarker` | (Fixed in prior commit dbfb0ab) |
| `useConsistentOrderedListMarker` | (Fixed in prior commit dbfb0ab) |
| `useConsistentUnorderedListMarker` | (Fixed in prior commit dbfb0ab) |

---

## Commits

| Commit | Description |
|--------|-------------|
| `dbfb0abcc1` | Lexer thematic break fallback fix + 8 style rule test fixtures |
| `e53e7ef6fb` | 3 rule bug fixes + 15 rule test fixture improvements |

---

## Test Results

All 200 spec tests pass. All 81 unit tests pass. Snapshots correctly show diagnostics with proper violation detection and fix suggestions.

---

## Remaining Test Coverage Gaps

The following areas could benefit from additional edge-case test fixtures in the future:

1. **Rules with configurable options** — Most tests use default options only. Testing non-default option values (e.g., `useConsistentEmphasisMarker` with `"underscore"` instead of default `"star"`) would improve confidence.
2. **Multi-line content** — Some rules process content across multiple lines (e.g., blockquote continuation, list item content indent). More complex multi-line fixtures would exercise these code paths.
3. **Interaction between rules** — Documents that trigger multiple rules simultaneously to ensure fixes don't conflict.
4. **Unicode content** — Most fixtures use ASCII only. Unicode characters in headings, emphasis, links, and definitions could expose byte-offset bugs.
