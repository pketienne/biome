use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, AstNodeList, TextRange};
use biome_rule_options::use_valid_schema::UseValidSchemaOptions;
use biome_yaml_syntax::{AnyYamlBlockInBlockNode, AnyYamlBlockMapEntry, AnyYamlBlockNode, AnyYamlMappingImplicitKey, YamlDocument};
use std::path::{Path, PathBuf};

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

        let file_path_str = ctx.file_path().as_str();

        let schema_path = match &options.schema_path {
            Some(path) => path.clone(),
            None => {
                // Try schema associations first (glob pattern matching)
                if let Some(path) = find_schema_by_association(file_path_str, options) {
                    path
                } else {
                    // Then try to find a schema comment in the document
                    match find_schema_comment(document) {
                        Some(path) => path,
                        None => return Box::new([]),
                    }
                }
            }
        };

        // Detect URL schemas and report a helpful diagnostic
        if schema_path.starts_with("http://") || schema_path.starts_with("https://") {
            return Box::new([SchemaError {
                message: format!(
                    "URL-based schemas are not yet supported. Provide a local file path instead: {}",
                    schema_path
                ),
                range: document.syntax().text_trimmed_range(),
            }]);
        }

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
        // Pre-compute the root mapping range for root-level errors
        let root_mapping_range = root_mapping_entries_range(&root_node);

        validator
            .iter_errors(&json_value)
            .map(|error| {
                let instance_path = error.instance_path.to_string();
                let error_message = error.to_string();
                let range = if !instance_path.is_empty() && instance_path != "/" {
                    resolve_path_range(&root_node, &instance_path).unwrap_or(doc_range)
                } else {
                    // Root-level error: try to find specific entry from error message
                    find_entry_range_from_error(&root_node, &error_message)
                        .or(root_mapping_range)
                        .unwrap_or(doc_range)
                };
                SchemaError {
                    message: error_message,
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
/// Returns `None` for URL schemas (http/https) since those require network access.
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
                        // Skip URL schemas — they require network access
                        if path.starts_with("http://") || path.starts_with("https://") {
                            return None;
                        }
                        return Some(path.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Get the combined range of all entries in the root mapping.
fn root_mapping_entries_range(root: &AnyYamlBlockNode) -> Option<TextRange> {
    let mapping = match root {
        AnyYamlBlockNode::AnyYamlBlockInBlockNode(AnyYamlBlockInBlockNode::YamlBlockMapping(
            m,
        )) => m,
        _ => return None,
    };

    let entries = mapping.entries();
    if entries.is_empty() {
        return None;
    }

    let first = entries.iter().next()?;
    let last = entries.iter().last()?;
    Some(TextRange::new(
        first.syntax().text_trimmed_range().start(),
        last.syntax().text_trimmed_range().end(),
    ))
}

/// Try to extract a property name from a schema error message and find its
/// entry range in the root mapping.
fn find_entry_range_from_error(
    root: &AnyYamlBlockNode,
    error_message: &str,
) -> Option<TextRange> {
    let property_name = extract_property_from_error(error_message)?;

    let mapping = match root {
        AnyYamlBlockNode::AnyYamlBlockInBlockNode(AnyYamlBlockInBlockNode::YamlBlockMapping(
            m,
        )) => m,
        _ => return None,
    };

    for entry in mapping.entries().iter() {
        let key_text = match &entry {
            AnyYamlBlockMapEntry::YamlBlockMapImplicitEntry(e) => {
                e.key().map(|k| implicit_key_text(&k))
            }
            AnyYamlBlockMapEntry::YamlBlockMapExplicitEntry(e) => e
                .key()
                .map(|k| k.syntax().text_trimmed().to_string().trim().to_string()),
            AnyYamlBlockMapEntry::YamlBogusBlockMapEntry(_) => None,
        };
        if key_text.as_deref() == Some(property_name) {
            return Some(entry.syntax().text_trimmed_range());
        }
    }
    None
}

/// Extract a property name from common JSON Schema error patterns.
fn extract_property_from_error(message: &str) -> Option<&str> {
    // Pattern: "'extra' was unexpected" (additionalProperties)
    if let Some(rest) = message.strip_prefix('\'') {
        if let Some(name_end) = rest.find("' was unexpected") {
            return Some(&rest[..name_end]);
        }
    }
    // Pattern: "\"name\" is a required property" (required)
    if let Some(rest) = message.strip_prefix('"') {
        if let Some(name_end) = rest.find("\" is a required property") {
            return Some(&rest[..name_end]);
        }
    }
    None
}

fn implicit_key_text(key: &AnyYamlMappingImplicitKey) -> String {
    key.syntax().text_trimmed().to_string().trim().to_string()
}

/// Match a file path against the configured schema associations and return
/// the first matching schema path.
fn find_schema_by_association(
    file_path: &str,
    options: &UseValidSchemaOptions,
) -> Option<String> {
    let associations = options.schema_associations.as_ref()?;
    let file = Path::new(file_path);

    for (pattern, schema_path) in associations {
        if glob_matches(pattern, file) {
            return Some(schema_path.clone());
        }
    }
    None
}

/// Simple glob matching that supports `*` (any chars except `/`) and `**`
/// (any chars including `/`).
fn glob_matches(pattern: &str, path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let path_str = path_str.as_ref();
    glob_match_str(pattern, path_str)
}

fn glob_match_str(pattern: &str, text: &str) -> bool {
    // Split pattern into segments by "**"
    if let Some((before, after)) = pattern.split_once("**") {
        let before = before.strip_suffix('/').unwrap_or(before);
        let after = after.strip_prefix('/').unwrap_or(after);

        // Before must match a prefix, after must match a suffix
        if !before.is_empty() {
            // Find a prefix that matches `before`
            for (i, _) in text.char_indices() {
                let prefix = &text[..i];
                if simple_glob_match(before, prefix) {
                    let rest = text[i..].strip_prefix('/').unwrap_or(&text[i..]);
                    if after.is_empty() || glob_match_str(after, rest) {
                        return true;
                    }
                }
            }
            // Also try the full text as prefix
            if simple_glob_match(before, text) && after.is_empty() {
                return true;
            }
            return false;
        }
        // No before part, so ** matches any prefix
        if after.is_empty() {
            return true;
        }
        // Try matching `after` against any suffix
        for (i, _) in text.char_indices() {
            if glob_match_str(after, &text[i..]) {
                return true;
            }
        }
        return false;
    }

    simple_glob_match(pattern, text)
}

/// Match a simple glob pattern with `*` (matches any chars except `/`).
fn simple_glob_match(pattern: &str, text: &str) -> bool {
    if let Some((before, after)) = pattern.split_once('*') {
        if let Some(rest) = text.strip_prefix(before) {
            // `*` matches everything up to the next `/` (or end)
            for (i, ch) in rest.char_indices() {
                if ch == '/' {
                    break;
                }
                if simple_glob_match(after, &rest[i + ch.len_utf8()..]) {
                    return true;
                }
            }
            // Try matching with `*` consuming nothing or everything up to end
            return simple_glob_match(after, rest)
                || (!rest.contains('/') && simple_glob_match(after, ""));
        }
        return false;
    }

    pattern == text
}
