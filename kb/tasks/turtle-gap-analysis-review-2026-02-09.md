# Turtle Gap Analysis Review

## Date: 2026-02-09

Review of the original gap analysis (`turtle-gap-analysis-and-plan.md`, 2026-02-08) against the current implementation state. All P0, P1, and P2 items are complete. Only P3 (nice-to-have) items remain.

---

## Formatter Gaps

| Feature | Priority | Status |
|---------|----------|--------|
| Basic indentation and line breaks | P0 | Done |
| Prefix declaration formatting | P0 | Done |
| Triple formatting | P0 | Done |
| Predicate-object list with `;` | P0 | Done |
| Object list with `,` | P0 | Done |
| Blank node property list `[ ... ]` | P0 | Done |
| Collection `( ... )` | P0 | Done |
| Literal formatting | P0 | Done |
| Blank lines between statement groups | P1 | Done |
| `rdf:type` -> `a` shorthand | P2 | Done (lint rule + assist) |
| Quote normalization | P2 | Done |
| Escape normalization | P2 | Done |
| Literal short notation | P2 | Done |
| Prefix alignment (on `:`) | P3 | Missing — deferred (single-pass IR limitation) |
| Predicate alignment (vertical) | P3 | Missing — deferred (same reason) |
| Object alignment (vertical) | P3 | Missing — deferred (same reason) |
| First predicate on new line option | P3 | Done |
| Prefix ordering | P3 | Done (assist) |
| Predicate ordering | P3 | Missing — deferred (high complexity) |
| Subject ordering | P3 | Missing — deferred (high complexity) |
| Directive style option | P3 | Done |
| Diff-optimized output mode | P3 | Missing |
| Unused prefix removal | P3 | Done (assist) |
| Comma usage for repeated predicates | P3 | Missing |

## Linter Gaps

| Rule | Priority | Status |
|------|----------|--------|
| `noUndefinedPrefix` | P0 | Done |
| `noUnusedPrefix` | P0 | Done (with auto-fix) |
| `noDuplicatePrefixDeclaration` | P0 | Done (with auto-fix) |
| `noInvalidIri` | P1 | Done |
| `noInvalidLanguageTag` | P1 | Done |
| `noDuplicateTriple` | P1 | Done |
| `noLiteralTrimIssues` | P2 | Done (with auto-fix) |
| `useShorthandRdfType` | P2 | Done (with auto-fix) |
| `useConsistentQuotes` | P2 | Done (with auto-fix) |
| `useConsistentDirectiveStyle` | P2 | Done (with auto-fix) |
| `useSortedPrefixes` | P3 | Done |
| `useGroupedSubjects` | P3 | Done |
| `usePrefixedNames` | P3 | Done |
| `noMalformedDatatype` | P3 | Done |
| `noUndefinedSubjectReference` | P3 | Missing — skipped (high false-positive risk) |

## Configuration Gaps

| Option | Priority | Status |
|--------|----------|--------|
| `quoteStyle` | P2 | Done |
| `useAForRdfType` | P2 | Covered by lint rule, not a formatter option |
| `alignPredicates` | P3 | Missing — deferred |
| `firstPredicateInNewLine` | P3 | Done |
| `directiveStyle` | P3 | Done |
| `keepUnusedPrefixes` | P3 | Missing |
| `prefixOrder` | P3 | Missing — deferred |
| `predicateOrder` | P3 | Missing — deferred |

---

## Remaining Gaps (All P3)

### 1. Alignment options — Not feasible

`alignPredicates`, prefix alignment on `:`, object alignment. Biome's formatter is single-pass IR-based. The `align()` primitive only supports fixed-width alignment, not dynamic vertical alignment that requires measuring sibling widths.

### 2. Ordering options — Deferred (high complexity)

`prefixOrder`, `predicateOrder`, subject ordering. Requires string array serialization in the configuration schema and complex node reordering logic. Low value relative to effort.

### 3. `noUndefinedSubjectReference` — Skipped

Objects frequently reference external resources that are never defined as subjects in the same document. This rule would produce excessive false positives without a cross-file semantic model.

### 4. Diff-optimized output mode

A formatting mode that emits one triple per line without `;`/`,` grouping, optimized for clean VCS diffs. Would require a new formatter option (e.g., `outputMode: "compact" | "diff-friendly"`) and changes to predicate-object list and object list formatting.

### 5. `keepUnusedPrefixes` config option

An inverse toggle for the `noUnusedPrefix` lint rule. Low value — users can already suppress the rule per-file or disable it in configuration.

### 6. Comma usage for repeated predicates

Merge separate triples with the same subject and predicate into object lists: `ex:s ex:p ex:o1 . ex:s ex:p ex:o2 .` → `ex:s ex:p ex:o1, ex:o2 .`. Requires cross-triple analysis and node restructuring. Could be implemented as an assist action rather than a formatter feature.

---

## Coverage Summary

| Priority | Total Items | Done | Missing |
|----------|-------------|------|---------|
| P0 | 11 | 11 | 0 |
| P1 | 4 | 4 | 0 |
| P2 | 10 | 10 | 0 |
| P3 | 16 | 7 | 9 |
| **Total** | **41** | **32** | **9** |

**Overall coverage: ~78% of all items, 100% of P0-P2.**

The 9 missing items break down as: 3 architecturally infeasible (alignment), 3 deferred due to complexity (ordering), 3 implementable but low-value (diff mode, keepUnusedPrefixes, comma merging, noUndefinedSubjectReference).
