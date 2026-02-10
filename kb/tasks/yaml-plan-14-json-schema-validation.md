# Plan 14: JSON Schema Validation for YAML

## Status: IMPLEMENTED (initial version)

## Context

JSON Schema validation allows validating YAML document structure against a schema definition. This is the feature that makes YAML Language Server (Red Hat / VS Code) so powerful — it provides auto-completion, hover descriptions, and structure validation for Kubernetes, docker-compose, GitHub Actions, and other schema-backed YAML formats.

### Evaluation: Can the existing Biome JSON formatter be leveraged?

**Finding: No.** Biome has **no JSON schema validation** infrastructure:

- No JSON schema validation exists as a lint rule, service feature, or otherwise
- The only schema-related dependency is `schemars` (v1.2.1), used exclusively for **generating** JSON schemas from Biome's own config types (via `derive(JsonSchema)`) — not for validating external schemas
- Biome's configuration validation uses a custom deserialization framework (`biome_deserialize`) with Rust types defining expected structure, not JSON schemas
- The JSON analyzer has only 4 structural rules (`noDuplicateObjectKeys`, etc.) — none are schema-based
- No JSON schema validation crate (`jsonschema`, `jsonschema-rs`) exists in the dependency tree

**What CAN be leveraged:**
- `biome_json_value::JsonValue` enum (Array, Bool, Null, Number, Object, String) — runtime JSON value representation
- Lint rule framework — fully operational for YAML with 23 existing rules
- Configuration system — supports per-rule options for schema file paths
- Diagnostic reporting — mature system for validation error messages
- YAML AST infrastructure — proven value extraction patterns

## Proposed Architecture

### Approach: Lint Rule with YAML-to-JSON Conversion

Implement as a lint rule (follows Biome's pattern — all validation is lint rules), not a service feature.

### New Dependencies

Add `jsonschema` crate to workspace:
```toml
# Cargo.toml (workspace)
[workspace.dependencies]
jsonschema = "0.28"  # or latest stable
```

### Components

#### 14A. YAML-to-JSON value converter

**New file**: `crates/biome_yaml_analyze/src/utils/yaml_to_json.rs`

Convert YAML AST nodes to `serde_json::Value` for schema validation:

```rust
use biome_yaml_syntax::*;
use serde_json::Value;

pub fn yaml_to_json_value(node: &AnyYamlBlockNode) -> Value {
    match node {
        AnyYamlBlockNode::AnyYamlBlockInBlockNode(inner) => match inner {
            AnyYamlBlockInBlockNode::YamlBlockMapping(mapping) => {
                let mut map = serde_json::Map::new();
                for entry in mapping.entries() {
                    if let Ok(entry) = entry {
                        let key = extract_key_text(&entry);
                        let value = extract_value(&entry);
                        map.insert(key, value);
                    }
                }
                Value::Object(map)
            }
            AnyYamlBlockInBlockNode::YamlBlockSequence(seq) => {
                let items: Vec<Value> = seq.entries()
                    .iter()
                    .filter_map(|e| e.ok())
                    .map(|e| /* convert entry value */)
                    .collect();
                Value::Array(items)
            }
            // ... scalar types → Value::String / Value::Number / Value::Bool / Value::Null
        },
        // ... flow nodes
    }
}
```

#### 14B. Schema loading and caching

**New file**: `crates/biome_yaml_analyze/src/utils/schema_loader.rs`

Schema discovery strategy (in priority order):
1. Explicit path in rule options: `"schemaPath": "./schema.json"`
2. Inline schema reference: `# yaml-language-server: $schema=...` comment
3. SchemaStore lookup by filename (e.g., `docker-compose.yml` → Docker Compose schema)

```rust
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct SchemaCache {
    cache: Mutex<HashMap<String, Value>>,
}

impl SchemaCache {
    pub fn load_schema(&self, path: &str) -> Result<Value, SchemaError> {
        // Check cache first
        // Load from filesystem
        // Parse as JSON
        // Cache and return
    }
}
```

#### 14C. Validation lint rule

**New file**: `crates/biome_yaml_analyze/src/lint/validation/use_valid_schema.rs`

```rust
declare_lint_rule! {
    /// Validate YAML document structure against a JSON schema.
    ///
    /// When a JSON schema is configured for YAML files, this rule validates
    /// that the document structure matches the schema definition.
    ///
    /// ## Options
    ///
    /// ```json
    /// {
    ///   "linter": {
    ///     "rules": {
    ///       "validation": {
    ///         "useValidSchema": {
    ///           "level": "error",
    ///           "options": {
    ///             "schemaPath": "./schema.json"
    ///           }
    ///         }
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    pub UseValidSchema {
        version: "next",
        name: "useValidSchema",
        language: "yaml",
        recommended: false,
        severity: Error,
    }
}

impl Rule for UseValidSchema {
    type Query = Ast<YamlDocument>;
    type State = Vec<SchemaValidationError>;
    type Signals = Vec<Self::State>;
    type Options = UseValidSchemaOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let options = ctx.options();

        // 1. Load schema from configured path
        let schema = load_schema(&options.schema_path)?;

        // 2. Convert YAML AST to JSON value
        let json_value = yaml_to_json_value(document);

        // 3. Validate against schema
        let validator = jsonschema::validator_for(&schema)?;
        let errors: Vec<_> = validator.iter_errors(&json_value).collect();

        // 4. Map schema errors to diagnostics with source ranges
        errors.into_iter().map(|e| {
            SchemaValidationError {
                message: e.to_string(),
                path: e.instance_path.to_string(),
                range: resolve_range_from_path(document, &e.instance_path),
            }
        }).collect()
    }
}
```

#### 14D. Rule registration

**Files**:
- `crates/biome_yaml_analyze/src/lint/mod.rs` — add `pub mod validation;`
- `crates/biome_yaml_analyze/src/lint/validation/mod.rs` — declare module
- Run codegen: `cargo run -p xtask_codegen -- analyzer`

## Verification
1. `cargo build -p biome_yaml_analyze` — compiles with jsonschema dependency
2. Unit tests with inline schemas validating simple YAML documents
3. Integration test with a real schema (e.g., docker-compose schema)
4. Performance test with large YAML files and complex schemas

## Effort Estimate

| Component | Effort |
|-----------|--------|
| YAML-to-JSON converter | 1-2 days |
| Schema loading/caching | 1 day |
| Lint rule implementation | 2-3 days |
| Error range mapping (JSON path → YAML AST range) | 2-3 days |
| Testing | 2-3 days |
| SchemaStore integration (optional) | 2-3 days |
| **Total** | **8-15 days** |

## Notes

- The hardest part is **error range mapping** — JSON schema errors report paths like `/spec/containers/0/ports/0`, which need to be mapped back to YAML AST ranges for accurate diagnostic locations
- SchemaStore integration (auto-discovering schemas by filename) is a nice-to-have that can be deferred
- The `jsonschema` crate adds ~200KB to the binary and brings in several transitive dependencies
- This feature would put Biome ahead of other YAML linters (yamllint, eslint-plugin-yml) which don't offer schema validation
