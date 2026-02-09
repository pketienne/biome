# Turtle Remaining Potential Work

**Date:** 2026-02-09
**Status:** Reference
**Depends on:** `turtle-lint-rule-migration-summary-2026-02-09.md`

## Now Feasible (thanks to semantic model)

### 1. `noUndefinedSubjectReference` rule
- Was skipped because it needed a semantic model to track subject definitions vs references
- Now feasible — the semantic model's `triples()` and `triples_for_subject()` APIs provide the necessary data
- **Caveat:** False-positive concerns — subjects can be defined in external ontologies, so flagging "undefined" subjects may be noisy
- **Recommendation:** Consider making it opt-in (not recommended) or adding a configuration for known external namespaces

### 2. LSP features
The semantic model enables:
- **Rename prefix** — rename a namespace across all usages (declarations + references)
- **Go-to-definition** for prefixed names — jump to the `@prefix` declaration
- These would live in the service layer integration (`biome_service`)

### 3. `debug_semantic_model` capability
- The wiring exists in `crates/biome_service/src/file_handlers/turtle.rs`
- Implementation could be enhanced to output useful debug info (prefix map, triple count, duplicate detection results)

## Still Pending

### 4. Website documentation
- Pages for the 14 lint rules and 8 assist actions need to be created on the Biome website
- Rule documentation is generated from rustdoc comments in the rule implementations
- Assist documentation would need manual creation

### 5. Upstream submission
- PR against `biomejs/biome` main repository
- Requires aligning with upstream conventions, getting reviews
- Should target `main` branch (nursery rules don't follow semantic versioning)
- Per CLAUDE.md: must disclose AI assistance in the PR

## Still Infeasible

### 6. Alignment formatter options
- `alignPredicates`, `alignObjects`, `alignComments`
- Biome's single-pass IR formatter architecture doesn't support cross-line alignment
- Would require upstream changes to the formatter infrastructure
- Not worth pursuing without upstream buy-in

## Quality of Life

### 7. Additional test coverage
- More edge cases for the semantic model: blank nodes as subjects, nested collections, deeply nested blank node property lists
- Stress tests with large Turtle files

### 8. Performance optimization
- The semantic model does a single AST walk (good), but could be lazily computed if needed for large files
- Currently always built when linter/assist is enabled — could be gated on whether any semantic rules are active

## Priority Recommendation

1. **Highest impact:** Upstream submission — gets Turtle support into Biome proper
2. **Quick win:** `noUndefinedSubjectReference` rule — leverages the new semantic model
3. **Future:** LSP features — adds real IDE value but requires more service layer work
