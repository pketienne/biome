use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::AstNode;
use biome_yaml_syntax::YamlPlainScalar;

declare_lint_rule! {
    /// Disallow trailing zeros in floating-point numbers.
    ///
    /// Trailing zeros in float values (e.g., `1.0`, `2.50`) add no information
    /// and can be simplified. This rule flags floats with unnecessary trailing
    /// zeros after the decimal point.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```yaml,expect_diagnostic
    /// value: 1.0
    /// ```
    ///
    /// ### Valid
    ///
    /// ```yaml
    /// value: 1
    /// ```
    pub NoFloatTrailingZeros {
        version: "next",
        name: "noFloatTrailingZeros",
        language: "yaml",
        recommended: false,
        severity: Severity::Warning,
    }
}

impl Rule for NoFloatTrailingZeros {
    type Query = Ast<YamlPlainScalar>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let scalar = ctx.query();
        let token = scalar.value_token().ok()?;
        let text = token.text_trimmed().trim();

        // Must contain a decimal point to be a float
        if !text.contains('.') {
            return None;
        }

        // Split on decimal point
        let parts: Vec<&str> = text.splitn(2, '.').collect();
        if parts.len() != 2 {
            return None;
        }

        let integer_part = parts[0];
        let decimal_part = parts[1];

        // Verify integer part is numeric (allow optional leading sign)
        let int_digits = integer_part.strip_prefix(['+', '-']).unwrap_or(integer_part);
        if int_digits.is_empty() || !int_digits.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }

        // Strip any scientific notation suffix before checking trailing zeros
        let (frac, _) = decimal_part
            .split_once(['e', 'E'])
            .unwrap_or((decimal_part, ""));

        // Check if fractional part has trailing zeros
        if !frac.is_empty() && frac.chars().all(|c| c.is_ascii_digit()) && frac.ends_with('0') {
            return Some(());
        }

        None
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        let scalar = ctx.query();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                scalar.syntax().text_trimmed_range(),
                markup! {
                    "Float value has unnecessary trailing zeros."
                },
            )
            .note(markup! {
                "Remove trailing zeros from the decimal portion of the number."
            }),
        )
    }
}
