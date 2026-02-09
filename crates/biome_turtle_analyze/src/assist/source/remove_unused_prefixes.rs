use crate::TurtleRuleAction;
use biome_analyze::{Ast, FixKind, Rule, RuleDiagnostic, context::RuleContext, declare_source_rule};
use biome_console::markup;
use biome_diagnostics::category;
use biome_rowan::{AstNode, BatchMutationExt, TextRange};
use biome_turtle_syntax::{AnyTurtleDirective, AnyTurtleStatement, TurtlePrefixedName, TurtleRoot};
use std::collections::{HashMap, HashSet};

declare_source_rule! {
    /// Remove all unused prefix declarations.
    ///
    /// Scans the document for prefix declarations that are never referenced
    /// in any prefixed name and removes them all at once.
    ///
    /// ## Examples
    ///
    /// ```turtle,expect_diff
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// @prefix dc: <http://purl.org/dc/elements/1.1/> .
    /// @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
    /// <http://example.org/alice> foaf:name "Alice" .
    /// ```
    ///
    pub RemoveUnusedPrefixes {
        version: "next",
        name: "removeUnusedPrefixes",
        language: "turtle",
        fix_kind: FixKind::Safe,
    }
}

pub struct UnusedPrefixGroup {
    range: TextRange,
    count: usize,
    directives: Vec<AnyTurtleDirective>,
}

impl Rule for RemoveUnusedPrefixes {
    type Query = Ast<TurtleRoot>;
    type State = UnusedPrefixGroup;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut declared: HashMap<String, (TextRange, AnyTurtleDirective)> = HashMap::new();
        let mut used: HashSet<String> = HashSet::new();

        for statement in root.statements() {
            if let AnyTurtleStatement::AnyTurtleDirective(directive) = &statement {
                let info = match directive {
                    AnyTurtleDirective::TurtlePrefixDeclaration(decl) => {
                        decl.namespace_token().ok().map(|t| {
                            (t.text_trimmed().to_string(), directive.syntax().text_trimmed_range(), directive.clone())
                        })
                    }
                    AnyTurtleDirective::TurtleSparqlPrefixDeclaration(decl) => {
                        decl.namespace_token().ok().map(|t| {
                            (t.text_trimmed().to_string(), directive.syntax().text_trimmed_range(), directive.clone())
                        })
                    }
                    _ => None,
                };
                if let Some((ns, range, dir)) = info {
                    declared.insert(ns, (range, dir));
                }
            }
        }

        for node in root.syntax().descendants() {
            if let Some(prefixed_name) = TurtlePrefixedName::cast_ref(&node) {
                if let Ok(token) = prefixed_name.value() {
                    let text = token.text_trimmed();
                    if let Some(colon_pos) = text.find(':') {
                        used.insert(text[..=colon_pos].to_string());
                    }
                }
            }
        }

        let unused: Vec<(TextRange, AnyTurtleDirective)> = declared
            .into_iter()
            .filter(|(ns, _)| !used.contains(ns.as_str()))
            .map(|(_, v)| v)
            .collect();

        if unused.is_empty() {
            return None;
        }

        let first = unused.iter().map(|(r, _)| r.start()).min()?;
        let last = unused.iter().map(|(r, _)| r.end()).max()?;
        let directives: Vec<AnyTurtleDirective> = unused.into_iter().map(|(_, d)| d).collect();
        let count = directives.len();

        Some(UnusedPrefixGroup {
            range: TextRange::new(first, last),
            count,
            directives,
        })
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                category!("assist/source/removeUnusedPrefixes"),
                state.range,
                markup! { {std::format!("{} unused prefix declaration(s) found.", state.count)} },
            )
            .note(markup! { "Remove all unused prefix declarations." }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<TurtleRuleAction> {
        let mut mutation = ctx.root().begin();

        for directive in &state.directives {
            mutation.remove_node(directive.clone());
        }

        Some(TurtleRuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove all unused prefix declarations." }.to_owned(),
            mutation,
        ))
    }
}
