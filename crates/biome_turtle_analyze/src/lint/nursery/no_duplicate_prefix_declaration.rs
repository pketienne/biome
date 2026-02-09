use biome_analyze::{FixKind, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, BatchMutationExt, TextRange};
use biome_turtle_syntax::{AnyTurtleDirective, AnyTurtleStatement, TurtleRoot};

use crate::TurtleRuleAction;
use crate::services::semantic::Semantic;

declare_lint_rule! {
    /// Disallow duplicate prefix declarations in Turtle documents.
    ///
    /// Having multiple prefix declarations for the same namespace prefix is
    /// confusing and error-prone. Only the last declaration will take effect,
    /// which can lead to unexpected behavior.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// @prefix foaf: <http://example.org/foaf/> .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    /// @prefix dc: <http://purl.org/dc/elements/1.1/> .
    /// ```
    ///
    pub NoDuplicatePrefixDeclaration {
        version: "next",
        name: "noDuplicatePrefixDeclaration",
        language: "turtle",
        recommended: true,
        severity: Severity::Error,
        fix_kind: FixKind::Safe,
    }
}

pub struct DuplicatePrefix {
    namespace: String,
    range: TextRange,
    directive: AnyTurtleDirective,
}

impl Rule for NoDuplicatePrefixDeclaration {
    type Query = Semantic<TurtleRoot>;
    type State = DuplicatePrefix;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let model = ctx.model();
        let mut signals = Vec::new();

        for binding in model.duplicate_prefixes() {
            // Find the directive AST node at this range for the action
            let directive = find_prefix_directive(root, binding.range);
            if let Some(directive) = directive {
                signals.push(DuplicatePrefix {
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
                    "Duplicate prefix declaration for '"{ &state.namespace }"'."
                },
            )
            .note(markup! {
                "Only the last declaration takes effect. Remove the duplicate."
            }),
        )
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<TurtleRuleAction> {
        let mut mutation = ctx.root().begin();
        mutation.remove_node(state.directive.clone());

        Some(TurtleRuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove the duplicate prefix declaration." }.to_owned(),
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
