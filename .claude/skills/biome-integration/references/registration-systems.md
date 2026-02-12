# Biome's Three Registration Systems

Adding a language analyzer to Biome requires registering it in three independent systems. Missing any one causes bugs ranging from compile errors to silent runtime failures.

## System 1: xtask/codegen analyzer

**What it does:** Generates `registry.rs`, `lint.rs`, and per-group module files for the analyzer crate.

**Where:** `xtask/codegen/src/` — look for the analyzer codegen function.

**What to add:** Register `biome_{lang}_analyze` as a source crate so codegen discovers its rules and groups.

**If missing:** Compile error — the generated registry won't include the language's rules, and dependent code won't find the rule types.

**How to verify:** `cargo run -p xtask_codegen -- analyzer` succeeds and the generated files reference the new language's rules.

## System 2: xtask/codegen configuration

**What it does:** Generates the unified `Rules` enum in `rules.rs` that maps rule names to groups and provides enabled/disabled status.

**Where:** `xtask/codegen/src/` — look for the configuration codegen function.

**What to add:** Include the new language's analyzer groups in the configuration generation.

**If missing:** The rule won't appear in the `Rules` enum. Depending on how configuration resolution works, this may cause a compile error or a silent omission.

**How to verify:** After running `cargo run -p xtask_codegen -- configuration`, search the generated `rules.rs` for the new rule name. It should appear in the enum.

## System 3: biome_configuration_macros proc macro

**What it does:** At compile time, generates group structs (e.g., `Suspicious`, `Correctness`) that include `recommended_rules_as_filters()`. This method returns the list of rules that are enabled by default when a group is set to `recommended`.

**Where:** `crates/biome_configuration_macros/src/lib.rs` and `visitors.rs`.

**What to add:**
1. Add `biome_{lang}_analyze` as a dependency in the proc macro's `Cargo.toml`
2. Add a `biome_{lang}_analyze::visit_registry(&mut registry)` call in the `collect_lint_rules()` function (or equivalent)

**If missing:** **SILENT FAILURE.** The rule compiles. Unit tests pass (`quick_test` in the analyzer crate tests the rule in isolation). The rule appears in the `Rules` enum (System 2 handled that). But at runtime, the `Suspicious` struct's `recommended_rules_as_filters()` method doesn't include the rule because `visit_registry` was never called for the new language's analyzer. The rule is invisible to `biome lint`.

**Symptoms:**
- `biome lint test.{ext}` reports zero diagnostics for a rule you know should fire
- The rule count in `biome rage` is lower than expected
- Unit tests pass but end-to-end tests fail

**How to verify:**
1. Run `biome lint test.{ext}` on a file that should trigger the rule
2. Check that the rule appears in output
3. If not, search `biome_configuration_macros` for `visit_registry` calls — every `biome_{lang}_analyze` crate should have one

## Verification script

After adding any lint rules, verify all three systems:

```bash
# System 1: codegen knows about the analyzer
cargo run -p xtask_codegen -- analyzer

# System 2: configuration knows about the rules
cargo run -p xtask_codegen -- configuration
grep -r "ruleName" crates/biome_configuration/src/generated/

# System 3: proc macro visits the registry
grep -r "biome_{lang}_analyze" crates/biome_configuration_macros/

# End-to-end: rule actually fires
echo '<test content that triggers rule>' > /tmp/test.{ext}
cargo run -p biome_cli -- lint /tmp/test.{ext}
```

## Why this will recur

Every new language needs the same three registrations. Systems 1 and 2 are somewhat discoverable (compile errors or missing rules in generated code). System 3 is invisible until you run an end-to-end test and notice the rule doesn't fire.

**This is the strongest candidate for automated verification.** A post-codegen check should verify: for every `biome_{lang}_analyze` crate that exists, there is a corresponding `visit_registry` call in `biome_configuration_macros`. This check is mechanical and would have saved 2+ hours during YAML integration.
