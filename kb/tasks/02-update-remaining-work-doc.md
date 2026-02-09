# Update markdown-remaining-work.md

**Status:** Completed
**Created:** 2026-02-09
**Effort:** Low
**Impact:** Documentation accuracy

---

## Context

`kb/tasks/markdown-remaining-work.md` is outdated. It was written when:
- All 100 rules were in nursery (now promoted to target groups)
- Nearly zero rules had code fixes (now 62 have fixes)
- The formatter was verbatim-only (now has heading + thematic break formatting)
- Only one smoke test existed (now 11 inline + 5 spec tests)

## Changes Needed

### Section: Linter

Update to reflect:
- Rules promoted to `a11y/`, `correctness/`, `style/`, `suspicious/` (not nursery)
- 62 rules have `fn action()` code fix implementations
- Test fixtures exist for all 100 rules
- Remove "rule promotion" as remaining work (done)
- Remove "code fix actions" as remaining work (largely done — 62/100)
- Keep: 38 rules still lack fixes, edge case test coverage could improve

### Section: Formatter

Update to reflect:
- ATX heading normalization (space after hashes)
- Thematic break normalization (all styles → `---`)
- Block list uses `join_nodes_with_hardline` for blank line preservation
- Spec test infrastructure in place (5 fixtures, snapshot testing)
- Verbatim range bug fixed
- Remaining: code fences, lists, blockquotes, line wrapping still verbatim

### Section: Parser

No changes — still accurate (minimal, headings + paragraphs only).

### Section: Priority order

Update to reflect completed items and new priorities.

## Verification

Review the updated document for accuracy against the codebase.
