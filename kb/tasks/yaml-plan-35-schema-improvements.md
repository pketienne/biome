# Plan 35: Schema Validation Improvements

## Status: PENDING

## Context

`useValidSchema` supports local file schemas and gracefully rejects URLs. This plan improves schema validation with better diagnostics, `$ref` handling, and optional HTTP fetching.

---

## Changes

### Phase 1: Better schema error messages
- Extract property paths from nested errors and show them in the diagnostic note
- For `required` errors, list which properties are missing
- For `additionalProperties` errors, name the unexpected property
- File: `crates/biome_yaml_analyze/src/lint/nursery/use_valid_schema.rs`

### Phase 2: `$ref` resolution in local schemas
- When loading a schema, resolve `$ref` pointers within the same file
- Use serde_json path traversal for `#/definitions/Foo` references
- This is already handled by the `jsonschema` crate — verify it works

### Phase 3: Optional HTTP schema fetching (future)
- Add `allow_url_schemas: bool` option (default: false)
- When enabled, use `ureq` or similar lightweight HTTP client
- Cache fetched schemas in memory for the session
- Requires adding HTTP dependency — may be deferred

## Verification
- `cargo build -p biome_yaml_analyze`
- `cargo test -p biome_yaml_analyze`
