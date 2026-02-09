# Plan: Gate Semantic Model Construction on Semantic Rule Activity

**Date:** 2026-02-09
**Status:** To implement
**Effort:** Medium (architectural consideration required)

## Context

The Turtle semantic model is built unconditionally at the workspace level (`crates/biome_service/src/workspace/server.rs`, lines 417-423) whenever linting or assists are enabled. Of 15 total Turtle lint rules, 8 use `Semantic<TurtleRoot>` queries and 7 use `Ast<...>` queries. The CSS language follows the same unconditional pattern.

The model construction performs a single AST preorder walk plus index building. It's not expensive for small files, but for large Turtle datasets it could be skipped entirely when only AST-based rules are active.

## Current Flow

1. `server.rs` lines 417-423: checks `is_linter_enabled() || is_assist_enabled()`
2. If true: builds `TurtleDocumentServices` with semantic model
3. File handler (`turtle.rs` lines 457-462): passes model to analyzer
4. Analyzer: only uses model if a `Semantic<TurtleRoot>` rule fires

## Proposed Change

Add a helper method to check if any Turtle semantic rules are enabled before building the model.

### Implementation

**Modify** `crates/biome_service/src/workspace/server.rs` — add semantic rule check:

```rust
if language.to_turtle_file_source().is_some()
    && (settings.is_linter_enabled() || settings.is_assist_enabled())
    && settings.has_turtle_semantic_rules_enabled()
{
    services = TurtleDocumentServices::default()
        .with_turtle_semantic_model(&any_parse.tree())
        .into();
}
```

The `has_turtle_semantic_rules_enabled()` method would check if any of these 8 rules are active:
- `noDuplicatePrefixDeclaration`
- `noDuplicateTriple`
- `noUndefinedPrefix`
- `noUndefinedSubjectReference`
- `noUnusedPrefix`
- `useGroupedSubjects`
- `usePrefixedNames`
- `useSortedPrefixes`

Plus any Turtle assist actions that use the semantic model.

### Feasibility Concern

The challenge is that rule enablement is currently determined in the file handler via `AnalyzerVisitorBuilder`, which runs *after* the semantic model is already constructed. To gate model construction, we'd need to resolve rule enablement earlier — at the workspace level.

This could be done by:
1. Extracting rule resolution logic to be callable from the workspace server
2. Or maintaining a separate, lightweight check that just queries the settings/configuration for whether any known semantic rule names are enabled

Option 2 is simpler but creates a maintenance burden (hardcoded rule name list).

## Files to Modify

| File | Change |
|------|--------|
| `crates/biome_service/src/workspace/server.rs` | Add semantic rule check before model construction |
| `crates/biome_service/src/settings.rs` (or similar) | Add `has_turtle_semantic_rules_enabled()` helper |

## Decision

Given that:
- 8 of 15 rules (53%) use the semantic model, including most recommended rules
- The optimization only helps when a user disables ALL 8 semantic rules while keeping some AST-only rules
- CSS has the same unconditional pattern and nobody has complained
- The semantic model construction is a single AST walk (already efficient)

**Recommendation: Defer this.** The benefit is marginal for the complexity introduced. Document it as a future optimization if profiling reveals it as a bottleneck for large files.
