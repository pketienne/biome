use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, AstNodeList, AstSeparatedList, TextRange};
use biome_yaml_syntax::{
    AnyYamlBlockMapEntry, AnyYamlBlockNode, AnyYamlFlowNode, YamlBlockMapping,
};

use crate::lint::correctness::no_duplicate_keys::get_key_text;

declare_lint_rule! {
    /// Validates that merge keys (`<<`) have alias values.
    ///
    /// The YAML merge key (`<<`) is used to merge mappings via aliases. The value
    /// of a merge key must be an alias (`*name`) or a sequence of aliases. Using
    /// a plain scalar or inline mapping as the merge key value is invalid.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// defaults:
    ///   timeout: 30
    /// server:
    ///   <<: not_an_alias
    ///   port: 8080
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// defaults: &defaults
    ///   timeout: 30
    /// server:
    ///   <<: *defaults
    ///   port: 8080
    /// ```
    pub UseValidMergeKeys {
        version: "next",
        name: "useValidMergeKeys",
        language: "yaml",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct InvalidMergeKeyState {
    range: TextRange,
}

impl Rule for UseValidMergeKeys {
    type Query = Ast<YamlBlockMapping>;
    type State = InvalidMergeKeyState;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let mapping = ctx.query();
        let mut signals = Vec::new();

        for entry in mapping.entries().iter() {
            let (key_text, value, value_range) = match &entry {
                AnyYamlBlockMapEntry::YamlBlockMapImplicitEntry(implicit) => {
                    let key = match implicit.key() {
                        Some(k) => k,
                        None => continue,
                    };
                    let text = match get_key_text(&key) {
                        Some(t) => t,
                        None => continue,
                    };
                    let value = implicit.value();
                    let range = implicit.syntax().text_trimmed_range();
                    (text, value, range)
                }
                AnyYamlBlockMapEntry::YamlBlockMapExplicitEntry(_) | AnyYamlBlockMapEntry::YamlBogusBlockMapEntry(_) => continue,
            };

            if key_text != "<<" {
                continue;
            }

            // Merge key found — validate value is an alias or sequence of aliases
            let value = match value {
                Some(v) => v,
                None => {
                    signals.push(InvalidMergeKeyState { range: value_range });
                    continue;
                }
            };

            if !is_valid_merge_value(&value) {
                signals.push(InvalidMergeKeyState { range: value_range });
            }
        }

        signals.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Merge key "<Emphasis>"<<"</Emphasis>" must have an alias value."
                },
            )
            .note(markup! {
                "The value of a merge key must be an alias (e.g., `*defaults`) or a sequence of aliases."
            }),
        )
    }
}

/// Check if a block node value is a valid merge key target (alias or sequence of aliases)
fn is_valid_merge_value(value: &AnyYamlBlockNode) -> bool {
    match value {
        AnyYamlBlockNode::YamlFlowInBlockNode(flow_in_block) => {
            if let Ok(flow) = flow_in_block.flow() {
                is_valid_merge_flow_value(&flow)
            } else {
                false
            }
        }
        // Block sequences/mappings are not valid merge targets
        _ => false,
    }
}

/// Check if a flow node is a valid merge value (alias, or flow sequence of aliases)
fn is_valid_merge_flow_value(flow: &AnyYamlFlowNode) -> bool {
    match flow {
        AnyYamlFlowNode::YamlAliasNode(_) => true,
        AnyYamlFlowNode::YamlFlowJsonNode(json_node) => {
            // Check if it's a flow sequence containing only aliases
            if let Some(content) = json_node.content() {
                match content {
                    biome_yaml_syntax::AnyYamlJsonContent::YamlFlowSequence(seq) => {
                        let entries = seq.entries();
                        if entries.is_empty() {
                            return false;
                        }
                        entries.iter().flatten().all(|entry| {
                            matches!(
                                entry,
                                biome_yaml_syntax::AnyYamlFlowSequenceEntry::AnyYamlFlowNode(
                                    AnyYamlFlowNode::YamlAliasNode(_)
                                )
                            )
                        })
                    }
                    _ => false,
                }
            } else {
                false
            }
        }
        _ => false,
    }
}
