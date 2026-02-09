use crate::TurtleRuleAction;
use biome_analyze::{Ast, FixKind, Rule, RuleDiagnostic, context::RuleContext, declare_source_rule};
use biome_console::markup;
use biome_diagnostics::category;
use biome_rowan::{AstNode, AstSeparatedList, BatchMutationExt, TextRange};
use biome_turtle_syntax::{
    AnyTurtleStatement, TurtlePredicateObjectPair, TurtlePredicateObjectPairList, TurtleRoot,
    TurtleSyntaxNode, TurtleTriples, TurtleVerb,
};
use std::collections::HashMap;

declare_source_rule! {
    /// Merge triples with the same subject and predicate.
    ///
    /// Combines triples that share a subject and predicate into a single
    /// statement with comma-separated objects, and merges triples with the
    /// same subject into semicolon-separated predicate-object pairs.
    ///
    /// ## Examples
    ///
    /// ```turtle,expect_diff
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// <http://example.org/alice> foaf:knows <http://example.org/bob> .
    /// <http://example.org/alice> foaf:knows <http://example.org/carol> .
    /// ```
    ///
    pub MergeTriples {
        version: "next",
        name: "mergeTriples",
        language: "turtle",
        fix_kind: FixKind::Safe,
    }
}

pub struct MergeableTriples {
    range: TextRange,
    count: usize,
}

/// A single-predicate, single-or-multi-object triple.
struct TripleInfo {
    subject: String,
    verb: String,
    objects: Vec<String>,
    node: TurtleTriples,
}

fn collect_triple_infos(root: &TurtleRoot) -> Option<Vec<TripleInfo>> {
    let mut triple_infos: Vec<TripleInfo> = Vec::new();
    for statement in root.statements() {
        if let AnyTurtleStatement::TurtleTriples(triples) = &statement {
            let subject = triples.subject().ok()?;
            let subject_text = subject.syntax().text_trimmed().to_string();

            let predicates = triples.predicates().ok()?;
            let pairs: TurtlePredicateObjectPairList = predicates.pairs();
            let pair_list: Vec<TurtlePredicateObjectPair> = pairs
                .elements()
                .filter_map(|e| e.node().ok().cloned())
                .collect();

            // Only merge triples that have exactly one predicate-object pair
            if pair_list.len() != 1 {
                continue;
            }

            let pair = &pair_list[0];
            let verb: TurtleVerb = pair.verb().ok()?;
            let verb_text = verb.syntax().text_trimmed().to_string();

            let objects = pair.objects();
            let obj_texts: Vec<String> = objects
                .elements()
                .filter_map(|e| e.node().ok().cloned())
                .map(|o| o.syntax().text_trimmed().to_string())
                .collect();

            triple_infos.push(TripleInfo {
                subject: subject_text,
                verb: verb_text,
                objects: obj_texts,
                node: triples.clone(),
            });
        }
    }
    Some(triple_infos)
}

impl Rule for MergeTriples {
    type Query = Ast<TurtleRoot>;
    type State = MergeableTriples;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let triple_infos = collect_triple_infos(root)?;

        // Group by subject
        let mut by_subject: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, info) in triple_infos.iter().enumerate() {
            by_subject
                .entry(info.subject.clone())
                .or_default()
                .push(i);
        }

        // Check if any subject has multiple triples
        let mergeable_count: usize = by_subject
            .values()
            .filter(|indices| indices.len() > 1)
            .map(|indices| indices.len())
            .sum();

        if mergeable_count == 0 {
            return None;
        }

        let first = triple_infos.first()?.node.syntax().text_trimmed_range();
        let last = triple_infos.last()?.node.syntax().text_trimmed_range();

        Some(MergeableTriples {
            range: TextRange::new(first.start(), last.end()),
            count: mergeable_count,
        })
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                category!("assist/source/mergeTriples"),
                state.range,
                markup! { {std::format!("{} triple(s) can be merged.", state.count)} },
            )
            .note(markup! { "Merge triples with the same subject for conciseness." }),
        )
    }

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<TurtleRuleAction> {
        let root = ctx.query();
        let mut mutation = ctx.root().begin();

        let triple_infos = collect_triple_infos(root)?;

        // Group by subject, preserving order
        let mut subject_order: Vec<String> = Vec::new();
        let mut by_subject: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, info) in triple_infos.iter().enumerate() {
            let entry = by_subject.entry(info.subject.clone()).or_default();
            if entry.is_empty() {
                subject_order.push(info.subject.clone());
            }
            entry.push(i);
        }

        for subject in &subject_order {
            let indices = &by_subject[subject];
            if indices.len() < 2 {
                continue;
            }

            // Group by (subject, verb) preserving order
            let mut verb_order: Vec<String> = Vec::new();
            let mut by_verb: HashMap<String, Vec<String>> = HashMap::new();
            for &idx in indices {
                let info = &triple_infos[idx];
                let entry = by_verb.entry(info.verb.clone()).or_default();
                if entry.is_empty() {
                    verb_order.push(info.verb.clone());
                }
                entry.extend(info.objects.iter().cloned());
            }

            // Build merged triple text
            let mut parts = Vec::new();
            for verb in &verb_order {
                let objs = &by_verb[verb];
                parts.push(std::format!("{} {}", verb, objs.join(", ")));
            }
            let merged_text = if parts.len() == 1 {
                std::format!("{} {} .", subject, parts[0])
            } else {
                let joined = parts.join(" ;\n    ");
                std::format!("{}\n    {} .", subject, joined)
            };

            // Parse the merged text to get a proper AST node
            let wrapped = {
                let mut prefix_text = String::new();
                for statement in root.statements() {
                    if let AnyTurtleStatement::AnyTurtleDirective(_) = &statement {
                        prefix_text.push_str(&statement.syntax().text_trimmed().to_string());
                        prefix_text.push('\n');
                    }
                }
                std::format!("{prefix_text}{merged_text}\n")
            };
            let parsed = biome_turtle_parser::parse_turtle(&wrapped);
            let new_root = parsed.tree();

            // Find the triple statement in the parsed tree
            let mut new_triple_node: Option<TurtleSyntaxNode> = None;
            for stmt in new_root.statements() {
                if let AnyTurtleStatement::TurtleTriples(t) = &stmt {
                    new_triple_node = Some(t.syntax().clone());
                    break;
                }
            }
            let new_node = new_triple_node?;

            // Replace first triple with merged, remove the rest
            let first_idx = indices[0];
            mutation.replace_element(
                triple_infos[first_idx].node.syntax().clone().into(),
                new_node.into(),
            );

            for &idx in &indices[1..] {
                mutation.remove_node(triple_infos[idx].node.clone());
            }
        }

        Some(TurtleRuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Merge triples with the same subject." }.to_owned(),
            mutation,
        ))
    }
}
