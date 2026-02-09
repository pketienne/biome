use crate::TurtleRuleAction;
use biome_analyze::{Ast, FixKind, Rule, RuleDiagnostic, context::RuleContext, declare_source_rule};
use biome_console::markup;
use biome_diagnostics::category;
use biome_rowan::{AstNode, BatchMutationExt, TextRange};
use biome_turtle_syntax::{AnyTurtleStatement, TurtleRoot, TurtleTriples};

declare_source_rule! {
    /// Sort triple statements by subject.
    ///
    /// Organizes triple statements in alphabetical order by their subject
    /// for improved document structure and readability.
    ///
    /// ## Examples
    ///
    /// ```turtle,expect_diff
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// <http://example.org/carol> foaf:name "Carol" .
    /// <http://example.org/alice> foaf:name "Alice" .
    /// <http://example.org/bob> foaf:name "Bob" .
    /// ```
    ///
    pub SortTriples {
        version: "next",
        name: "sortTriples",
        language: "turtle",
        fix_kind: FixKind::Safe,
    }
}

pub struct UnsortedTriples {
    range: TextRange,
}

impl Rule for SortTriples {
    type Query = Ast<TurtleRoot>;
    type State = UnsortedTriples;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut triples: Vec<(String, TextRange)> = Vec::new();

        for statement in root.statements() {
            if let AnyTurtleStatement::TurtleTriples(triple) = &statement {
                if let Ok(subject) = triple.subject() {
                    triples.push((
                        subject.syntax().text_trimmed().to_string().to_lowercase(),
                        triple.syntax().text_trimmed_range(),
                    ));
                }
            }
        }

        if triples.len() < 2 {
            return None;
        }

        let is_sorted = triples.windows(2).all(|w| w[0].0 <= w[1].0);
        if is_sorted {
            return None;
        }

        let first_range = triples.first()?.1;
        let last_range = triples.last()?.1;
        Some(UnsortedTriples {
            range: TextRange::new(first_range.start(), last_range.end()),
        })
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                category!("assist/source/sortTriples"),
                state.range,
                markup! { "Triple statements are not sorted by subject." },
            )
            .note(markup! { "Sort triples by subject for consistency." }),
        )
    }

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<TurtleRuleAction> {
        let root = ctx.query();
        let mut mutation = ctx.root().begin();

        let mut keyed_triples: Vec<(String, TurtleTriples)> = Vec::new();
        for statement in root.statements() {
            if let AnyTurtleStatement::TurtleTriples(triple) = &statement {
                if let Ok(subject) = triple.subject() {
                    keyed_triples.push((
                        subject.syntax().text_trimmed().to_string().to_lowercase(),
                        triple.clone(),
                    ));
                }
            }
        }

        if keyed_triples.len() < 2 {
            return None;
        }

        let original_nodes: Vec<TurtleTriples> =
            keyed_triples.iter().map(|(_, t)| t.clone()).collect();

        let mut sorted = keyed_triples;
        sorted.sort_by(|a, b| a.0.cmp(&b.0));

        for (i, original) in original_nodes.iter().enumerate() {
            let sorted_triple = &sorted[i].1;
            if original.syntax().text_trimmed() != sorted_triple.syntax().text_trimmed() {
                mutation.replace_element(
                    original.syntax().clone().into(),
                    sorted_triple.syntax().clone().into(),
                );
            }
        }

        Some(TurtleRuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Sort triples by subject." }.to_owned(),
            mutation,
        ))
    }
}
