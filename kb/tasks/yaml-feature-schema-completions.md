# Feature: Schema-Driven LSP Completions for YAML

## Status: Not Started

## Summary

Add LSP completions that suggest valid keys and values based on JSON Schema definitions. Currently, completions only suggest anchor names when typing `*`. Schema-aware completions would suggest valid mapping keys, enum values, and types based on the schema associated with the file.

## Current State

- **Anchor completions** (`crates/biome_service/src/file_handlers/yaml.rs`): Working — typing `*` suggests available anchors from the semantic model
- **Schema validation** (`crates/biome_yaml_analyze/src/lint/nursery/use_valid_schema.rs`): The `useValidSchema` rule validates YAML against JSON Schema files, proving schema loading/parsing already works
- **LSP completion handler** (`crates/biome_lsp/src/capabilities.rs`): `completion_provider: Some(CompletionOptions::default())` — already enabled

## Implementation Plan

### 1. Schema Resolution

Determine which JSON Schema applies to a given YAML file. Options:
- **Filename pattern matching**: e.g., `docker-compose*.yaml` → Docker Compose schema
- **Config-based mapping** in `biome.json`: user specifies `{ "yaml": { "schemas": { "glob": "schema_url" } } }`
- **In-file comment**: `# yaml-language-server: $schema=...` (SchemaStore convention)
- **SchemaStore catalog**: Auto-detect from https://www.schemastore.org/api/json/catalog.json

### 2. Schema Loading and Caching

**New module or extend existing:** Schema loading infrastructure

- Load JSON Schema from file path or URL
- Parse into a navigable structure (serde_json::Value or a typed representation)
- Cache parsed schemas per-session (avoid re-parsing on every completion request)
- Handle `$ref` resolution within schemas
- The `useValidSchema` rule already loads schemas per-analysis — this logic can be extracted and shared

### 3. YAML Path Resolution

Given a cursor position, determine the "schema path":
- Parse the YAML to find the cursor's position in the document tree
- Walk up from the cursor to build a path like `spec.template.spec.containers[0].`
- Map this path to a schema node by following `properties`, `items`, `additionalProperties`, etc.

### 4. Completion Generation

From the resolved schema node, generate `CompletionItem`s:
- **Keys**: From `properties` — suggest property names not yet present in the mapping
- **Enum values**: From `enum` — suggest valid values for the current key
- **Types**: From `type` — suggest type-appropriate placeholder values
- **Required markers**: Indicate which properties are required via `detail` field
- **Documentation**: Use `description` from schema as `documentation` field

### 5. Integration

**File:** `crates/biome_service/src/file_handlers/yaml.rs`

Extend the existing `get_completions` handler:
```
1. Check if cursor is in alias position → return anchor completions (existing)
2. Check if a schema is associated with the file → return schema completions (new)
3. Otherwise → return empty
```

### 6. Configuration

**File:** `crates/biome_configuration/src/yaml.rs`

Add schema mapping configuration:
```json
{
  "yaml": {
    "schemas": {
      "https://json.schemastore.org/github-workflow.json": ".github/workflows/*.yaml",
      "./schemas/config.json": "config.yaml"
    }
  }
}
```

## Estimated Scope

~200-400 lines across multiple files:
- Schema resolution/caching: ~100 lines (new module)
- YAML path resolution: ~50-80 lines
- Completion generation: ~80-120 lines
- Configuration types: ~30-50 lines
- Integration in completions handler: ~20-30 lines

## Dependencies

- `serde_json` (already in deps) for schema parsing
- Potentially `jsonschema` crate for `$ref` resolution (or implement manually)
- SchemaStore integration would require HTTP fetching (optional, can start with local schemas)

## References

- Existing completions: `crates/biome_service/src/file_handlers/yaml.rs` (search `get_completions`)
- Schema validation: `crates/biome_yaml_analyze/src/lint/nursery/use_valid_schema.rs`
- Schema loading utils: `crates/biome_yaml_analyze/src/utils/` (if schema helpers exist)
- SchemaStore catalog: https://www.schemastore.org/api/json/catalog.json
- LSP completion spec: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_completion
- yaml-language-server schema comment: https://github.com/redhat-developer/yaml-language-server#language-server-settings
