use biome_analyze::{Ast, FixKind, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, BatchMutationExt};
use biome_turtle_syntax::{TurtleSyntaxKind, TurtleSyntaxToken, TurtleVerb};

use crate::TurtleRuleAction;

declare_lint_rule! {
    /// Suggest using the `a` shorthand for `rdf:type`.
    ///
    /// The Turtle syntax allows using `a` as a shorthand for the predicate
    /// `rdf:type` or `<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>`.
    /// Using the shorthand is more idiomatic and concise.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// <http://example.org/alice> rdf:type foaf:Person .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// <http://example.org/alice> a foaf:Person .
    /// ```
    ///
    pub UseShorthandRdfType {
        version: "next",
        name: "useShorthandRdfType",
        language: "turtle",
        recommended: true,
        severity: Severity::Information,
        fix_kind: FixKind::Safe,
    }
}

const RDF_TYPE_IRI: &str = "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>";

impl Rule for UseShorthandRdfType {
    type Query = Ast<TurtleVerb>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let node = ctx.query();

        // Check if the verb is already `a` keyword
        for token in node.syntax().children_with_tokens() {
            if let Some(token) = token.into_token() {
                if token.kind() == TurtleSyntaxKind::A_KW {
                    return None; // Already using shorthand
                }
            }
        }

        // Check if verb text matches rdf:type or the full IRI
        let text = node.syntax().text_trimmed().to_string();
        if text == "rdf:type" || text == RDF_TYPE_IRI {
            Some(())
        } else {
            None
        }
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                ctx.query().syntax().text_trimmed_range(),
                markup! {
                    "Use the "<Emphasis>"a"</Emphasis>" shorthand instead of "<Emphasis>"rdf:type"</Emphasis>"."
                },
            )
            .note(markup! {
                "The 'a' keyword is the idiomatic shorthand for rdf:type in Turtle."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<TurtleRuleAction> {
        let node = ctx.query();
        // Get the first meaningful token from the verb node
        let old_token = node
            .syntax()
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .next()?;

        let new_token =
            TurtleSyntaxToken::new_detached(TurtleSyntaxKind::A_KW, "a", [], []);

        let mut mutation = ctx.root().begin();
        mutation.replace_token_transfer_trivia(old_token, new_token);

        Some(TurtleRuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Replace with "<Emphasis>"a"</Emphasis>" shorthand." }.to_owned(),
            mutation,
        ))
    }
}
