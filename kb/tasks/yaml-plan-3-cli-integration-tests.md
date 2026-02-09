# Plan 3: CLI Integration Tests

## Status: COMPLETE

## Context

No YAML-specific CLI integration tests exist. Adding basic end-to-end tests ensures `biome format`, `biome lint`, and `biome check` work correctly on YAML files through the full CLI pipeline.

## Pattern

Tests use `MemoryFileSystem` + `BufferConsole` + `run_cli()` with snapshot assertions. Follow existing patterns in `crates/biome_cli/tests/cases/`.

## Test File

**Create**: `crates/biome_cli/tests/cases/handle_yaml_files.rs`

### Tests to Add

1. **`format_yaml_file`** — Format a simple YAML file, verify output
2. **`format_yaml_file_write`** — Format with `--write`, verify file is modified
3. **`lint_yaml_file`** — Lint a YAML file with violations, verify diagnostics
4. **`check_yaml_file`** — Run check on a YAML file
5. **`format_yaml_with_config`** — Format with custom indent settings via biome.json

### Config Constants

Add to `crates/biome_cli/tests/configs.rs`:
```rust
pub const CONFIG_YAML_FORMAT: &str = r#"{
  "yaml": {
    "formatter": {
      "enabled": true
    }
  }
}"#;
```

### Module Registration

Add `mod handle_yaml_files;` to `crates/biome_cli/tests/main.rs` (in the `cases` module).

## Verification
1. `cargo test -p biome_cli handle_yaml` — YAML CLI tests pass
2. Review generated snapshots
