# YAML Implementation — Remaining Work Priorities

## Date: 2026-02-09

## Current State
- Plans 1-10 all COMPLETE
- 181 tests passing, 0 failures, 0 warnings
- Parser, formatter, linter, CLI integration, config options, overrides all working

## Remaining Items (Ranked by Impact)

### 1. Default YAML indent style to spaces
**Impact: High | Effort: Small**

The YAML spec says tabs MUST NOT be used for indentation, yet Biome defaults to tabs. Changing the YAML-specific default to spaces (indent_width: 2) would make `biome format` produce spec-compliant YAML out of the box.

**Files to change:**
- `crates/biome_service/src/file_handlers/yaml.rs` — override default indent_style/indent_width for YAML

### 2. Compact block sequence form
**Impact: High | Effort: Medium**

Currently `- key: value` sequences with mappings expand to:
```yaml
-
  key: value
```
instead of the compact form:
```yaml
- key: value
```

The compact form works correctly with space indentation (via `align(2)`). If #1 is done first (default to spaces), the compact form becomes viable and produces the standard YAML convention used in GitHub Actions, Kubernetes, docker-compose, etc.

**Files to change:**
- `crates/biome_yaml_formatter/src/yaml/auxiliary/block_sequence_entry.rs` — use `align(2)` for block mapping values

**Known limitation:** Does not work with tab indentation due to `align`/`indent` interaction producing indentation level collisions in the YAML parser. Only works with space indentation.

### 3. `quote_style` formatter option
**Impact: Medium | Effort: Medium**

Allow users to configure preferred quoting style (single/double). Currently deferred because no existing Biome language formatter has this pattern to follow.

**Files to change:**
- `crates/biome_yaml_formatter/src/context.rs` — add `quote_style` field to `YamlFormatOptions`
- `crates/biome_configuration/src/yaml.rs` — add `quote_style` to config
- `crates/biome_service/src/file_handlers/yaml.rs` — wire through
- `crates/biome_yaml_formatter/src/yaml/auxiliary/double_quoted_scalar.rs` / `single_quoted_scalar.rs` — respect the option

### 4. JSON Schema validation
**Impact: High | Effort: High**

Validate YAML structure against JSON schemas (like YAML Language Server does). Major feature for a future phase.

## Recommendation
Items 1 and 2 together would have the biggest user-visible impact — making the formatter produce idiomatic YAML that matches what users expect from tools like Prettier, and what's standard in GitHub Actions, Kubernetes, and docker-compose files.
