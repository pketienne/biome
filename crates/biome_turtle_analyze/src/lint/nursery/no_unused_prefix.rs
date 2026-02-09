use biome_analyze::{FixKind, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, BatchMutationExt, TextRange};
use biome_rule_options::no_unused_prefix::NoUnusedPrefixOptions;
use biome_turtle_syntax::{AnyTurtleDirective, AnyTurtleStatement, TurtleRoot};
use std::collections::HashSet;

use crate::TurtleRuleAction;
use crate::services::semantic::Semantic;

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
    /// Use the `keepUnusedPrefixes` option to disable this rule entirely
    /// while keeping it registered (e.g., for template files that declare
    /// prefixes for documentation purposes).
    ///
    /// ```json
    /// {
    ///     "linter": {
    ///         "rules": {
    ///             "nursery": {
    ///                 "noUnusedPrefix": {
    ///                     "level": "warn",
    ///                     "options": {
    ///                         "ignoredPrefixes": ["owl:", "skos:"],
    ///                         "keepUnusedPrefixes": false
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
    type Query = Semantic<TurtleRoot>;
    type State = UnusedPrefix;
    type Signals = Vec<Self::State>;
    type Options = NoUnusedPrefixOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let model = ctx.model();
        let options = ctx.options();

        if options.keep_unused_prefixes.unwrap_or(false) {
            return Vec::new();
        }

        let ignored: HashSet<&str> = options
            .ignored_prefixes
            .as_ref()
            .map(|arr| arr.iter().map(|s| s.as_ref()).collect())
            .unwrap_or_default();

        let mut signals = Vec::new();

        for binding in model.unused_prefixes() {
            if ignored.contains(binding.namespace.as_str()) {
                continue;
            }
            // Find the directive AST node at this range for the action
            if let Some(directive) = find_prefix_directive(root, binding.range) {
                signals.push(UnusedPrefix {
                    namespace: binding.namespace.clone(),
                    range: binding.range,
                    directive,
                });
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

fn find_prefix_directive(root: &TurtleRoot, range: TextRange) -> Option<AnyTurtleDirective> {
    for statement in root.statements() {
        if let AnyTurtleStatement::AnyTurtleDirective(directive) = statement {
            if directive.syntax().text_trimmed_range() == range {
                return Some(directive);
            }
        }
    }
    None
}
