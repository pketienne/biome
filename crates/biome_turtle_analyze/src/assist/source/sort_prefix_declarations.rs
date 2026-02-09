use crate::TurtleRuleAction;
use biome_analyze::{Ast, FixKind, Rule, RuleDiagnostic, context::RuleContext, declare_source_rule};
use biome_console::markup;
use biome_diagnostics::category;
use biome_rowan::{AstNode, BatchMutationExt, TextRange};
use biome_turtle_syntax::{AnyTurtleDirective, AnyTurtleStatement, TurtleRoot};

declare_source_rule! {
    /// Sort prefix declarations alphabetically.
    ///
    /// Organizes `@prefix` declarations in alphabetical order by namespace
    /// for improved readability and consistency.
    ///
    /// ## Examples
    ///
    /// ```turtle,expect_diff
    /// @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
    /// ```
    ///
    pub SortPrefixDeclarations {
        version: "next",
        name: "sortPrefixDeclarations",
        language: "turtle",
        fix_kind: FixKind::Safe,
    }
}

pub struct UnsortedPrefixes {
    range: TextRange,
}

impl Rule for SortPrefixDeclarations {
    type Query = Ast<TurtleRoot>;
    type State = UnsortedPrefixes;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut prefixes: Vec<(String, TextRange)> = Vec::new();

        for statement in root.statements() {
            match &statement {
                AnyTurtleStatement::AnyTurtleDirective(AnyTurtleDirective::TurtlePrefixDeclaration(decl)) => {
                    if let Ok(ns) = decl.namespace_token() {
                        prefixes.push((
                            ns.text_trimmed().to_lowercase(),
                            decl.syntax().text_trimmed_range(),
                        ));
                    }
                }
                AnyTurtleStatement::AnyTurtleDirective(AnyTurtleDirective::TurtleSparqlPrefixDeclaration(decl)) => {
                    if let Ok(ns) = decl.namespace_token() {
                        prefixes.push((
                            ns.text_trimmed().to_lowercase(),
                            decl.syntax().text_trimmed_range(),
                        ));
                    }
                }
                _ => {}
            }
        }

        if prefixes.len() < 2 {
            return None;
        }

        // Check if already sorted
        let is_sorted = prefixes.windows(2).all(|w| w[0].0 <= w[1].0);
        if is_sorted {
            return None;
        }

        // Return the range of the first out-of-order prefix
        let first_range = prefixes.first()?.1;
        let last_range = prefixes.last()?.1;
        let combined_range = TextRange::new(first_range.start(), last_range.end());

        Some(UnsortedPrefixes {
            range: combined_range,
        })
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                category!("assist/source/sortPrefixDeclarations"),
                state.range,
                markup! { "Prefix declarations are not sorted alphabetically." },
            )
            .note(markup! { "Sort prefix declarations for consistency." }),
        )
    }

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<TurtleRuleAction> {
        let root = ctx.query();
        let mut mutation = ctx.root().begin();

        // Collect all prefix directive nodes with their namespace text
        let mut prefix_directives: Vec<(String, AnyTurtleDirective)> = Vec::new();

        for statement in root.statements() {
            match &statement {
                AnyTurtleStatement::AnyTurtleDirective(dir @ AnyTurtleDirective::TurtlePrefixDeclaration(decl)) => {
                    if let Ok(ns) = decl.namespace_token() {
                        prefix_directives.push((ns.text_trimmed().to_lowercase(), dir.clone()));
                    }
                }
                AnyTurtleStatement::AnyTurtleDirective(dir @ AnyTurtleDirective::TurtleSparqlPrefixDeclaration(decl)) => {
                    if let Ok(ns) = decl.namespace_token() {
                        prefix_directives.push((ns.text_trimmed().to_lowercase(), dir.clone()));
                    }
                }
                _ => {}
            }
        }

        if prefix_directives.len() < 2 {
            return None;
        }

        // Save original order
        let original_nodes: Vec<AnyTurtleDirective> =
            prefix_directives.iter().map(|(_, d)| d.clone()).collect();

        // Sort by namespace
        let mut sorted = prefix_directives;
        sorted.sort_by(|a, b| a.0.cmp(&b.0));

        // Replace each original position with the sorted directive
        for (i, original) in original_nodes.iter().enumerate() {
            let sorted_directive = &sorted[i].1;
            if original.syntax().text_trimmed() != sorted_directive.syntax().text_trimmed() {
                mutation.replace_element(
                    original.syntax().clone().into(),
                    sorted_directive.syntax().clone().into(),
                );
            }
        }

        Some(TurtleRuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Sort prefix declarations alphabetically." }.to_owned(),
            mutation,
        ))
    }
}
