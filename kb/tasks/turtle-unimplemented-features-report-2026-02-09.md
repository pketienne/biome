# Turtle: Unimplemented Features Report

**Date:** 2026-02-09
**Status:** Reference
**Source:** Review of all 27 `kb/tasks/turtle-*.md` files

## Lint Rules — Not Implemented (4 unique)

| Rule | Description | Value |
|------|-------------|-------|
| `noMissingSemicolonBeforeDot` | Detect likely missing `;` separator in predicate-object lists | Medium — catches a common editing mistake |
| `noEmptyPrefixedName` | Flag bare prefix (e.g., `:`) without local name where likely unintentional | Low — niche |
| `useBlanksAroundBlocks` | Enforce blank lines between subject blocks | Low — formatter already handles this |
| `useRdfsLabel` / `useRdfsComment` | Resources should have `rdfs:label` / classes should have `rdfs:comment` | Low — ontology quality, very opinionated |

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
| Predicate ordering | High complexity, requires array config serialization |
| Subject ordering | Same |

## Infrastructure — Not Implemented (5)

| Feature | Description | Value |
|---------|-------------|-------|
| LSP: Rename prefix | Rename namespace across all usages | High — real IDE value |
| LSP: Go-to-definition | Jump from prefixed name to `@prefix` declaration | High — real IDE value |
| LSP: Find references | From prefix declaration, find all usages | High — real IDE value |
| Lexer: DOUBLE zero fractional digits | Fix `1.E3` parsing | Low — extremely rare, workaround exists |
| Semantic model gating | Gate construction on active semantic rules | Low — marginal benefit |

## Documentation / Process (2)

| Item | Description | Value |
|------|-------------|-------|
| Website documentation | Pages for 15 lint rules + 8 assist actions | Required for upstream |
| Upstream submission | PR to `biomejs/biome` main | Highest impact |

## Summary

- **Implemented:** 15 lint rules, 8 assist actions, 17 formatter features, 7 config options, full infrastructure
- **Not implemented:** 4 lint rules, 2 formatter features, 5 infrastructure items, 2 documentation/process items
- **Infeasible:** 3 formatter features (alignment — blocked by Biome's IR architecture)
- **Deferred:** 2 formatter features (ordering — high complexity)
- **Overall coverage:** 90% of gap analysis items complete; 100% of P0/P1/P2 items done
