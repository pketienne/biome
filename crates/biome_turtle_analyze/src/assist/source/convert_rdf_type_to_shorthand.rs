use crate::TurtleRuleAction;
use biome_analyze::{Ast, FixKind, Rule, RuleDiagnostic, context::RuleContext, declare_source_rule};
use biome_console::markup;
use biome_diagnostics::category;
use biome_rowan::{AstNode, BatchMutationExt, TextRange};
use biome_turtle_syntax::{
    AnyTurtleStatement, TurtleRoot, TurtleSyntaxKind, TurtleSyntaxToken, TurtleVerb,
};

declare_source_rule! {
    /// Convert all `rdf:type` usages to the `a` shorthand.
    ///
    /// Replaces every `rdf:type` or
    /// `<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>` verb with the
    /// idiomatic `a` keyword in a single action.
    ///
    /// ## Examples
    ///
    /// ```turtle,expect_diff
    /// @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// <http://example.org/alice> rdf:type foaf:Person .
    /// <http://example.org/bob> rdf:type foaf:Person .
    /// ```
    ///
    pub ConvertRdfTypeToShorthand {
        version: "next",
        name: "convertRdfTypeToShorthand",
        language: "turtle",
        fix_kind: FixKind::Safe,
    }
}

const RDF_TYPE_PREFIXED: &str = "rdf:type";
const RDF_TYPE_IRI: &str = "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>";

pub struct RdfTypeUsages {
    range: TextRange,
    count: usize,
    tokens: Vec<TurtleSyntaxToken>,
}

impl Rule for ConvertRdfTypeToShorthand {
    type Query = Ast<TurtleRoot>;
    type State = RdfTypeUsages;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut tokens = Vec::new();

        for statement in root.statements() {
            if let AnyTurtleStatement::TurtleTriples(triples) = &statement {
                // Check all verb nodes in this triple
                for node in triples.syntax().descendants() {
                    if let Some(verb) = TurtleVerb::cast_ref(&node) {
                        // Skip if already using `a`
                        let has_a = verb.syntax().children_with_tokens().any(|el| {
                            el.into_token()
                                .is_some_and(|t| t.kind() == TurtleSyntaxKind::A_KW)
                        });
                        if has_a {
                            continue;
                        }

                        let text = verb.syntax().text_trimmed().to_string();
                        if text == RDF_TYPE_PREFIXED || text == RDF_TYPE_IRI {
                            // Get the first meaningful token from the verb
                            if let Some(token) = verb
                                .syntax()
                                .children_with_tokens()
                                .filter_map(|el| el.into_token())
                                .next()
                            {
                                tokens.push(token);
                            }
                        }
                    }
                }
            }
        }

        if tokens.is_empty() {
            return None;
        }

        let first = tokens.first()?.text_trimmed_range();
        let last = tokens.last()?.text_trimmed_range();
        let count = tokens.len();

        Some(RdfTypeUsages {
            range: TextRange::new(first.start(), last.end()),
            count,
            tokens,
        })
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                category!("assist/source/convertRdfTypeToShorthand"),
                state.range,
                markup! { {std::format!("{} rdf:type usage(s) can be replaced with 'a'.", state.count)} },
            )
            .note(markup! { "The 'a' keyword is the idiomatic shorthand for rdf:type." }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<TurtleRuleAction> {
        let mut mutation = ctx.root().begin();

        for old_token in &state.tokens {
            let new_token =
                TurtleSyntaxToken::new_detached(TurtleSyntaxKind::A_KW, "a", [], []);
            mutation.replace_token_transfer_trivia(old_token.clone(), new_token);
        }

        Some(TurtleRuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Replace all rdf:type with 'a' shorthand." }.to_owned(),
            mutation,
        ))
    }
}
