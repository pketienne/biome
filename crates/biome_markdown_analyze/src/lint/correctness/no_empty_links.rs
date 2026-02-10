use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdInlineLink;
use biome_rowan::{AstNode, AstNodeList};

use crate::MarkdownRuleAction;
use crate::utils::fix_utils::make_text_replacement;

declare_lint_rule! {
    /// Disallow links with empty URLs.
    ///
    /// Links with empty URLs (`[text]()`) are broken and do not navigate anywhere.
    /// Either provide a valid URL or remove the link.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [click here]()
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [click here](https://example.com)
    /// ```
    pub NoEmptyLinks {
        version: "next",
        name: "noEmptyLinks",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

impl Rule for NoEmptyLinks {
    type Query = Ast<MdInlineLink>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let link = ctx.query();
        // Check if the source (URL) is empty or whitespace-only
        let source = link.source();
        if source.is_empty() {
            return Some(());
        }
        let source_text = source.syntax().text_trimmed().to_string();
        if source_text.trim().is_empty() {
            Some(())
        } else {
            None
        }
    }

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<MarkdownRuleAction> {
        let link = ctx.query();
        let link_text = link.text().syntax().text_trimmed().to_string();
        let range = link.syntax().text_trimmed_range();
        let mutation = make_text_replacement(&ctx.root(), range, &link_text)?;
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove the empty link." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                ctx.query().syntax().text_trimmed_range(),
                markup! {
                    "Link has an empty URL."
                },
            )
            .note(markup! {
                "Provide a valid URL for the link or remove it."
            }),
        )
    }
}
