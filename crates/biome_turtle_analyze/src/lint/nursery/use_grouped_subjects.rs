use biome_analyze::{Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::TextRange;
use biome_turtle_syntax::TurtleRoot;
use std::collections::{HashMap, HashSet};

use crate::services::semantic::Semantic;

declare_lint_rule! {
    /// Suggest grouping triples with the same subject.
    ///
    /// When the same subject appears in multiple separate triple blocks,
    /// they can usually be merged into a single block using `;` to separate
    /// predicates. This makes the document more concise and readable.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix ex: <http://example.org/> .
    /// ex:alice ex:name "Alice" .
    /// ex:alice ex:age "30" .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix ex: <http://example.org/> .
    /// ex:alice ex:name "Alice" ;
    ///     ex:age "30" .
    /// ```
    ///
    pub UseGroupedSubjects {
        version: "next",
        name: "useGroupedSubjects",
        language: "turtle",
        recommended: false,
        severity: Severity::Information,
    }
}

pub struct UngroupedSubject {
    range: TextRange,
    subject: String,
}

impl Rule for UseGroupedSubjects {
    type Query = Semantic<TurtleRoot>;
    type State = UngroupedSubject;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let model = ctx.model();
        let triples = model.triples();
        let mut signals = Vec::new();
        // Track first seen statement_range per subject
        let mut seen_subjects: HashMap<&str, TextRange> = HashMap::new();
        // Track which statement_ranges we've already reported
        let mut reported: HashSet<TextRange> = HashSet::new();

        for triple in triples {
            let subject = triple.subject.as_str();
            let range = triple.statement_range;

            match seen_subjects.get(subject) {
                Some(first_range) if *first_range != range => {
                    // Same subject in a different TurtleTriples block
                    if reported.insert(range) {
                        signals.push(UngroupedSubject {
                            range,
                            subject: triple.subject.clone(),
                        });
                    }
                }
                None => {
                    seen_subjects.insert(subject, range);
                }
                _ => {
                    // Same subject, same block — skip (using ; notation)
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
                    "Subject '"{ &state.subject }"' appears in multiple triple blocks."
                },
            )
            .note(markup! {
                "Consider merging triples with the same subject using ';' to separate predicates."
            }),
        )
    }
}
