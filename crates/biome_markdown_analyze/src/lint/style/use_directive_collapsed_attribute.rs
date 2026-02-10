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
    /// Enforce collapsed class attributes in directives.
    ///
    /// Directive syntax supports `.class` as shorthand for `class="class"`.
    /// Using the shorthand form is more concise and idiomatic.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// :::note{class="warning"}
    /// :::
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// :::note{.warning}
    /// :::
    /// ```
    pub UseDirectiveCollapsedAttribute {
        version: "next",
        name: "useDirectiveCollapsedAttribute",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct ExpandedClass {
    range: TextRange,
    value: String,
    corrected: String,
}

impl Rule for UseDirectiveCollapsedAttribute {
    type Query = Ast<MdDirective>;
    type State = ExpandedClass;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let directive = ctx.query();
        let mut signals = Vec::new();

        for attr in directive.attributes().iter() {
            let name = attr.name().syntax().text_trimmed().to_string();
            // Skip shorthands (already using `.class` form)
            if name.starts_with('.') || name.starts_with('#') {
                continue;
            }
            if name == "class" {
                if let Some(val_node) = attr.value() {
                    let val = val_node.content().syntax().text_trimmed().to_string();
                    let corrected = format!(".{val}");
                    signals.push(ExpandedClass {
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
            markup! { "Use collapsed class shorthand." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Use collapsed class shorthand instead of "{ "class=\"" }{ &state.value }{ "\"" }"."
                },
            )
            .note(markup! {
                "Use the shorthand form: ."{ &state.value }" instead."
            }),
        )
    }
}
