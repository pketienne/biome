# Plan 28: Schema URL Support

## Status: IMPLEMENTED

## Context

Currently `useValidSchema` only supports local file paths for schemas. If a user provides an HTTP/HTTPS URL (via comment or options), it tries to read it as a file and fails silently. This plan adds graceful handling.

---

## Changes

### Phase 1: Graceful URL detection
- In `find_schema_comment()` and schema resolution, detect `http://` or `https://` prefixes
- Return a diagnostic note explaining URL schemas aren't yet supported
- Avoid silent failure

### Phase 2 (future): URL fetching
- Would require HTTP client dependency
- Out of scope for now

---

## Files Modified

- `crates/biome_yaml_analyze/src/lint/nursery/use_valid_schema.rs`

## Verification

- `cargo build -p biome_yaml_analyze`
- `cargo test -p biome_yaml_analyze`
