use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange};
use biome_rule_options::use_valid_schema::UseValidSchemaOptions;
use biome_yaml_syntax::YamlDocument;
use std::path::PathBuf;

use crate::utils::yaml_to_json::{resolve_path_range, yaml_node_to_json};

declare_lint_rule! {
    /// Validate a YAML document against a JSON Schema.
    ///
    /// When a JSON Schema is configured via the rule options, this rule converts
    /// the YAML document to a JSON value and validates it against the schema.
    /// Validation errors are reported as diagnostics pointing to the offending
    /// YAML nodes.
    ///
    /// ## Examples
    ///
    /// Given a schema that requires a `name` field of type string:
    ///
    /// ```json
    /// {
    ///   "type": "object",
    ///   "required": ["name"],
    ///   "properties": {
    ///     "name": { "type": "string" }
    ///   }
    /// }
    /// ```
    ///
    /// ### Invalid
    ///
    /// ```yaml,ignore
    /// age: 30
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml,ignore
    /// name: John
    /// age: 30
    /// ```
    pub UseValidSchema {
        version: "next",
        name: "useValidSchema",
        language: "yaml",
        recommended: false,
        severity: Severity::Error,
    }
}

pub struct SchemaError {
    message: String,
    range: TextRange,
}

impl Rule for UseValidSchema {
    type Query = Ast<YamlDocument>;
    type State = SchemaError;
    type Signals = Box<[Self::State]>;
    type Options = UseValidSchemaOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let options = ctx.options();

        let schema_path = match &options.schema_path {
            Some(path) => path.clone(),
            None => {
                // Try to find a schema comment in the document
                match find_schema_comment(document) {
                    Some(path) => path,
                    None => return Box::new([]),
                }
            }
        };

        // Resolve relative paths against the directory of the file being analyzed
        let resolved_path = resolve_schema_path(&schema_path, ctx.file_path().as_str());

        // Read the schema file
        let schema_json = match std::fs::read_to_string(&resolved_path) {
            Ok(content) => content,
            Err(_) => return Box::new([]),
        };

        let schema_value: serde_json::Value = match serde_json::from_str(&schema_json) {
            Ok(v) => v,
            Err(_) => return Box::new([]),
        };

        // Get the root node of the document
        let root_node = match document.node() {
            Some(node) => node,
            None => return Box::new([]),
        };

        // Convert YAML to JSON
        let json_value = yaml_node_to_json(&root_node);

        // Validate
        let validator = match jsonschema::validator_for(&schema_value) {
            Ok(v) => v,
            Err(_) => return Box::new([]),
        };

        let doc_range = document.syntax().text_trimmed_range();

        validator
            .iter_errors(&json_value)
            .map(|error| {
                let instance_path = error.instance_path.to_string();
                let range =
                    resolve_path_range(&root_node, &instance_path).unwrap_or(doc_range);
                SchemaError {
                    message: error.to_string(),
                    range,
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(RuleDiagnostic::new(
            rule_category!(),
            state.range,
            markup! {
                "Schema validation error: "{&state.message}
            },
        ))
    }
}

/// Resolve a schema path. If the path is relative, resolve it against
/// the directory containing the YAML file being analyzed.
fn resolve_schema_path(schema_path: &str, file_path: &str) -> PathBuf {
    let path = PathBuf::from(schema_path);
    if path.is_absolute() {
        return path;
    }
    // Resolve relative to the YAML file's directory
    if let Some(parent) = PathBuf::from(file_path).parent() {
        let resolved = parent.join(&path);
        if resolved.exists() {
            return resolved;
        }
    }
    // Fall back to the path as-is (relative to CWD)
    path
}

/// Look for a `# yaml-language-server: $schema=<path>` comment in the YAML
/// document text and return the schema path if found.
fn find_schema_comment(document: &YamlDocument) -> Option<String> {
    let text = document.syntax().to_string();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('#') {
            let rest = rest.trim();
            if let Some(schema_part) = rest.strip_prefix("yaml-language-server:") {
                let schema_part = schema_part.trim();
                if let Some(path) = schema_part.strip_prefix("$schema=") {
                    let path = path.trim();
                    if !path.is_empty() {
                        return Some(path.to_string());
                    }
                }
            }
        }
    }
    None
}
