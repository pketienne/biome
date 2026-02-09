use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_yaml_syntax::YamlBlockMapImplicitEntry;

declare_lint_rule! {
    /// Disallow empty mapping values in YAML.
    ///
    /// An empty mapping value results in an implicit `null` value. This is
    /// often unintentional and can lead to unexpected behavior. If you intend
    /// a null value, use `null` explicitly for clarity.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// name:
    /// age: 30
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// name: null
    /// age: 30
    /// ```
    ///
    /// ```yaml
    /// name: John
    /// age: 30
    /// ```
    pub NoEmptyValues {
        version: "next",
        name: "noEmptyValues",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
    }
}

impl Rule for NoEmptyValues {
    type Query = Ast<YamlBlockMapImplicitEntry>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let entry = ctx.query();

        // Check if the value is missing (implicit null)
        if entry.value().is_none() {
            return Some(());
        }
        None
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        let entry = ctx.query();
        let range = entry.colon_token().ok()?.text_trimmed_range();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                range,
                markup! {
                    "Empty mapping values result in implicit null."
                },
            )
            .note(markup! {
                "Use an explicit "<Emphasis>"null"</Emphasis>" value or provide a meaningful value."
            }),
        )
    }
}
