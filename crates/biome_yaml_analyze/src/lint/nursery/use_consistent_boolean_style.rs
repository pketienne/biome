use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::AstNode;
use biome_yaml_syntax::YamlPlainScalar;

declare_lint_rule! {
    /// Enforce consistent boolean value style in YAML.
    ///
    /// YAML supports multiple representations for boolean values:
    /// `true`/`false`, `True`/`False`, `TRUE`/`FALSE`. This rule enforces
    /// the use of lowercase `true` and `false` for consistency and clarity.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// enabled: True
    /// ```
    ///
    /// ```yaml,expect_diagnostic
    /// enabled: FALSE
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// enabled: true
    /// disabled: false
    /// ```
    pub UseConsistentBooleanStyle {
        version: "next",
        name: "useConsistentBooleanStyle",
        language: "yaml",
        recommended: true,
        severity: Severity::Warning,
    }
}

/// Boolean values that should be normalized to lowercase
const BOOLEAN_VARIANTS: &[(&str, &str)] = &[
    ("True", "true"),
    ("TRUE", "true"),
    ("False", "false"),
    ("FALSE", "false"),
];

impl Rule for UseConsistentBooleanStyle {
    type Query = Ast<YamlPlainScalar>;
    type State = (&'static str, &'static str);
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let scalar = ctx.query();
        let text = scalar.value_token().ok()?;
        let value = text.text_trimmed();

        for &(variant, normalized) in BOOLEAN_VARIANTS {
            if value == variant {
                return Some((variant, normalized));
            }
        }
        None
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let node = ctx.query();
        let (variant, normalized) = state;

        Some(
            RuleDiagnostic::new(
                rule_category!(),
                node.syntax().text_trimmed_range(),
                markup! {
                    "Use lowercase "<Emphasis>{normalized}</Emphasis>" instead of "<Emphasis>{variant}</Emphasis>"."
                },
            )
            .note(markup! {
                "For consistency, always use lowercase "<Emphasis>"true"</Emphasis>" and "<Emphasis>"false"</Emphasis>" for boolean values."
            }),
        )
    }
}
