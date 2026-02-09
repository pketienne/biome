# Plan 5: Documentation

## Status: PENDING

## Context

The YAML implementation lacks user-facing documentation. This includes rule documentation (inline rustdoc), configuration guides, and migration aids.

## Scope

### 5A. Lint Rule Documentation
Each of the 23 lint rules needs rustdoc documentation with:
- Description of what the rule checks
- Example of invalid code
- Example of valid code
- Any configuration options

Rules are in `crates/biome_yaml_analyze/src/lint/nursery/`.

### 5B. Configuration Guide
Document how to configure YAML support in `biome.json`:
- Enable/disable formatter and linter
- Set indent style and width
- Configure specific rules

### 5C. Migration Guide from yamllint
Map common yamllint rules to their Biome equivalents to help users migrate.

## Files
- Rule files in `crates/biome_yaml_analyze/src/lint/nursery/*.rs` (inline docs)
- Website docs (separate PR to biomejs/website repo)

## Verification
- `cargo doc -p biome_yaml_analyze` builds without warnings
- Rule docs render correctly in generated documentation
