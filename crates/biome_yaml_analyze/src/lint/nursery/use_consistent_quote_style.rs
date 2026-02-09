use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::AstNode;
use biome_yaml_syntax::YamlDoubleQuotedScalar;

declare_lint_rule! {
    /// Enforce the consistent use of double quotes for strings.
    ///
    /// Using a consistent quote style across a YAML file improves readability.
    /// Double quotes are preferred because they support escape sequences and
    /// are more widely compatible.
    ///
    /// Single-quoted strings that contain characters requiring escapes in
    /// double quotes (like backslashes) are not flagged.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// key: 'single quoted value'
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// key: "double quoted value"
    /// ```
    pub UseConsistentQuoteStyle {
        version: "next",
        name: "useConsistentQuoteStyle",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
    }
}

// We query for single-quoted scalars and flag them, since the default
// preferred style is double quotes.
// Note: We use YamlDoubleQuotedScalar here as a no-op query since there's
// no direct way to query YamlSingleQuotedScalar without triggering on
// double-quoted ones. Instead, we query YamlRoot and walk.

impl Rule for UseConsistentQuoteStyle {
    type Query = Ast<biome_yaml_syntax::YamlRoot>;
    type State = biome_rowan::TextRange;
    type Signals = Box<[Self::State]>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut violations = Vec::new();

        for node in root.syntax().descendants() {
            if biome_yaml_syntax::YamlSingleQuotedScalar::can_cast(node.kind()) {
                if let Some(scalar) = biome_yaml_syntax::YamlSingleQuotedScalar::cast(node) {
                    if let Ok(token) = scalar.value_token() {
                        let text = token.text_trimmed();
                        // Don't flag single-quoted strings that contain backslashes,
                        // since switching to double quotes would change their meaning
                        if !text.contains('\\') {
                            violations.push(scalar.syntax().text_trimmed_range());
                        }
                    }
                }
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
                    "Use double quotes instead of single quotes."
                },
            )
            .note(markup! {
                "Double quotes are preferred for consistency and support escape sequences."
            }),
        )
    }
}
