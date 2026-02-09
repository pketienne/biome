use crate::TurtleRuleAction;
use biome_analyze::{Ast, FixKind, Rule, RuleDiagnostic, context::RuleContext, declare_source_rule};
use biome_console::markup;
use biome_diagnostics::category;
use biome_rowan::{AstNode, AstSeparatedList, BatchMutationExt, TextRange};
use biome_turtle_syntax::{
    AnyTurtleStatement, TurtlePredicateObjectPair, TurtlePredicateObjectPairList, TurtleRoot,
    TurtleVerb,
};

declare_source_rule! {
    /// Sort predicates within subject blocks alphabetically.
    ///
    /// Organizes predicate-object pairs in alphabetical order by predicate
    /// name for improved readability and consistency.
    ///
    /// ## Examples
    ///
    /// ```turtle,expect_diff
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// <http://example.org/alice>
    ///     foaf:knows <http://example.org/bob> ;
    ///     foaf:age 30 ;
    ///     foaf:name "Alice" .
    /// ```
    ///
    pub SortPredicates {
        version: "next",
        name: "sortPredicates",
        language: "turtle",
        fix_kind: FixKind::Safe,
    }
}

pub struct UnsortedPredicates {
    range: TextRange,
}

fn sort_key_for_verb(verb: &TurtleVerb) -> String {
    let text = verb.syntax().text_trimmed().to_string();
    if text == "a" {
        "rdf:type".to_lowercase()
    } else {
        text.to_lowercase()
    }
}

impl Rule for SortPredicates {
    type Query = Ast<TurtleRoot>;
    type State = UnsortedPredicates;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut has_unsorted = false;
        let mut first_range: Option<TextRange> = None;
        let mut last_range: Option<TextRange> = None;

        for statement in root.statements() {
            if let AnyTurtleStatement::TurtleTriples(triples) = &statement {
                let predicates = triples.predicates().ok()?;
                let pairs: TurtlePredicateObjectPairList = predicates.pairs();

                let mut sort_keys: Vec<String> = Vec::new();
                for element in pairs.elements() {
                    let pair = element.node().ok()?.clone();
                    let verb: TurtleVerb = pair.verb().ok()?;
                    sort_keys.push(sort_key_for_verb(&verb));
                }

                if sort_keys.len() >= 2 {
                    let is_sorted = sort_keys.windows(2).all(|w| w[0] <= w[1]);
                    if !is_sorted {
                        has_unsorted = true;
                        let range = triples.syntax().text_trimmed_range();
                        if first_range.is_none() {
                            first_range = Some(range);
                        }
                        last_range = Some(range);
                    }
                }
            }
        }

        if !has_unsorted {
            return None;
        }

        let first = first_range?;
        let last = last_range?;
        Some(UnsortedPredicates {
            range: TextRange::new(first.start(), last.end()),
        })
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                category!("assist/source/sortPredicates"),
                state.range,
                markup! { "Predicates are not sorted alphabetically within subject blocks." },
            )
            .note(markup! { "Sort predicates for consistency and readability." }),
        )
    }

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<TurtleRuleAction> {
        let root = ctx.query();
        let mut mutation = ctx.root().begin();

        for statement in root.statements() {
            if let AnyTurtleStatement::TurtleTriples(triples) = &statement {
                let predicates = triples.predicates().ok()?;
                let pairs: TurtlePredicateObjectPairList = predicates.pairs();

                let mut keyed_pairs: Vec<(String, TurtlePredicateObjectPair)> = Vec::new();
                for element in pairs.elements() {
                    let pair = element.node().ok()?.clone();
                    let verb: TurtleVerb = pair.verb().ok()?;
                    keyed_pairs.push((sort_key_for_verb(&verb), pair));
                }

                if keyed_pairs.len() < 2 {
                    continue;
                }

                let is_sorted = keyed_pairs.windows(2).all(|w| w[0].0 <= w[1].0);
                if is_sorted {
                    continue;
                }

                // Save original order
                let original_nodes: Vec<_> =
                    keyed_pairs.iter().map(|(_, p)| p.clone()).collect();

                // Sort by predicate text
                let mut sorted = keyed_pairs;
                sorted.sort_by(|a, b| a.0.cmp(&b.0));

                // Replace each original position with the sorted pair
                for (i, original) in original_nodes.iter().enumerate() {
                    let sorted_pair = &sorted[i].1;
                    if original.syntax().text_trimmed() != sorted_pair.syntax().text_trimmed() {
                        mutation.replace_element(
                            original.syntax().clone().into(),
                            sorted_pair.syntax().clone().into(),
                        );
                    }
                }
            }
        }

        Some(TurtleRuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Sort predicates alphabetically." }.to_owned(),
            mutation,
        ))
    }
}
