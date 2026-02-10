use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::TextRange;
use biome_yaml_semantic::semantic_model;
use biome_yaml_syntax::YamlRoot;

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
}
