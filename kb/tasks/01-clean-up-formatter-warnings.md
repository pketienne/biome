# Clean Up Formatter Warnings

**Status:** Completed
**Created:** 2026-02-09
**Effort:** Low
**Impact:** Code tidiness, cleaner CI output

---

## Context

`cargo build -p biome_markdown_formatter` produces 9 warnings. These are leftover from the initial generated scaffold and should be cleaned up.

## Warnings to Fix

### Unused imports (3 — auto-fixable)

1. **`src/trivia.rs:4`** — `use biome_formatter::prelude::syntax_token_cow_slice;`
   - Remove the import.

2. **`src/verbatim.rs:11`** — `use biome_rowan::{AstNode, Direction, SyntaxElement, TextRange};`
   - Remove `AstNode` from the import list, keep the others.

3. **`src/lib.rs:18`** — `pub(crate) use crate::trivia::*;`
   - Remove the glob re-export. Individual imports exist where needed.

### Dead code (5)

4. **`src/separated.rs:12`** — `struct MarkdownFormatSeparatedElementRule<N>` never constructed
5. **`src/separated.rs:37`** — `type MarkdownFormatSeparatedIter<Node, C>` never used
6. **`src/separated.rs:44`** — `trait FormatAstSeparatedListExtension` never used
   - All three in `separated.rs` — the entire separated list infrastructure is unused because markdown has no separator-delimited lists (like comma-separated). Options:
     - Delete `separated.rs` entirely and remove `mod separated;` from `lib.rs`
     - Or add `#[allow(dead_code)]` if we expect future use

7. **`src/trivia.rs:44`** — `fn on_skipped()` never used
8. **`src/trivia.rs:51`** — `fn on_removed()` never used
   - Both are convenience wrappers. Either delete them or add `#[allow(dead_code)]`.

### Unfulfilled lint expectation (1)

9. **`src/lib.rs:121`** — `#[expect(dead_code)]` on something that's no longer dead
   - Remove the `#[expect(dead_code)]` attribute.

## Approach

Run `cargo fix --lib -p biome_markdown_formatter` to auto-fix the 3 unused imports, then manually handle the dead code items. Prefer deletion over `#[allow(dead_code)]` for code that has no foreseeable use.

## Verification

```bash
cargo build -p biome_markdown_formatter 2>&1 | grep warning
# Should produce zero warnings
```
