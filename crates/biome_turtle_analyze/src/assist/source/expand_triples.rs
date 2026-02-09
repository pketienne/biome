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
    /// Expand compound triples into one-triple-per-line form.
    ///
    /// Converts triples with multiple predicates (`;`) or multiple objects (`,`)
    /// into individual single-statement triples, optimized for clean VCS diffs.
    ///
    /// ## Examples
    ///
    /// ```turtle,expect_diff
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// <http://example.org/alice>
    ///     foaf:name "Alice" ;
    ///     foaf:knows <http://example.org/bob> .
    /// ```
    ///
    pub ExpandTriples {
        version: "next",
        name: "expandTriples",
        language: "turtle",
        fix_kind: FixKind::Safe,
    }
}

pub struct ExpandableTriples {
    range: TextRange,
    count: usize,
    expanded_text: String,
}

impl Rule for ExpandTriples {
    type Query = Ast<TurtleRoot>;
    type State = ExpandableTriples;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut expandable_count = 0;
        let mut first_range: Option<TextRange> = None;
        let mut last_range: Option<TextRange> = None;

        // Build the expanded document text
        let mut output_lines: Vec<String> = Vec::new();
        let mut had_expansion = false;

        for statement in root.statements() {
            match &statement {
                AnyTurtleStatement::TurtleTriples(triples) => {
                    let subject = triples.subject().ok()?;
                    let subject_text = subject.syntax().text_trimmed().to_string();
                    let predicates = triples.predicates().ok()?;
                    let pairs: TurtlePredicateObjectPairList = predicates.pairs();
                    let pair_list: Vec<TurtlePredicateObjectPair> = pairs
                        .elements()
                        .filter_map(|e| e.node().ok().cloned())
                        .collect();

                    // Check if this triple needs expansion
                    let mut is_compound = pair_list.len() > 1;
                    if !is_compound {
                        for pair in &pair_list {
                            let objects = pair.objects();
                            if objects.elements().filter_map(|e| e.node().ok().cloned()).count() > 1 {
                                is_compound = true;
                                break;
                            }
                        }
                    }

                    if is_compound {
                        expandable_count += 1;
                        had_expansion = true;
                        let range = triples.syntax().text_trimmed_range();
                        if first_range.is_none() {
                            first_range = Some(range);
                        }
                        last_range = Some(range);

                        // Expand this triple
                        for pair in &pair_list {
                            let verb: TurtleVerb = pair.verb().ok()?;
                            let verb_text = verb.syntax().text_trimmed().to_string();
                            let objects = pair.objects();
                            for obj_element in objects.elements() {
                                if let Ok(obj) = obj_element.node() {
                                    let obj_text = obj.syntax().text_trimmed().to_string();
                                    output_lines.push(std::format!(
                                        "{subject_text} {verb_text} {obj_text} ."
                                    ));
                                }
                            }
                        }
                    } else {
                        output_lines.push(statement.syntax().text_trimmed().to_string());
                    }
                }
                AnyTurtleStatement::AnyTurtleDirective(_) => {
                    output_lines.push(statement.syntax().text_trimmed().to_string());
                }
                _ => {}
            }
        }

        if !had_expansion {
            return None;
        }

        let first = first_range?;
        let last = last_range?;
        let expanded_text = output_lines.join("\n") + "\n";

        Some(ExpandableTriples {
            range: TextRange::new(first.start(), last.end()),
            count: expandable_count,
            expanded_text,
        })
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                category!("assist/source/expandTriples"),
                state.range,
                markup! { {std::format!("{} compound triple(s) can be expanded.", state.count)} },
            )
            .note(markup! { "Expand to one triple per line for cleaner diffs." }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<TurtleRuleAction> {
        let mut mutation = ctx.root().begin();

        // Parse the expanded text to get a new root
        let parsed = biome_turtle_parser::parse_turtle(&state.expanded_text);
        let new_root = parsed.tree();

        // Replace the entire root with the new one
        mutation.replace_element(
            ctx.root().syntax().clone().into(),
            new_root.syntax().clone().into(),
        );

        Some(TurtleRuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Expand compound triples to one per line." }.to_owned(),
            mutation,
        ))
    }
}
