use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange};
use biome_turtle_syntax::{AnyTurtleStatement, TurtleRoot, TurtleTriples};
use std::collections::HashMap;

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
    type Query = Ast<TurtleRoot>;
    type State = UngroupedSubject;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut signals = Vec::new();
        let mut seen_subjects: HashMap<String, TextRange> = HashMap::new();

        for statement in root.statements() {
            if let AnyTurtleStatement::TurtleTriples(triples) = &statement {
                let subject_text = extract_subject_text(triples);
                if let Some(subject) = subject_text {
                    if let Some(_first_range) = seen_subjects.get(&subject) {
                        signals.push(UngroupedSubject {
                            range: triples.syntax().text_trimmed_range(),
                            subject: subject.clone(),
                        });
                    } else {
                        seen_subjects
                            .insert(subject, triples.syntax().text_trimmed_range());
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
                    "Subject '"{ &state.subject }"' appears in multiple triple blocks."
                },
            )
            .note(markup! {
                "Consider merging triples with the same subject using ';' to separate predicates."
            }),
        )
    }
}

fn extract_subject_text(triples: &TurtleTriples) -> Option<String> {
    // The subject is the first child of TurtleTriples
    let subject = triples.subject().ok()?;
    Some(subject.syntax().text_trimmed().to_string())
}
