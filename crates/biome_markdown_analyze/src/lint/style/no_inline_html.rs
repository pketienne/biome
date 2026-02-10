use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdParagraph;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::no_inline_html::NoInlineHtmlOptions;

use crate::MarkdownRuleAction;
use crate::utils::fix_utils::make_text_replacement;
use crate::utils::inline_utils::{find_code_spans, find_html_tags};

declare_lint_rule! {
    /// Disallow inline HTML in markdown.
    ///
    /// Inline HTML can reduce the portability of markdown documents.
    /// Use markdown syntax instead.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// This has <em>inline HTML</em>.
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// This has *emphasis* instead.
    /// ```
    ///
    /// ## Options
    ///
    /// ### `allowedElements`
    ///
    /// HTML elements that are allowed. Default: empty (no elements allowed).
    pub NoInlineHtml {
        version: "next",
        name: "noInlineHtml",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct InlineHtml {
    range: TextRange,
    tag_name: String,
}

impl Rule for NoInlineHtml {
    type Query = Ast<MdParagraph>;
    type State = InlineHtml;
    type Signals = Vec<Self::State>;
    type Options = NoInlineHtmlOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let paragraph = ctx.query();
        let text = paragraph.syntax().text_trimmed().to_string();
        let base = paragraph.syntax().text_trimmed_range().start();
        let allowed = ctx.options().allowed_elements();
        let mut signals = Vec::new();
        let mut offset = 0usize;

        for line in text.lines() {
            let code_spans = find_code_spans(line);
            let tags = find_html_tags(line, &code_spans);

            for tag in &tags {
                if allowed.iter().any(|a: &String| a.to_lowercase() == tag.tag_name) {
                    continue;
                }

                signals.push(InlineHtml {
                    range: TextRange::new(
                        base + TextSize::from((offset + tag.start) as u32),
                        base + TextSize::from((offset + tag.end) as u32),
                    ),
                    tag_name: tag.tag_name.clone(),
                });
            }

            offset += line.len() + 1;
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let mutation = make_text_replacement(&ctx.root(), state.range, "")?;
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove the HTML tag." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Inline HTML element \""{ &state.tag_name }"\" is not allowed."
                },
            )
            .note(markup! {
                "Use markdown syntax instead of inline HTML."
            }),
        )
    }
}
