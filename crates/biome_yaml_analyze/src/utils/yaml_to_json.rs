use biome_rowan::{AstNode, AstNodeList, AstSeparatedList, TextRange};
use biome_yaml_syntax::*;
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::{Map, Value};

/// Context for YAML→JSON conversion with alias resolution.
struct ConvertCtx {
    /// Anchor name → the block/flow node that the anchor annotates.
    anchor_values: FxHashMap<String, YamlSyntaxNode>,
    /// Guard against infinite recursion when resolving circular aliases.
    resolving: FxHashSet<String>,
}

impl ConvertCtx {
    fn new(root: &YamlSyntaxNode) -> Self {
        let mut anchor_values = FxHashMap::default();

        // Collect all anchors and their value nodes.
        // An anchor property is a child of a "properties" wrapper node,
        // which in turn is a child of the node being anchored.
        for node in root.descendants() {
            if node.kind() == YamlSyntaxKind::YAML_ANCHOR_PROPERTY {
                if let Some(token) = node
                    .children_with_tokens()
                    .filter_map(|c| c.into_token())
                    .find(|t| t.kind() == YamlSyntaxKind::ANCHOR_PROPERTY_LITERAL)
                {
                    let name = token
                        .text_trimmed()
                        .strip_prefix('&')
                        .unwrap_or(token.text_trimmed())
                        .to_string();
                    // The value node is the grandparent: anchor_prop → properties → value_node
                    if let Some(value_node) = node.parent().and_then(|p| p.parent()) {
                        anchor_values.insert(name, value_node);
                    }
                }
            }
        }

        Self {
            anchor_values,
            resolving: FxHashSet::default(),
        }
    }
}

/// Convert a YAML document's root node to a JSON value.
/// Aliases are resolved to their anchor values (circular references return null).
pub fn yaml_node_to_json(node: &AnyYamlBlockNode) -> Value {
    let mut ctx = ConvertCtx::new(node.syntax());
    yaml_node_to_json_ctx(node, &mut ctx)
}

fn yaml_node_to_json_ctx(node: &AnyYamlBlockNode, ctx: &mut ConvertCtx) -> Value {
    match node {
        AnyYamlBlockNode::AnyYamlBlockInBlockNode(inner) => block_in_block_to_json(inner, ctx),
        AnyYamlBlockNode::YamlFlowInBlockNode(flow) => flow
            .flow()
            .map_or(Value::Null, |f| flow_node_to_json(&f, ctx)),
        AnyYamlBlockNode::YamlBogusBlockNode(_) => Value::Null,
    }
}

fn block_in_block_to_json(node: &AnyYamlBlockInBlockNode, ctx: &mut ConvertCtx) -> Value {
    match node {
        AnyYamlBlockInBlockNode::YamlBlockMapping(mapping) => {
            block_mapping_to_json(mapping, ctx)
        }
        AnyYamlBlockInBlockNode::YamlBlockSequence(seq) => block_sequence_to_json(seq, ctx),
        AnyYamlBlockInBlockNode::YamlLiteralScalar(s) => scalar_content_to_json(s.content().ok()),
        AnyYamlBlockInBlockNode::YamlFoldedScalar(s) => scalar_content_to_json(s.content().ok()),
    }
}

fn block_mapping_to_json(mapping: &YamlBlockMapping, ctx: &mut ConvertCtx) -> Value {
    let mut map = Map::new();
    for entry in mapping.entries().iter() {
        match entry {
            AnyYamlBlockMapEntry::YamlBlockMapImplicitEntry(e) => {
                if let Some(key) = e.key() {
                    let key_text = implicit_key_text(&key);
                    let value = e
                        .value()
                        .map(|v| yaml_node_to_json_ctx(&v, ctx))
                        .unwrap_or(Value::Null);
                    map.insert(key_text, value);
                }
            }
            AnyYamlBlockMapEntry::YamlBlockMapExplicitEntry(e) => {
                let key_text = e
                    .key()
                    .map(|k| k.syntax().text_trimmed().to_string().trim().to_string())
                    .unwrap_or_default();
                let value = e
                    .value()
                    .map(|v| yaml_node_to_json_ctx(&v, ctx))
                    .unwrap_or(Value::Null);
                map.insert(key_text, value);
            }
            AnyYamlBlockMapEntry::YamlBogusBlockMapEntry(_) => {}
        }
    }
    Value::Object(map)
}

fn block_sequence_to_json(seq: &YamlBlockSequence, ctx: &mut ConvertCtx) -> Value {
    let items: Vec<Value> = seq
        .entries()
        .iter()
        .filter_map(|entry| match entry {
            AnyYamlBlockSequenceEntry::YamlBlockSequenceEntry(e) => Some(
                e.value()
                    .map(|v| yaml_node_to_json_ctx(&v, ctx))
                    .unwrap_or(Value::Null),
            ),
            AnyYamlBlockSequenceEntry::YamlBogus(_) => None,
        })
        .collect();
    Value::Array(items)
}

fn flow_node_to_json(node: &AnyYamlFlowNode, ctx: &mut ConvertCtx) -> Value {
    match node {
        AnyYamlFlowNode::YamlFlowJsonNode(json) => flow_json_node_to_json(json, ctx),
        AnyYamlFlowNode::YamlFlowYamlNode(yaml) => yaml
            .content()
            .map(|s| plain_scalar_to_json(&s))
            .unwrap_or(Value::Null),
        AnyYamlFlowNode::YamlAliasNode(alias) => resolve_alias_to_json(alias, ctx),
        AnyYamlFlowNode::YamlBogusFlowNode(_) => Value::Null,
    }
}

/// Resolve an alias node to its anchor's JSON value.
fn resolve_alias_to_json(alias: &YamlAliasNode, ctx: &mut ConvertCtx) -> Value {
    let name = alias
        .value_token()
        .ok()
        .map(|t| {
            t.text_trimmed()
                .strip_prefix('*')
                .unwrap_or(t.text_trimmed())
                .to_string()
        })
        .unwrap_or_default();

    if name.is_empty() {
        return Value::Null;
    }

    // Guard against circular references
    if ctx.resolving.contains(&name) {
        return Value::Null;
    }

    let value_node = match ctx.anchor_values.get(&name) {
        Some(node) => node.clone(),
        None => return Value::Null,
    };

    ctx.resolving.insert(name.clone());

    // Try to cast the value node to a block node and convert
    let result = if let Some(block_mapping) = YamlBlockMapping::cast(value_node.clone()) {
        block_mapping_to_json(&block_mapping, ctx)
    } else if let Some(block_seq) = YamlBlockSequence::cast(value_node.clone()) {
        block_sequence_to_json(&block_seq, ctx)
    } else if let Some(flow_node) = AnyYamlFlowNode::cast(value_node.clone()) {
        flow_node_to_json(&flow_node, ctx)
    } else if let Some(flow_in_block) = YamlFlowInBlockNode::cast(value_node.clone()) {
        flow_in_block
            .flow()
            .map_or(Value::Null, |f| flow_node_to_json(&f, ctx))
    } else if let Some(block_in_block) = AnyYamlBlockInBlockNode::cast(value_node.clone()) {
        block_in_block_to_json(&block_in_block, ctx)
    } else if let Some(block_node) = AnyYamlBlockNode::cast(value_node) {
        yaml_node_to_json_ctx(&block_node, ctx)
    } else {
        Value::Null
    };

    ctx.resolving.remove(&name);
    result
}

fn flow_json_node_to_json(node: &YamlFlowJsonNode, ctx: &mut ConvertCtx) -> Value {
    match node.content() {
        Some(content) => json_content_to_json(&content, ctx),
        None => Value::Null,
    }
}

fn json_content_to_json(content: &AnyYamlJsonContent, ctx: &mut ConvertCtx) -> Value {
    match content {
        AnyYamlJsonContent::YamlFlowMapping(mapping) => flow_mapping_to_json(mapping, ctx),
        AnyYamlJsonContent::YamlFlowSequence(seq) => flow_sequence_to_json(seq, ctx),
        AnyYamlJsonContent::YamlDoubleQuotedScalar(s) => s
            .value_token()
            .ok()
            .map(|t| Value::String(unquote_double(t.text())))
            .unwrap_or(Value::Null),
        AnyYamlJsonContent::YamlSingleQuotedScalar(s) => s
            .value_token()
            .ok()
            .map(|t| Value::String(unquote_single(t.text())))
            .unwrap_or(Value::Null),
    }
}

fn flow_mapping_to_json(mapping: &YamlFlowMapping, ctx: &mut ConvertCtx) -> Value {
    let mut map = Map::new();
    for entry in mapping.entries().iter().flatten() {
        match entry {
            AnyYamlFlowMapEntry::YamlFlowMapImplicitEntry(e) => {
                if let Some(key) = e.key() {
                    let key_text = implicit_key_text(&key);
                    let value = e
                        .value()
                        .map(|v| flow_node_to_json(&v, ctx))
                        .unwrap_or(Value::Null);
                    map.insert(key_text, value);
                }
            }
            AnyYamlFlowMapEntry::YamlFlowMapExplicitEntry(e) => {
                if let Some(key) = e.key() {
                    let key_text = implicit_key_text(&key);
                    let value = e
                        .value()
                        .map(|v| flow_node_to_json(&v, ctx))
                        .unwrap_or(Value::Null);
                    map.insert(key_text, value);
                }
            }
        }
    }
    Value::Object(map)
}

fn flow_sequence_to_json(seq: &YamlFlowSequence, ctx: &mut ConvertCtx) -> Value {
    let items: Vec<Value> = seq
        .entries()
        .iter()
        .flatten()
        .map(|entry| match entry {
            AnyYamlFlowSequenceEntry::AnyYamlFlowNode(node) => flow_node_to_json(&node, ctx),
            AnyYamlFlowSequenceEntry::AnyYamlFlowMapEntry(map_entry) => {
                // Compact notation: [a: 1] is [{a: 1}]
                let mut map = Map::new();
                match map_entry {
                    AnyYamlFlowMapEntry::YamlFlowMapImplicitEntry(e) => {
                        if let Some(key) = e.key() {
                            let key_text = implicit_key_text(&key);
                            let value = e
                                .value()
                                .map(|v| flow_node_to_json(&v, ctx))
                                .unwrap_or(Value::Null);
                            map.insert(key_text, value);
                        }
                    }
                    AnyYamlFlowMapEntry::YamlFlowMapExplicitEntry(e) => {
                        if let Some(key) = e.key() {
                            let key_text = implicit_key_text(&key);
                            let value = e
                                .value()
                                .map(|v| flow_node_to_json(&v, ctx))
                                .unwrap_or(Value::Null);
                            map.insert(key_text, value);
                        }
                    }
                }
                Value::Object(map)
            }
        })
        .collect();
    Value::Array(items)
}

fn implicit_key_text(key: &AnyYamlMappingImplicitKey) -> String {
    match key {
        AnyYamlMappingImplicitKey::YamlFlowJsonNode(node) => {
            node.content().map_or(String::new(), |c| match &c {
                AnyYamlJsonContent::YamlDoubleQuotedScalar(s) => s
                    .value_token()
                    .ok()
                    .map(|t| unquote_double(t.text()))
                    .unwrap_or_default(),
                AnyYamlJsonContent::YamlSingleQuotedScalar(s) => s
                    .value_token()
                    .ok()
                    .map(|t| unquote_single(t.text()))
                    .unwrap_or_default(),
                other => other.syntax().text_trimmed().to_string().trim().to_string(),
            })
        }
        AnyYamlMappingImplicitKey::YamlFlowYamlNode(node) => node
            .content()
            .map(|s| s.value_token().ok().map(|t| t.text().trim().to_string()))
            .flatten()
            .unwrap_or_default(),
    }
}

/// Interpret a plain scalar as a typed JSON value.
fn plain_scalar_to_json(scalar: &YamlPlainScalar) -> Value {
    let text = scalar
        .value_token()
        .ok()
        .map(|t| t.text().trim().to_string())
        .unwrap_or_default();
    parse_scalar_value(&text)
}

fn scalar_content_to_json(content: Option<YamlBlockContent>) -> Value {
    content
        .and_then(|c| c.value_token().ok())
        .map(|t| Value::String(t.text().to_string()))
        .unwrap_or(Value::Null)
}

/// Parse a plain scalar string into a typed JSON value.
fn parse_scalar_value(text: &str) -> Value {
    match text {
        "null" | "Null" | "NULL" | "~" | "" => Value::Null,
        "true" | "True" | "TRUE" => Value::Bool(true),
        "false" | "False" | "FALSE" => Value::Bool(false),
        _ => {
            // Try integer
            if let Ok(n) = text.parse::<i64>() {
                return Value::Number(n.into());
            }
            // Try float
            if let Ok(n) = text.parse::<f64>() {
                if let Some(n) = serde_json::Number::from_f64(n) {
                    return Value::Number(n);
                }
            }
            Value::String(text.to_string())
        }
    }
}

fn unquote_double(s: &str) -> String {
    s.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(s)
        .to_string()
}

fn unquote_single(s: &str) -> String {
    s.strip_prefix('\'')
        .and_then(|s| s.strip_suffix('\''))
        .unwrap_or(s)
        .to_string()
}

/// Resolve a JSON instance path (e.g., "/spec/containers/0/ports") to a
/// `TextRange` in the YAML document. Returns `None` if the path cannot be
/// resolved.
pub fn resolve_path_range(root: &AnyYamlBlockNode, path: &str) -> Option<TextRange> {
    if path.is_empty() || path == "/" {
        return Some(root.syntax().text_trimmed_range());
    }

    let segments: Vec<&str> = path
        .strip_prefix('/')
        .unwrap_or(path)
        .split('/')
        .collect();

    resolve_segments(root, &segments)
}

fn resolve_segments(node: &AnyYamlBlockNode, segments: &[&str]) -> Option<TextRange> {
    if segments.is_empty() {
        return Some(node.syntax().text_trimmed_range());
    }

    let segment = segments[0];
    let rest = &segments[1..];

    match node {
        AnyYamlBlockNode::AnyYamlBlockInBlockNode(inner) => match inner {
            AnyYamlBlockInBlockNode::YamlBlockMapping(mapping) => {
                for entry in mapping.entries().iter() {
                    let (key_text, value) = match &entry {
                        AnyYamlBlockMapEntry::YamlBlockMapImplicitEntry(e) => {
                            let key = e.key()?;
                            let key_text = implicit_key_text(&key);
                            (key_text, e.value())
                        }
                        AnyYamlBlockMapEntry::YamlBlockMapExplicitEntry(e) => {
                            let key_text = e
                                .key()
                                .map(|k| k.syntax().text_trimmed().to_string().trim().to_string())
                                .unwrap_or_default();
                            (key_text, e.value())
                        }
                        AnyYamlBlockMapEntry::YamlBogusBlockMapEntry(_) => continue,
                    };

                    if key_text == segment {
                        if let Some(value_node) = value {
                            if rest.is_empty() {
                                return Some(entry.syntax().text_trimmed_range());
                            }
                            return resolve_segments(&value_node, rest);
                        }
                        return Some(entry.syntax().text_trimmed_range());
                    }
                }
                None
            }
            AnyYamlBlockInBlockNode::YamlBlockSequence(seq) => {
                let index: usize = segment.parse().ok()?;
                let entries: Vec<_> = seq.entries().iter().collect();
                let entry = entries.get(index)?;
                match entry {
                    AnyYamlBlockSequenceEntry::YamlBlockSequenceEntry(e) => {
                        if let Some(value_node) = e.value() {
                            if rest.is_empty() {
                                return Some(e.syntax().text_trimmed_range());
                            }
                            return resolve_segments(&value_node, rest);
                        }
                        Some(e.syntax().text_trimmed_range())
                    }
                    AnyYamlBlockSequenceEntry::YamlBogus(_) => None,
                }
            }
            _ => None,
        },
        AnyYamlBlockNode::YamlFlowInBlockNode(_) => {
            // Flow nodes in block context — return range of the whole node
            Some(node.syntax().text_trimmed_range())
        }
        AnyYamlBlockNode::YamlBogusBlockNode(_) => None,
    }
}
