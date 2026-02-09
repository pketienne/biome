use biome_analyze::{Ast, FixKind, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, BatchMutationExt, TextRange};
use biome_rule_options::no_unused_prefix::NoUnusedPrefixOptions;
use biome_turtle_syntax::{AnyTurtleDirective, AnyTurtleStatement, TurtlePrefixedName, TurtleRoot};
use std::collections::{HashMap, HashSet};

use crate::TurtleRuleAction;

declare_lint_rule! {
    /// Disallow unused prefix declarations in Turtle documents.
    ///
    /// Prefix declarations that are never used add unnecessary clutter to the
    /// document and may indicate leftover code from refactoring.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// @prefix dc: <http://purl.org/dc/elements/1.1/> .
    /// <http://example.org/alice> foaf:name "Alice" .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// <http://example.org/alice> foaf:name "Alice" .
    /// ```
    ///
    /// ## Options
    ///
    /// Use the `ignoredPrefixes` option to whitelist prefix namespaces that
    /// should not trigger this rule even when unused.
    ///
    /// ```json
    /// {
    ///     "linter": {
    ///         "rules": {
    ///             "nursery": {
    ///                 "noUnusedPrefix": {
    ///                     "level": "warn",
    ///                     "options": {
    ///                         "ignoredPrefixes": ["owl:", "skos:"]
    ///                     }
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    ///
    pub NoUnusedPrefix {
        version: "next",
        name: "noUnusedPrefix",
        language: "turtle",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct UnusedPrefix {
    namespace: String,
    range: TextRange,
    directive: AnyTurtleDirective,
}

impl Rule for NoUnusedPrefix {
    type Query = Ast<TurtleRoot>;
    type State = UnusedPrefix;
    type Signals = Vec<Self::State>;
    type Options = NoUnusedPrefixOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let options = ctx.options();
        let ignored: HashSet<&str> = options
            .ignored_prefixes
            .as_ref()
            .map(|arr| arr.iter().map(|s| s.as_ref()).collect())
            .unwrap_or_default();
        let mut declared: HashMap<String, (TextRange, AnyTurtleDirective)> = HashMap::new();
        let mut used: HashSet<String> = HashSet::new();

        // Collect all declared prefixes
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

        // Collect all used prefixes
        for node in root.syntax().descendants() {
            if let Some(prefixed_name) = TurtlePrefixedName::cast_ref(&node) {
                if let Ok(token) = prefixed_name.value() {
                    let text = token.text_trimmed();
                    if let Some(colon_pos) = text.find(':') {
                        let prefix = &text[..=colon_pos];
                        used.insert(prefix.to_string());
                    }
                }
            }
        }

        // Report unused (skip ignored prefixes)
        declared
            .into_iter()
            .filter(|(ns, _)| !used.contains(ns.as_str()) && !ignored.contains(ns.as_str()))
            .map(|(namespace, (range, directive))| UnusedPrefix { namespace, range, directive })
            .collect()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Prefix '"{ &state.namespace }"' is declared but never used."
                },
            )
            .note(markup! {
                "Remove the unused prefix declaration."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<TurtleRuleAction> {
        let mut mutation = ctx.root().begin();
        mutation.remove_node(state.directive.clone());

        Some(TurtleRuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove the unused prefix declaration." }.to_owned(),
            mutation,
        ))
    }
}
