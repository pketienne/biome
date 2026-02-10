# Plan 31: LSP Enhancements — Hover, Go-to-Definition, Completions

## Status: PENDING

## Context

YAML has a rich semantic model for anchors/aliases but no LSP hover, go-to-definition, or completions. These capabilities don't exist for ANY language in Biome yet, so this requires adding new infrastructure across the service and LSP layers.

---

## Changes

### Phase 1: Service Layer Infrastructure

**`crates/biome_service/src/workspace.rs`** — Add types and trait methods:
- `HoverParams { project_key, path, position: TextSize }`
- `HoverResult { content: String, range: Option<TextRange> }`
- `GotoDefinitionParams { project_key, path, position: TextSize }`
- `GotoDefinitionResult { definitions: Vec<TextRange> }`
- `GetCompletionsParams { project_key, path, position: TextSize }`
- `GetCompletionsResult { items: Vec<CompletionItem> }`
- Add `fn hover()`, `fn goto_definition()`, `fn get_completions()` to `Workspace` trait

**`crates/biome_service/src/file_handlers/mod.rs`** — Add capability types:
- `type Hover = fn(&BiomePath, AnyParse, TextSize) -> Result<HoverResult, WorkspaceError>`
- `type GotoDefinition = fn(&BiomePath, AnyParse, TextSize) -> Result<GotoDefinitionResult, WorkspaceError>`
- `type GetCompletions = fn(&BiomePath, AnyParse, TextSize) -> Result<GetCompletionsResult, WorkspaceError>`
- Add fields to `AnalyzerCapabilities`

### Phase 2: YAML Handlers

**`crates/biome_service/src/file_handlers/yaml.rs`** — Implement:
- `fn hover()`: At cursor position, check token kind. If alias → show anchor info + value preview. If anchor → show alias count.
- `fn goto_definition()`: If alias → return anchor's range. If anchor → return all alias ranges.
- `fn get_completions()`: If cursor is after `*`, list all anchors in the document.

Uses `biome_yaml_semantic::semantic_model` for resolution.

### Phase 3: Workspace Server

**`crates/biome_service/src/workspace/server.rs`** — Implement trait methods, dispatch to file handler capabilities.

### Phase 4: LSP Layer

**`crates/biome_lsp/src/capabilities.rs`** — Advertise `hover_provider`, `definition_provider`, `completion_provider`
**`crates/biome_lsp/src/server.rs`** — Add `hover()`, `goto_definition()`, `completion()` method impls
**`crates/biome_lsp/src/handlers/analysis.rs`** — Add handler functions

## Verification

- `cargo build -p biome_service`
- `cargo build -p biome_lsp`
- `cargo build -p biome_cli`
