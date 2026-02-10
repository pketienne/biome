use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, BatchMutationExt, TextRange};
use biome_yaml_semantic::semantic_model;
use biome_yaml_syntax::{YamlAnchorProperty, YamlLanguage, YamlRoot, YamlSyntaxKind};

declare_lint_rule! {
    /// Disallow unused anchors in YAML documents.
    ///
    /// Anchors that are defined but never referenced by an alias are likely
    /// unnecessary and may indicate dead code or a typo in the alias name.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// first: &unused value1
    /// second: value2
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// first: &anchor value1
    /// second: *anchor
    /// ```
    pub NoUnusedAnchors {
        version: "next",
        name: "noUnusedAnchors",
        language: "yaml",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct UnusedAnchorState {
    anchor_name: String,
    range: TextRange,
}

impl Rule for NoUnusedAnchors {
    type Query = Ast<YamlRoot>;
    type State = UnusedAnchorState;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let model = semantic_model(root);

        model
            .all_anchors()
            .filter(|anchor| anchor.all_aliases().is_empty())
            .map(|anchor| UnusedAnchorState {
                anchor_name: anchor.name().to_string(),
                range: anchor.range(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "The anchor "<Emphasis>{&state.anchor_name}</Emphasis>" is defined but never referenced."
                },
            )
            .note(markup! {
                "Remove the unused anchor or add an alias that references it."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleAction<YamlLanguage>> {
        let root = ctx.query();
        let mut mutation = ctx.root().begin();

        // Find the YamlAnchorProperty node that contains the token at state.range
        for node in root.syntax().descendants() {
            if node.kind() == YamlSyntaxKind::YAML_ANCHOR_PROPERTY {
                if let Some(anchor_prop) = YamlAnchorProperty::cast(node) {
                    if let Ok(token) = anchor_prop.value_token() {
                        if token.text_trimmed_range() == state.range {
                            mutation.remove_node(anchor_prop);
                            return Some(RuleAction::new(
                                ctx.metadata().action_category(ctx.category(), ctx.group()),
                                ctx.metadata().applicability(),
                                markup! { "Remove unused anchor." }
                                    .to_owned(),
                                mutation,
                            ));
                        }
                    }
                }
            }
        }

        None
    }
}
