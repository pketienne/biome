use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::AstNode;
use biome_yaml_syntax::YamlAnchorProperty;

declare_lint_rule! {
    /// Forbid the use of anchors and aliases in YAML.
    ///
    /// YAML anchors (`&name`) and aliases (`*name`) allow referencing and
    /// reusing data within a document. While powerful, they reduce readability
    /// and can make YAML files harder to understand and maintain. Some teams
    /// prefer to ban them in favor of explicit duplication.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// defaults: &defaults
    ///   timeout: 30
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// defaults:
    ///   timeout: 30
    /// production:
    ///   timeout: 30
    /// ```
    pub NoAnchorReferences {
        version: "next",
        name: "noAnchorReferences",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
    }
}

impl Rule for NoAnchorReferences {
    type Query = Ast<YamlAnchorProperty>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(_ctx: &RuleContext<Self>) -> Self::Signals {
        Some(())
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        let anchor = ctx.query();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                anchor.syntax().text_trimmed_range(),
                markup! {
                    "Anchors and aliases are not allowed."
                },
            )
            .note(markup! {
                "Duplicate the data explicitly instead of using anchors and aliases."
            }),
        )
    }
}
