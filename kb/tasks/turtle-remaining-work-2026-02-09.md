# Turtle Remaining Potential Work

**Date:** 2026-02-09
**Status:** Reference
**Depends on:** `turtle-lint-rule-migration-summary-2026-02-09.md`

## Completed Since Last Update

### ~~1. `noUndefinedSubjectReference` rule~~ — DONE
- Implemented as Option C: Info severity, not recommended, with `allowedPrefixes` option
- Built-in allowed prefixes: rdf:, rdfs:, owl:, xsd:, dc:, dcterms:, skos:, foaf:, schema:, sh:, prov:, dcat:
- Only checks prefixed-name objects; skips literals, blank nodes, and full IRIs
- Commit: `c59ec7a62f`

### ~~2. `xsd:double` literal short notation~~ — DONE
- Completed the formatter's literal short notation support (was missing `xsd:double`)
- Converts e.g. `"4.2E9"^^xsd:double` → `4.2E9`, validates against Turtle DOUBLE grammar
- All 4 XSD shorthand types now supported: boolean, integer, decimal, double
- Commit: `80531b53bc`

## Remaining Work

### 1. LSP features
The semantic model enables:
- **Rename prefix** — rename a namespace across all usages (declarations + references)
- **Go-to-definition** for prefixed names — jump to the `@prefix` declaration
- These would live in the service layer integration (`biome_service`)

### 2. `debug_semantic_model` capability
- The wiring exists in `crates/biome_service/src/file_handlers/turtle.rs`
- Implementation could be enhanced to output useful debug info (prefix map, triple count, duplicate detection results)

### 3. Website documentation
- Pages for the 15 lint rules and 8 assist actions need to be created on the Biome website
- Rule documentation is generated from rustdoc comments in the rule implementations
- Assist documentation would need manual creation

### 4. Upstream submission
- PR against `biomejs/biome` main repository
- Requires aligning with upstream conventions, getting reviews
- Should target `main` branch (nursery rules don't follow semantic versioning)
- Per CLAUDE.md: must disclose AI assistance in the PR

## Still Infeasible

### 5. Alignment formatter options
- `alignPredicates`, `alignObjects`, `alignComments`
- Biome's single-pass IR formatter architecture doesn't support cross-line alignment
- Would require upstream changes to the formatter infrastructure
- Not worth pursuing without upstream buy-in

## Quality of Life

### 6. Additional test coverage
- More edge cases for the semantic model: blank nodes as subjects, nested collections, deeply nested blank node property lists
- Stress tests with large Turtle files

### 7. Performance optimization
- The semantic model does a single AST walk (good), but could be lazily computed if needed for large files
- Currently always built when linter/assist is enabled — could be gated on whether any semantic rules are active

## Priority Recommendation

1. **Highest impact:** Upstream submission — gets Turtle support into Biome proper
2. **Future:** LSP features — adds real IDE value but requires more service layer work
