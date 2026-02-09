use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange};
use biome_yaml_syntax::YamlRoot;
use biome_yaml_syntax::{YamlFlowMapping, YamlFlowSequence};

declare_lint_rule! {
    /// Enforce block style for mappings and sequences instead of flow style.
    ///
    /// Flow style (`{key: value}`, `[1, 2, 3]`) can be harder to read and
    /// maintain compared to block style. This rule flags flow mappings and
    /// flow sequences, recommending block style instead.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// items: {key: value}
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// items:
    ///   key: value
    /// ```
    pub UseBlockStyle {
        version: "next",
        name: "useBlockStyle",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
    }
}

impl Rule for UseBlockStyle {
    type Query = Ast<YamlRoot>;
    type State = TextRange;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut violations = Vec::new();

        for node in root.syntax().descendants() {
            if YamlFlowMapping::can_cast(node.kind()) {
                violations.push(node.text_trimmed_range());
            } else if YamlFlowSequence::can_cast(node.kind()) {
                violations.push(node.text_trimmed_range());
            }
        }

        violations.into_boxed_slice()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state,
                markup! {
                    "Use block style instead of flow style for collections."
                },
            )
            .note(markup! {
                "Block style is more readable and consistent. Replace flow syntax with block indentation."
            }),
        )
    }
}
