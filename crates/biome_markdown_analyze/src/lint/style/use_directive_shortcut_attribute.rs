use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDirective;
use biome_rowan::{AstNode, AstNodeList, TextRange};

use crate::MarkdownRuleAction;
use crate::utils::fix_utils::make_text_replacement;

declare_lint_rule! {
    /// Enforce shortcut id attributes in directives.
    ///
    /// Directive syntax supports `#id` as shorthand for `id="id"`.
    /// Using the shorthand form is more concise and idiomatic.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// :::note{id="main"}
    /// :::
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// :::note{#main}
    /// :::
    /// ```
    pub UseDirectiveShortcutAttribute {
        version: "next",
        name: "useDirectiveShortcutAttribute",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct ExpandedId {
    range: TextRange,
    value: String,
    corrected: String,
}

impl Rule for UseDirectiveShortcutAttribute {
    type Query = Ast<MdDirective>;
    type State = ExpandedId;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let directive = ctx.query();
        let mut signals = Vec::new();

        for attr in directive.attributes().iter() {
            let name = attr.name().syntax().text_trimmed().to_string();
            // Skip shorthands (already using `#id` form)
            if name.starts_with('#') || name.starts_with('.') {
                continue;
            }
            if name == "id" {
                if let Some(val_node) = attr.value() {
                    let val = val_node.content().syntax().text_trimmed().to_string();
                    let corrected = format!("#{val}");
                    signals.push(ExpandedId {
                        range: attr.syntax().text_trimmed_range(),
                        value: val,
                        corrected,
                    });
                }
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let mutation = make_text_replacement(&ctx.root(), state.range, &state.corrected)?;
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Use shortcut id attribute." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Use shortcut id instead of "{ "id=\"" }{ &state.value }{ "\"" }"."
                },
            )
            .note(markup! {
                "Use the shorthand form: #"{ &state.value }" instead."
            }),
        )
    }
}
