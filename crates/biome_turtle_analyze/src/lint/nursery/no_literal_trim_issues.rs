use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_turtle_syntax::TurtleRdfLiteral;
use biome_rowan::TextRange;

declare_lint_rule! {
    /// Warn about string literals with leading or trailing whitespace.
    ///
    /// String literals with unexpected leading or trailing whitespace
    /// may indicate data quality issues. This whitespace is part of the
    /// literal value and could cause problems in data processing.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix ex: <http://example.org/> .
    /// ex:s ex:name " Alice " .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix ex: <http://example.org/> .
    /// ex:s ex:name "Alice" .
    /// ```
    ///
    pub NoLiteralTrimIssues {
        version: "next",
        name: "noLiteralTrimIssues",
        language: "turtle",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct TrimIssue {
    range: TextRange,
    has_leading: bool,
    has_trailing: bool,
}

impl Rule for NoLiteralTrimIssues {
    type Query = Ast<TurtleRdfLiteral>;
    type State = TrimIssue;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let node = ctx.query();
        let string_node = node.value().ok()?;
        let token = string_node.value().ok()?;
        let text = token.text_trimmed();

        // Determine quote style and extract inner content
        let inner = if text.starts_with("\"\"\"") && text.ends_with("\"\"\"") {
            &text[3..text.len() - 3]
        } else if text.starts_with("'''") && text.ends_with("'''") {
            &text[3..text.len() - 3]
        } else if text.starts_with('"') && text.ends_with('"') {
            &text[1..text.len() - 1]
        } else if text.starts_with('\'') && text.ends_with('\'') {
            &text[1..text.len() - 1]
        } else {
            return None;
        };

        if inner.is_empty() {
            return None;
        }

        let has_leading = inner.starts_with(' ') || inner.starts_with('\t');
        let has_trailing = inner.ends_with(' ') || inner.ends_with('\t');

        if has_leading || has_trailing {
            Some(TrimIssue {
                range: token.text_trimmed_range(),
                has_leading,
                has_trailing,
            })
        } else {
            None
        }
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let message = match (state.has_leading, state.has_trailing) {
            (true, true) => "String literal has leading and trailing whitespace.",
            (true, false) => "String literal has leading whitespace.",
            (false, true) => "String literal has trailing whitespace.",
            _ => return None,
        };

        Some(
            RuleDiagnostic::new(rule_category!(), state.range, markup! { {message} }).note(
                markup! {
                    "This whitespace is part of the literal value and may be unintentional."
                },
            ),
        )
    }
}
