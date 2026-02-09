use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, AstSeparatedList, TextRange};
use biome_turtle_syntax::{
    AnyTurtleStatement, TurtleObjectList, TurtlePredicateObjectPairList, TurtleRoot, TurtleTriples,
    TurtleVerb,
};
use std::collections::HashSet;

declare_lint_rule! {
    /// Disallow duplicate triples in Turtle documents.
    ///
    /// Duplicate subject-predicate-object triples are redundant in RDF.
    /// They add no new information and may indicate copy-paste errors.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix ex: <http://example.org/> .
    /// ex:alice ex:name "Alice" .
    /// ex:alice ex:name "Alice" .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix ex: <http://example.org/> .
    /// ex:alice ex:name "Alice" .
    /// ex:bob ex:name "Bob" .
    /// ```
    ///
    pub NoDuplicateTriple {
        version: "next",
        name: "noDuplicateTriple",
        language: "turtle",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct DuplicateTriple {
    range: TextRange,
    triple_text: String,
}

fn extract_triples(triples: &TurtleTriples) -> Vec<(String, String, String)> {
    let mut result = Vec::new();

    let subject = triples
        .subject()
        .map(|s| s.syntax().text_trimmed().to_string())
        .unwrap_or_default();

    if let Ok(predicates) = triples.predicates() {
        let pair_list: TurtlePredicateObjectPairList = predicates.pairs();
        for element in pair_list.elements() {
            if let Ok(pair) = element.node() {
                let verb = pair
                    .verb()
                    .map(|v: TurtleVerb| v.syntax().text_trimmed().to_string())
                    .unwrap_or_default();

                let obj_list: TurtleObjectList = pair.objects();
                for obj_element in obj_list.elements() {
                    if let Ok(obj) = obj_element.node() {
                        let object = obj.syntax().text_trimmed().to_string();
                        result.push((subject.clone(), verb.clone(), object));
                    }
                }
            }
        }
    }

    result
}

impl Rule for NoDuplicateTriple {
    type Query = Ast<TurtleRoot>;
    type State = DuplicateTriple;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut seen: HashSet<(String, String, String)> = HashSet::new();
        let mut signals = Vec::new();

        for statement in root.statements() {
            if let AnyTurtleStatement::TurtleTriples(triples) = &statement {
                for triple in extract_triples(triples) {
                    let triple_text =
                        format!("{} {} {}", triple.0, triple.1, triple.2);
                    if !seen.insert(triple.clone()) {
                        signals.push(DuplicateTriple {
                            range: triples.syntax().text_trimmed_range(),
                            triple_text,
                        });
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
                    "Duplicate triple: "{ &state.triple_text }"."
                },
            )
            .note(markup! {
                "Remove the duplicate triple."
            }),
        )
    }
}
