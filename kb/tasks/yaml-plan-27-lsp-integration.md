# Plan 27: LSP Integration for YAML

## Status: PENDING

## Context

YAML already has basic LSP support (diagnostics, formatting, code actions, rename). This plan adds hover, go-to-definition, and completions for anchors/aliases.

---

## Changes Required

### A. Hover Support
- Show anchor info when hovering over aliases
- Show alias count when hovering over anchors
- Requires adding hover handler to yaml file handler

### B. Go-to-Definition
- Navigate from alias to its anchor definition
- Uses semantic model to resolve alias → anchor

### C. Completions
- Suggest anchor names when typing `*` (alias prefix)
- Uses semantic model to list available anchors

---

## Files to Modify

### In biome_service:
- `crates/biome_service/src/file_handlers/yaml.rs` — add hover, go-to-definition, completion handlers
- `crates/biome_service/src/file_handlers/mod.rs` — wire new capabilities

### In biome_lsp (if needed):
- `crates/biome_lsp/src/handlers/` — add YAML-specific handlers

## Note

This plan requires changes to crates outside the yaml-specific crates. Implementation may need to be adjusted based on existing LSP infrastructure patterns.

## Verification

- `cargo build -p biome_service`
- `cargo build -p biome_lsp`
- `cargo build -p biome_cli`
