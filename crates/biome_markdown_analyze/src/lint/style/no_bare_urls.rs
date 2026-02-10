use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdParagraph;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::MarkdownRuleAction;
use crate::utils::fix_utils::make_text_replacement;
use crate::utils::inline_utils::{find_bare_urls, find_code_spans};

declare_lint_rule! {
    /// Disallow bare URLs in markdown text.
    ///
    /// Bare URLs should be wrapped in angle brackets (`<url>`) or
    /// proper link syntax (`[text](url)`).
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// Visit https://example.com for details.
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// Visit <https://example.com> for details.
    /// ```
    pub NoBareUrls {
        version: "next",
        name: "noBareUrls",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct BareUrl {
    range: TextRange,
    corrected: String,
}

impl Rule for NoBareUrls {
    type Query = Ast<MdParagraph>;
    type State = BareUrl;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let paragraph = ctx.query();
        let text = paragraph.syntax().text_trimmed().to_string();
        let base = paragraph.syntax().text_trimmed_range().start();
        let mut signals = Vec::new();
        let mut offset = 0usize;

        for line in text.lines() {
            let code_spans = find_code_spans(line);
            let bare_urls = find_bare_urls(line, &code_spans);

            for (url_start, url_end) in bare_urls {
                let url_text = &line[url_start..url_end];
                let corrected = format!("<{}>", url_text);
                signals.push(BareUrl {
                    range: TextRange::new(
                        base + TextSize::from((offset + url_start) as u32),
                        base + TextSize::from((offset + url_end) as u32),
                    ),
                    corrected,
                });
            }

            offset += line.len() + 1;
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let mutation = make_text_replacement(&ctx.root(), state.range, &state.corrected)?;
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Wrap URL in angle brackets." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Bare URL found. Wrap it in angle brackets or use link syntax."
                },
            )
            .note(markup! {
                "Use "<Emphasis>"<url>"</Emphasis>" or "<Emphasis>"[text](url)"</Emphasis>" instead of a bare URL."
            }),
        )
    }
}
