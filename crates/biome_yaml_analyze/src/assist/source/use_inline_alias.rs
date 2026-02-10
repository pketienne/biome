use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_source_rule,
};
use biome_console::markup;
use biome_diagnostics::category;
use biome_rowan::{AstNode, BatchMutationExt, TextRange};
use biome_yaml_semantic::semantic_model;
use biome_yaml_syntax::{
    YamlLanguage, YamlRoot, YamlSyntaxKind, YamlSyntaxNode, YamlSyntaxToken,
};

declare_source_rule! {
    /// Replace a YAML alias with its anchor's value.
    ///
    /// When a YAML alias (`*name`) references a simple scalar anchor (`&name`),
    /// this assist inlines the anchor's value directly, removing the indirection.
    ///
    /// Only applies to aliases whose anchors hold simple scalar values (plain,
    /// single-quoted, or double-quoted). Complex values like mappings and sequences
    /// are not inlined.
    ///
    /// ## Examples
    ///
    /// ```yaml,expect_diff
    /// base: &default hello
    /// ref: *default
    /// ```
    pub UseInlineAlias {
        version: "next",
        name: "useInlineAlias",
        language: "yaml",
        recommended: false,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct InlineAliasState {
    alias_range: TextRange,
    anchor_name: String,
    replacement_text: String,
}

impl Rule for UseInlineAlias {
    type Query = Ast<YamlRoot>;
    type State = InlineAliasState;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let model = semantic_model(root);

        let mut states = Vec::new();

        for alias in model.all_aliases() {
            let anchor = match alias.anchor() {
                Some(a) => a,
                None => continue,
            };

            let anchor_syntax = match anchor.syntax() {
                Some(node) => node.clone(),
                None => continue,
            };

            // Extract the anchor's value text
            let value_text = match extract_anchor_value_text(&anchor_syntax) {
                Some(text) => text,
                None => continue,
            };

            states.push(InlineAliasState {
                alias_range: alias.range(),
                anchor_name: alias.name().to_string(),
                replacement_text: value_text,
            });
        }

        states.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                category!("assist/source/useInlineAlias"),
                state.alias_range,
                markup! {
                    "This alias can be replaced with the value of anchor "<Emphasis>{&state.anchor_name}</Emphasis>"."
                },
            )
            .note(markup! {
                "Inlining removes the indirection and makes the value explicit."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleAction<YamlLanguage>> {
        let root = ctx.query();
        let mut mutation = ctx.root().begin();

        // Find the alias node by range and replace its token
        for node in root.syntax().descendants() {
            if node.kind() == YamlSyntaxKind::YAML_ALIAS_NODE
                && node.text_trimmed_range() == state.alias_range
            {
                // Get the alias token (the *name token)
                let alias_token = node
                    .children_with_tokens()
                    .filter_map(|c| c.into_token())
                    .next()?;

                let new_token = YamlSyntaxToken::new_detached(
                    alias_token.kind(),
                    &state.replacement_text,
                    [],
                    [],
                );
                mutation.replace_token_transfer_trivia(alias_token, new_token);

                return Some(RuleAction::new(
                    ctx.metadata().action_category(ctx.category(), ctx.group()),
                    ctx.metadata().applicability(),
                    markup! { "Inline the alias value." }.to_owned(),
                    mutation,
                ));
            }
        }

        None
    }
}

/// Extract the scalar value text from an anchor property's associated value node.
///
/// Navigation: anchor_property → parent (properties wrapper) → parent (value node).
/// Then inspect the value node for simple scalar content.
/// Returns `None` for complex values (mappings, sequences, block scalars).
fn extract_anchor_value_text(anchor_node: &YamlSyntaxNode) -> Option<String> {
    // Navigate: anchor_prop → properties wrapper → value node
    let value_node = anchor_node.parent()?.parent()?;

    // Check each child of the value node for scalar content
    for child in value_node.descendants() {
        match child.kind() {
            YamlSyntaxKind::YAML_PLAIN_SCALAR => {
                let token = child
                    .children_with_tokens()
                    .filter_map(|c| c.into_token())
                    .find(|t| t.kind() == YamlSyntaxKind::PLAIN_LITERAL)?;
                return Some(token.text_trimmed().to_string());
            }
            YamlSyntaxKind::YAML_DOUBLE_QUOTED_SCALAR => {
                let token = child
                    .children_with_tokens()
                    .filter_map(|c| c.into_token())
                    .find(|t| t.kind() == YamlSyntaxKind::DOUBLE_QUOTED_LITERAL)?;
                return Some(token.text_trimmed().to_string());
            }
            YamlSyntaxKind::YAML_SINGLE_QUOTED_SCALAR => {
                let token = child
                    .children_with_tokens()
                    .filter_map(|c| c.into_token())
                    .find(|t| t.kind() == YamlSyntaxKind::SINGLE_QUOTED_LITERAL)?;
                return Some(token.text_trimmed().to_string());
            }
            // Skip complex values
            YamlSyntaxKind::YAML_BLOCK_MAPPING
            | YamlSyntaxKind::YAML_BLOCK_SEQUENCE
            | YamlSyntaxKind::YAML_FLOW_MAPPING
            | YamlSyntaxKind::YAML_FLOW_SEQUENCE
            | YamlSyntaxKind::YAML_LITERAL_SCALAR
            | YamlSyntaxKind::YAML_FOLDED_SCALAR => {
                return None;
            }
            _ => {}
        }
    }

    None
}
