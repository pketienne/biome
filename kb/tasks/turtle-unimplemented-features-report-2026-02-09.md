# Turtle: Unimplemented Features Report

**Date:** 2026-02-09 (updated)
**Status:** Reference
**Source:** Review of all 28 `kb/tasks/turtle-*.md` files + codebase audit

## Current Totals

- **18 lint rules** implemented (all with spec tests)
- **8 assist actions** implemented (all with spec tests)
- **21+ formatter features** implemented (16 test fixtures)
- **8 configuration options** (4 global + 3 Turtle-specific + 1 internal)
- **Full semantic model** (events, builder, model, integration, Display impl, 24 tests)
- **Full W3C Turtle 1.1 parser/lexer**
- **95+ tests** across all crates
- **100% of P0/P1/P2 gap analysis items done**

## Lint Rules — Not Implemented (2 stretch goals)

| Rule | Description | Value |
|------|-------------|-------|
| `useRdfsLabel` | Resources should have `rdfs:label` | Low — ontology quality, very opinionated |
| `useRdfsComment` | Classes/properties should have `rdfs:comment` | Low — ontology quality, very opinionated |

~~`noMissingSemicolonBeforeDot`~~, ~~`noEmptyPrefixedName`~~, ~~`useBlanksAroundBlocks`~~ — **now implemented** (commit `581f5f2964`).

## Formatter Features — Not Implemented (2)

| Feature | Description | Value |
|---------|-------------|-------|
| Diff-optimized output mode | One triple per line without `;`/`,` grouping | Low — `expandTriples` assist already does this |
| Auto-merge repeated predicates to comma lists | Formatter automatically groups objects | Low — `mergeTriples` assist already does this |

## Formatter Features — Infeasible (3)

| Feature | Reason |
|---------|--------|
| Prefix alignment on `:` | Single-pass IR formatter has no cross-line width measurement |
| Predicate vertical alignment | Same limitation |
| Object/comment vertical alignment | Same limitation |

## Formatter Features — Deferred (2)

| Feature | Reason |
|---------|--------|
| Predicate ordering as formatter config | High complexity, requires string array serialization in config schema. Available as `sortPredicates` assist. |
| Subject ordering as formatter config | Same. Available as `sortTriples` assist. |

## Configuration Options — Not Implemented (5)

| Option | Reason |
|--------|--------|
| `alignPredicates` | Infeasible (formatter architecture) |
| `keepUnusedPrefixes` | Low value — users can suppress `noUnusedPrefix` rule instead |
| `prefixOrder` (string array) | Deferred — complex config serialization |
| `predicateOrder` (string array) | Deferred — complex config serialization |
| `useAForRdfType` (formatter option) | Covered by `useShorthandRdfType` lint rule + `convertRdfTypeToShorthand` assist |

## Infrastructure — Not Implemented (5)

| Feature | Description | Value |
|---------|-------------|-------|
| LSP: Rename prefix | Rename namespace across all usages via semantic model | High — real IDE value |
| LSP: Go-to-definition | Jump from prefixed name to `@prefix` declaration | High — real IDE value |
| LSP: Find references | From prefix declaration, find all usages | High — real IDE value |
| Lexer: DOUBLE zero fractional digits | Fix `1.E3` parsing (greedy DECIMAL match) | Low — extremely rare, formatter workaround exists |
| Semantic model gating | Gate construction on active semantic rules | Low — marginal benefit, explicitly deferred |

## Parser/Lexer — Not Implemented (3 file extensions)

| Feature | Notes |
|---------|-------|
| `.nq` (N-Quads) file extension | Listed in original plan but not yet supported |
| `.trig` (TriG) file extension | Listed in original plan but not yet supported |
| `"trig"` language ID | Listed in original plan but not yet supported |

## Testing Gaps (3)

| Gap | Notes |
|-----|-------|
| Suppression tests for `noEmptyPrefixedName` | Has spec tests but no suppression test |
| Suppression tests for `useBlanksAroundBlocks` | Has spec tests but no suppression test |
| Suppression tests for `noMissingSemicolonBeforeDot` | Has spec tests but no suppression test |

## Documentation / Process (2)

| Item | Description | Value |
|------|-------------|-------|
| Website documentation | Pages for 18 lint rules + 8 assist actions | Required for upstream |
| Upstream submission | PR to `biomejs/biome` main | Highest impact |

## Summary

| Category | Implemented | Remaining | Infeasible | Deferred |
|----------|-------------|-----------|------------|----------|
| Lint Rules | 18 | 2 (stretch) | 0 | 0 |
| Assist Actions | 8 | 0 | 0 | 0 |
| Formatter Features | 21+ | 2 | 3 | 2 |
| Config Options | 8 | 5 | 1 | 2 |
| Semantic Model | Full | 1 (gating) | 0 | 0 |
| Parser/Lexer | Full W3C 1.1 | 1 bug + 3 extensions | 0 | 0 |
| Infrastructure | Full | 5 (3 LSP + 2 other) | 0 | 0 |
| Tests | 95+ | 3 suppression tests | 0 | 0 |
| Documentation | 0 | 2 | 0 | 0 |

**Overall coverage:** ~95% of gap analysis items complete. 100% of P0/P1/P2 done. Remaining items are stretch goals, infeasible, deferred, or documentation.
