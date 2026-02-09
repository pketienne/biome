use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, Direction, TextRange};
use biome_turtle_syntax::{TurtleRoot, TurtleSyntaxKind};

declare_lint_rule! {
    /// Disallow prefixed names with an empty local part.
    ///
    /// A prefixed name like `ex:` (with no local name after the colon) is
    /// valid Turtle syntax but often indicates a mistake — the author likely
    /// forgot to type the local name. This rule flags such occurrences
    /// in subject, predicate, or object positions (not in `@prefix` declarations
    /// where the bare namespace is expected).
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix ex: <http://example.org/> .
    /// ex:alice ex:knows ex: .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix ex: <http://example.org/> .
    /// ex:alice ex:knows ex:bob .
    /// ```
    ///
    pub NoEmptyPrefixedName {
        version: "next",
        name: "noEmptyPrefixedName",
        language: "turtle",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct EmptyPrefixedName {
    range: TextRange,
    namespace: String,
}

impl Rule for NoEmptyPrefixedName {
    type Query = Ast<TurtleRoot>;
    type State = EmptyPrefixedName;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut signals = Vec::new();

        for token in root.syntax().descendants_with_tokens(Direction::Next) {
            if let Some(token) = token.into_token() {
                if token.kind() == TurtleSyntaxKind::TURTLE_PNAME_NS_LITERAL {
                    // PNAME_NS tokens appear in both prefix declarations and
                    // prefixed names. We only want to flag usage in triples,
                    // not in @prefix declarations.
                    if let Some(parent) = token.parent() {
                        if parent.kind() == TurtleSyntaxKind::TURTLE_PREFIXED_NAME {
                            // This is a bare prefix used as a name (e.g., ex:)
                            let text = token.text_trimmed().to_string();
                            signals.push(EmptyPrefixedName {
                                range: token.text_trimmed_range(),
                                namespace: text,
                            });
                        }
                    }
                }
            }
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Prefixed name '"{ &state.namespace }"' has no local name."
                },
            )
            .note(markup! {
                "A bare prefix without a local name is usually a mistake. Add a local name after the colon."
            }),
        )
    }
}
