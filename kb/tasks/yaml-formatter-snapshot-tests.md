# Add YAML Formatter Snapshot Tests

## Context

The YAML formatter has per-node formatting logic for all 29 node types and 7 list types, but only has 5 inline unit tests in `lib.rs`. Other Biome formatters (JSON, JS, CSS) use the `biome_formatter_test` infrastructure with `insta` snapshot tests generated from spec input files. We need to add this same infrastructure and spec files for the YAML formatter to catch edge cases and prevent regressions.

## Step 1: Add dev-dependencies to `Cargo.toml`

**File**: `crates/biome_yaml_formatter/Cargo.toml`

Add to `[dev-dependencies]` (mirroring `crates/biome_json_formatter/Cargo.toml`):
```toml
biome_formatter_test = { path = "../biome_formatter_test" }
biome_fs             = { path = "../biome_fs" }
biome_service        = { path = "../biome_service" }
biome_test_utils     = { path = "../biome_test_utils" }
serde                = { workspace = true, features = ["derive"] }
serde_json           = { workspace = true }
tests_macros         = { path = "../tests_macros" }
```

## Step 2: Create test infrastructure files

All under `crates/biome_yaml_formatter/tests/`.

### 2a. `language.rs` — YamlTestFormatLanguage

Implements `TestFormatLanguage` trait (from `biome_formatter_test/src/lib.rs`):
- `parse()` calls `parse_yaml(text).into()`
- `to_format_language()` resolves options via `YamlLanguage::resolve_format_options()`

Reference: `crates/biome_json_formatter/tests/language.rs`

### 2b. `spec_test.rs` — Test runner

The `run()` function called by the gen_tests macro. Creates `SpecTestFile`, resolves formatting options with `create_formatting_options::<YamlLanguage>()`, builds a `SpecSnapshot`, and calls `snapshot.test()`.

Reference: `crates/biome_json_formatter/tests/spec_test.rs`

### 2c. `spec_tests.rs` — Test macro entry point

```rust
mod quick_test;
mod spec_test;

mod formatter {
    mod yaml_module {
        tests_macros::gen_tests! {"tests/specs/yaml/**/*.yaml", crate::spec_test::run, ""}
    }
}
```

### 2d. `quick_test.rs` — Development quick test

An `#[ignore]`-d test for rapid iteration without snapshots. Formats a YAML snippet, runs `CheckReformat` for idempotency, and asserts expected output.

## Step 3: Create spec input files

**Directory**: `crates/biome_yaml_formatter/tests/specs/yaml/`

### `mapping/`
- `simple.yaml` — Single key-value: `key: value`
- `multiple_keys.yaml` — Multiple top-level keys
- `nested.yaml` — Nested mappings (indentation test)

### `sequence/`
- `simple.yaml` — Block sequence: `- item1\n- item2`
- `nested.yaml` — Nested sequences and sequences of mappings

### `scalar/`
- `quoted.yaml` — Single and double quoted scalars
- `literal_block.yaml` — `|` literal block scalar
- `folded_block.yaml` — `>` folded block scalar

### `flow/`
- `mapping.yaml` — Flow mapping `{key: value}`
- `sequence.yaml` — Flow sequence `[a, b, c]`

### `document/`
- `markers.yaml` — `---` and `...` document markers
- `multiple.yaml` — Multiple documents in one file

### `properties/`
- `anchor_alias.yaml` — `&anchor` and `*alias`
- `tag.yaml` — Tag properties `!!str`

### `comments/`
- `inline.yaml` — Comments after values
- `own_line.yaml` — Comments on their own line

### Root level
- `empty.yaml` — Empty file
- `smoke.yaml` — Mixed representative YAML covering multiple features

## Step 4: Verification

1. `cargo build -p biome_yaml_formatter` — Ensure it compiles
2. `cargo test -p biome_yaml_formatter` — First run generates `.snap` files (tests will fail)
3. `cargo insta accept` — Accept the generated snapshots
4. `cargo test -p biome_yaml_formatter` — All tests should now pass
5. Review `.snap` files to verify formatting output is correct

## Files Summary

| File | Action |
|------|--------|
| `Cargo.toml` | Add dev-dependencies |
| `tests/language.rs` | **New** — TestFormatLanguage impl |
| `tests/spec_test.rs` | **New** — Test runner |
| `tests/spec_tests.rs` | **New** — Test macro entry |
| `tests/quick_test.rs` | **New** — Dev quick test |
| `tests/specs/yaml/**/*.yaml` | **New** — ~20 spec input files |
| `tests/specs/yaml/**/*.yaml.snap` | **Auto-generated** by insta |
