use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_turtle_syntax::TurtleString;
use biome_rowan::TextRange;

declare_lint_rule! {
    /// Enforce consistent use of quotes in Turtle string literals.
    ///
    /// Turtle allows both single (`'`) and double (`"`) quotes for string
    /// literals. Consistent quote usage improves readability.
    /// By default, this rule prefers double quotes.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix ex: <http://example.org/> .
    /// ex:s ex:name 'Alice' .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix ex: <http://example.org/> .
    /// ex:s ex:name "Alice" .
    /// ```
    ///
    pub UseConsistentQuotes {
        version: "next",
        name: "useConsistentQuotes",
        language: "turtle",
        recommended: false,
        severity: Severity::Information,
    }
}

pub struct InconsistentQuote {
    range: TextRange,
}

impl Rule for UseConsistentQuotes {
    type Query = Ast<TurtleString>;
    type State = InconsistentQuote;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let node = ctx.query();
        let token = node.value().ok()?;
        let text = token.text_trimmed();

        // Skip multi-line strings (triple-quoted)
        if text.starts_with("\"\"\"") || text.starts_with("'''") {
            return None;
        }

        // Flag single-quoted strings (prefer double quotes by default)
        if text.starts_with('\'') {
            Some(InconsistentQuote {
                range: token.text_trimmed_range(),
            })
        } else {
            None
        }
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Use double quotes for string literals."
                },
            )
            .note(markup! {
                "Prefer double quotes (\") over single quotes (') for consistency."
            }),
        )
    }
}
