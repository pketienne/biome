use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_source_rule,
};
use biome_console::markup;
use biome_diagnostics::category;
use biome_rowan::{AstNode, BatchMutationExt, Direction, TextRange};
use biome_yaml_semantic::semantic_model;
use biome_yaml_syntax::{YamlLanguage, YamlRoot, YamlSyntaxKind, YamlSyntaxToken};

declare_source_rule! {
    /// Expand YAML merge key (`<<`) references by inlining the merged mapping.
    ///
    /// Merge keys (`<<: *alias`) are a YAML feature that copies entries from
    /// one mapping into another. This assist replaces merge entries with the
    /// actual key-value pairs from the referenced anchor.
    ///
    /// Only applies when the merge value is a single alias referencing a
    /// flat mapping.
    ///
    /// ## Examples
    ///
    /// ```yaml,expect_diff
    /// defaults: &defaults
    ///   color: blue
    ///   size: large
    /// custom:
    ///   <<: *defaults
    ///   color: red
    /// ```
    pub UseExpandedMergeKeys {
        version: "next",
        name: "useExpandedMergeKeys",
        language: "yaml",
        recommended: false,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct MergeKeyState {
    /// Range of the merge entry (`<<: *alias`)
    entry_range: TextRange,
    /// The expanded key-value pairs to replace with
    expanded_text: String,
}

impl Rule for UseExpandedMergeKeys {
    type Query = Ast<YamlRoot>;
    type State = MergeKeyState;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let model = semantic_model(root);
        let mut states = Vec::new();

        // Walk all block mapping entries looking for `<<:` keys
        for node in root.syntax().descendants() {
            if node.kind() != YamlSyntaxKind::YAML_BLOCK_MAP_IMPLICIT_ENTRY {
                continue;
            }

            // Check if the key is `<<`
            let key_text = node
                .children()
                .find(|c| {
                    matches!(
                        c.kind(),
                        YamlSyntaxKind::YAML_PLAIN_SCALAR
                            | YamlSyntaxKind::YAML_DOUBLE_QUOTED_SCALAR
                            | YamlSyntaxKind::YAML_SINGLE_QUOTED_SCALAR
                    )
                })
                .map(|k| k.text_trimmed().to_string());

            let _key_text = match key_text {
                Some(t) if t.trim() == "<<" => t,
                _ => continue,
            };

            // Find the alias value in this entry
            let alias_node = node.descendants().find(|d| {
                d.kind() == YamlSyntaxKind::YAML_ALIAS_NODE
            });

            let alias_node = match alias_node {
                Some(n) => n,
                None => continue,
            };

            let alias_text = alias_node.text_trimmed().to_string();
            let alias_name = alias_text.strip_prefix('*').unwrap_or(&alias_text);

            // Look up the alias in the semantic model
            let alias_ref = model.all_aliases().find(|a| a.name() == alias_name);
            let alias_ref = match alias_ref {
                Some(a) => a,
                None => continue,
            };

            let anchor = match alias_ref.anchor() {
                Some(a) => a,
                None => continue,
            };

            // Get the anchor's associated mapping
            let anchor_syntax = match anchor.syntax() {
                Some(s) => s.clone(),
                None => continue,
            };

            // Walk up from the anchor property to find its sibling block mapping.
            // The tree structure varies, so we walk ancestors until we find a node
            // whose children include a YAML_BLOCK_MAPPING.
            let mut mapping = None;
            let mut current = Some(anchor_syntax.clone());
            while let Some(node) = current {
                // Check siblings at this level
                if let Some(parent) = node.parent() {
                    for child in parent.children() {
                        if child.kind() == YamlSyntaxKind::YAML_BLOCK_MAPPING {
                            mapping = Some(child);
                            break;
                        }
                    }
                    if mapping.is_some() {
                        break;
                    }
                    // Also check descendants of sibling nodes (e.g., through indented blocks)
                    for child in parent.children() {
                        if child.text_trimmed_range() != anchor_syntax.text_trimmed_range()
                        {
                            if let Some(m) = child.descendants().find(|d| {
                                d.kind() == YamlSyntaxKind::YAML_BLOCK_MAPPING
                            }) {
                                mapping = Some(m);
                                break;
                            }
                        }
                    }
                    if mapping.is_some() {
                        break;
                    }
                    current = Some(parent);
                } else {
                    break;
                }
            }

            let mapping = match mapping {
                Some(m) => m,
                None => continue,
            };

            // Extract key-value pairs from the mapping
            let mut expanded_lines = Vec::new();
            for entry in mapping.children() {
                if entry.kind() == YamlSyntaxKind::YAML_BLOCK_MAP_IMPLICIT_ENTRY {
                    let entry_text = entry.text_trimmed().to_string();
                    let trimmed = entry_text.trim();
                    if !trimmed.is_empty() {
                        expanded_lines.push(trimmed.to_string());
                    }
                }
            }

            if expanded_lines.is_empty() {
                continue;
            }

            states.push(MergeKeyState {
                entry_range: node.text_trimmed_range(),
                expanded_text: expanded_lines.join("\n"),
            });
        }

        states.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                category!("assist/source/useExpandedMergeKeys"),
                state.entry_range,
                markup! {
                    "This merge key can be expanded with the referenced entries."
                },
            )
            .note(markup! {
                "Expanding merge keys makes the mapping entries explicit and removes the alias dependency."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleAction<YamlLanguage>> {
        let root = ctx.query();
        let mut mutation = ctx.root().begin();

        for node in root.syntax().descendants() {
            if node.text_trimmed_range() != state.entry_range {
                continue;
            }

            // Collect all tokens in the merge entry node
            let tokens: Vec<_> = node.descendants_tokens(Direction::Next).collect();

            if tokens.is_empty() {
                return None;
            }

            // Replace the first token with the expanded text
            let first_token = tokens[0].clone();
            let new_token = YamlSyntaxToken::new_detached(
                first_token.kind(),
                &state.expanded_text,
                [],
                [],
            );
            mutation.replace_token_transfer_trivia(first_token, new_token);

            // Remove all subsequent tokens
            for token in &tokens[1..] {
                let empty = YamlSyntaxToken::new_detached(token.kind(), "", [], []);
                mutation.replace_token_transfer_trivia(token.clone(), empty);
            }

            return Some(RuleAction::new(
                ctx.metadata().action_category(ctx.category(), ctx.group()),
                ctx.metadata().applicability(),
                markup! { "Expand merge key entries." }.to_owned(),
                mutation,
            ));
        }

        None
    }
}
