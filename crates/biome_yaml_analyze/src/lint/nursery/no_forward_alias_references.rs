use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::TextRange;
use biome_yaml_semantic::semantic_model;
use biome_yaml_syntax::YamlRoot;

declare_lint_rule! {
    /// Disallow aliases that appear before their anchor declarations.
    ///
    /// While the YAML specification permits forward references (aliases that
    /// reference an anchor defined later in the document), many tools and
    /// processors do not handle them correctly. Requiring anchors to appear
    /// before their aliases makes documents easier to read and more portable.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// first: *anchor
    /// second: &anchor value
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// first: &anchor value
    /// second: *anchor
    /// ```
    pub NoForwardAliasReferences {
        version: "next",
        name: "noForwardAliasReferences",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct ForwardAliasState {
    alias_name: String,
    alias_range: TextRange,
    anchor_range: TextRange,
}

impl Rule for NoForwardAliasReferences {
    type Query = Ast<YamlRoot>;
    type State = ForwardAliasState;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let model = semantic_model(root);

        model
            .all_aliases()
            .filter_map(|alias| {
                let anchor = alias.anchor()?;
                // Forward reference: alias appears before its anchor
                if alias.range().start() < anchor.range().start() {
                    Some(ForwardAliasState {
                        alias_name: alias.name().to_string(),
                        alias_range: alias.range(),
                        anchor_range: anchor.range(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.alias_range,
                markup! {
                    "The alias "<Emphasis>{"*"}{&state.alias_name}</Emphasis>" is used before its anchor is declared."
                },
            )
            .detail(
                state.anchor_range,
                markup! {
                    "The anchor "<Emphasis>{"&"}{&state.alias_name}</Emphasis>" is declared here."
                },
            )
            .note(markup! {
                "Declare anchors before referencing them with aliases for better readability and tool compatibility."
            }),
        )
    }
}
