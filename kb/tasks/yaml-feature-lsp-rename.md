# Feature: Wire Up LSP textDocument/rename for YAML Anchors/Aliases

## Status: Not Started

## Summary

The YAML rename functionality is fully implemented at the workspace/service layer but not exposed via the standard LSP `textDocument/rename` protocol. Currently it's only available as a custom `biome/rename` JSON-RPC method.

## Current State

- **Service layer** (`crates/biome_service/src/file_handlers/yaml.rs` lines 477-598): Complete implementation that:
  - Finds the token at cursor position
  - Detects `ANCHOR_PROPERTY_LITERAL` (`&name`) or `ALIAS_LITERAL` (`*name`) tokens
  - Scopes rename to the containing YAML document (multi-doc aware)
  - Collects all matching anchor/alias tokens and builds `RenameResult` with text edits
- **LSP capabilities** (`crates/biome_lsp/src/capabilities.rs` line 115): `rename_provider: None`
- **Custom method** (`crates/biome_lsp/src/server.rs` line 704): `workspace_method!(builder, rename)` registers `biome/rename`
- **No existing rename tests** in `crates/biome_lsp/src/server.tests.rs` for any language

## Implementation Plan

### 1. Enable LSP Rename Capability

**File:** `crates/biome_lsp/src/capabilities.rs`

Change:
```rust
rename_provider: None,
```
to:
```rust
rename_provider: Some(OneOf::Left(true)),
```

### 2. Add textDocument/rename Handler

**File:** `crates/biome_lsp/src/server.rs`

Add a handler that translates:
- LSP `RenameParams` (position + new_name) → workspace `RenameParams` (symbol_at + new_name)
- workspace `RenameResult` (range + indels) → LSP `WorkspaceEdit` (document changes)

Follow the pattern used by other LSP handlers (hover, goto_definition).

### 3. Add LSP Tests

**File:** `crates/biome_lsp/src/server.tests.rs`

Three tests following existing YAML LSP test patterns:
- `yaml_rename_anchor_updates_aliases` — rename `&anchor` → all `*anchor` references updated
- `yaml_rename_alias_updates_anchor` — rename `*alias` → the `&alias` anchor updated
- `yaml_rename_non_renameable_returns_error` — rename on plain text returns error/null

### 4. Imports Needed

Add to `server.tests.rs` imports:
```rust
use tower_lsp_server::ls_types::RenameParams;
```

## Estimated Scope

~50-100 lines across 3 files. This is a feature addition (wiring up the LSP protocol), not just a test gap.

## Dependencies

None — the workspace-level rename is fully functional.

## References

- Workspace types: `crates/biome_service/src/workspace.rs` lines 1126-1141 (`RenameParams`, `RenameResult`)
- YAML rename impl: `crates/biome_service/src/file_handlers/yaml.rs` lines 477-598
- Existing YAML LSP tests: `crates/biome_lsp/src/server.tests.rs` (search `yaml_hover`, `yaml_goto_definition`)
- LSP rename spec: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_rename
