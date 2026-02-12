---
name: biome-integration
description: Use when integrating a new language into Biome. Provides the 7-layer extension contract checklist, the 3 registration systems that must all be wired, language defaults divergence patterns, and reference implementation guidance. Prevents the silent registration failures and defaults bugs discovered during YAML integration.
---

# Biome Language Integration

Operational knowledge for adding a new language to Biome. Distilled from the YAML integration (first language) and the extension contract study.

## When to use

- Implementing formatter, analyzer, or service integration for a new language
- Debugging why a lint rule compiles but doesn't fire at runtime
- Checking whether all registration points are wired for a language
- Understanding which Biome defaults a language needs to override

## The 7 layers

Every language integration touches these layers in order. See `references/layer-checklist.md` for per-layer trait requirements, file paths, and reference implementation pointers.

| Layer | What | Crate pattern | Depends on |
|-------|------|---------------|------------|
| 1 | Grammar (`.ungram`) | `biome_{lang}_syntax` | Nothing |
| 2 | Syntax Factory | `biome_{lang}_factory` | Layer 1 |
| 3 | Parser | `biome_{lang}_parser` | Layers 1-2 |
| 4 | Formatter | `biome_{lang}_formatter` | Layers 1-3 |
| 5 | Analyzer | `biome_{lang}_analyze` | Layers 1-3 |
| 6 | Configuration | `biome_configuration` | Layers 4-5 |
| 7 | Service Integration | `biome_service` | Layers 4-6 |

Layers 4 and 5 are **independent** of each other and can be built in parallel. Layer 6 depends on both. Layer 7 depends on all.

## Critical: 3 registration systems

The single hardest bug during YAML integration. See `references/registration-systems.md` for full details.

**All three must be wired for lint rules to fire at runtime:**

1. **xtask/codegen analyzer** — generates `registry.rs`, `lint.rs`, group files
2. **xtask/codegen configuration** — generates the `Rules` enum with rule-to-group mapping
3. **biome_configuration_macros proc macro** — generates group structs with `recommended_rules_as_filters()`

Missing #1 or #2 → compile error (discoverable).
Missing #3 → **silent failure** (rule compiles, unit tests pass, but never fires at runtime).

**Verification command:** After any analyzer work, run `biome lint <test-file>` and confirm the rule appears in output. Unit tests are insufficient.

## Language defaults divergence

Some languages require defaults that differ from Biome's global defaults. Document these in the spec and handle them explicitly.

**Pattern:** In the `FormatLanguage` trait implementation, override defaults:
```rust
// YAML requires spaces (spec forbids tabs)
fn options(&self) -> &Self::FormatOptions {
    // Use IndentStyle::Space, not IndentStyle::default() (which is Tab)
}
```

**Common divergences to check:**
- Indent style (spaces vs tabs) — YAML requires spaces
- Indent width — some languages have conventions (2 for YAML, 4 for Python)
- Line endings — some specs mandate LF only
- Trailing newline — some specs require it
- Quote style — language-specific conventions

## JSON as reference implementation

JSON is the simplest end-to-end Biome language integration. Use it as the reference for:
- Trait implementations (what methods to implement, what to return)
- File organization (where each file goes, naming conventions)
- Test infrastructure (harness structure, fixture patterns)
- Codegen integration (how to add a language to all three registration systems)

CSS is the secondary reference for languages with embedded language support or more complex formatting requirements.
