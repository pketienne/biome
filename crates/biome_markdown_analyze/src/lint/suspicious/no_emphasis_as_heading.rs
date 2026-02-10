use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdParagraph;
use biome_rowan::AstNode;

use crate::MarkdownRuleAction;
use crate::utils::fix_utils::make_text_replacement;

declare_lint_rule! {
    /// Disallow using emphasis or bold as a heading substitute.
    ///
    /// A paragraph that consists entirely of bold or italic text
    /// likely should be a proper heading instead.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ````md
    /// **This should be a heading**
    /// ````
    ///
    /// ### Valid
    ///
    /// ```md
    /// ## This is a heading
    /// ```
    pub NoEmphasisAsHeading {
        version: "next",
        name: "noEmphasisAsHeading",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct EmphasisHeading {
    inner_text: String,
}

impl Rule for NoEmphasisAsHeading {
    type Query = Ast<MdParagraph>;
    type State = EmphasisHeading;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let paragraph = ctx.query();
        let text = paragraph.syntax().text_trimmed().to_string();
        let trimmed = text.trim();

        if trimmed.is_empty() {
            return None;
        }

        // Check if the paragraph is entirely wrapped in bold emphasis markers
        // Pattern: **text** or __text__ as entire paragraph content
        let inner = if trimmed.starts_with("**") && trimmed.ends_with("**") && trimmed.len() > 4 {
            &trimmed[2..trimmed.len() - 2]
        } else if trimmed.starts_with("__") && trimmed.ends_with("__") && trimmed.len() > 4 {
            &trimmed[2..trimmed.len() - 2]
        } else {
            return None;
        };

        // Don't flag multi-line paragraphs (emphasis-as-heading is single-line)
        if inner.contains('\n') {
            return None;
        }

        Some(EmphasisHeading {
            inner_text: inner.to_string(),
        })
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let paragraph = ctx.query();
        let range = paragraph.syntax().text_trimmed_range();
        let corrected = format!("## {}", state.inner_text);
        let mutation = make_text_replacement(&ctx.root(), range, &corrected)?;
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Convert to ATX heading." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                ctx.query().syntax().text_trimmed_range(),
                markup! {
                    "Emphasis used as a heading substitute."
                },
            )
            .note(markup! {
                "Use a proper heading (e.g. ## Heading) instead of bold/italic text."
            }),
        )
    }
}
