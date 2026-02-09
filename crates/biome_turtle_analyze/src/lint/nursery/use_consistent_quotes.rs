use biome_analyze::{Ast, FixKind, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{BatchMutationExt, TextRange};
use biome_turtle_syntax::{TurtleString, TurtleSyntaxToken};

use crate::TurtleRuleAction;

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
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentQuote {
    range: TextRange,
    token: TurtleSyntaxToken,
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
                token: token.clone(),
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

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<TurtleRuleAction> {
        let text = state.token.text_trimmed();
        // Replace outer single quotes with double quotes
        // 'content' -> "content"
        let inner = &text[1..text.len() - 1];
        let new_text = format!("\"{inner}\"");
        let new_token = TurtleSyntaxToken::new_detached(state.token.kind(), &new_text, [], []);

        let mut mutation = ctx.root().begin();
        mutation.replace_token_transfer_trivia(state.token.clone(), new_token);

        Some(TurtleRuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Replace with double quotes." }.to_owned(),
            mutation,
        ))
    }
}
