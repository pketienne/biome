use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdParagraph;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::MarkdownRuleAction;
use crate::utils::fix_utils::make_text_replacement;
use crate::utils::inline_utils::{find_code_spans, find_inline_links};

declare_lint_rule! {
    /// Disallow reference-like URLs in inline links.
    ///
    /// Catches cases where an inline link's URL looks like a reference
    /// label (e.g. `[text][label]` instead of `[text](url)`).
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [text](label)
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [text](https://example.com)
    /// ```
    pub NoReferenceLikeUrl {
        version: "next",
        name: "noReferenceLikeUrl",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct ReferenceLikeUrl {
    range: TextRange,
    url: String,
    corrected: String,
}

impl Rule for NoReferenceLikeUrl {
    type Query = Ast<MdParagraph>;
    type State = ReferenceLikeUrl;
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

            for link in find_inline_links(line, &code_spans) {
                if link.is_image {
                    continue;
                }
                let url = link.url.trim();
                if !url.is_empty()
                    && !url.contains("://")
                    && !url.contains('/')
                    && !url.contains('.')
                    && !url.starts_with('#')
                    && !url.starts_with("mailto:")
                    && url.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ')
                {
                    let corrected = format!("[{}][{}]", link.text, url);
                    signals.push(ReferenceLikeUrl {
                        range: TextRange::new(
                            base + TextSize::from((offset + link.start) as u32),
                            base + TextSize::from((offset + link.end) as u32),
                        ),
                        url: url.to_string(),
                        corrected,
                    });
                }
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
            markup! { "Convert to reference link syntax." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let url = &state.url;
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "URL \""{ url }"\" looks like a reference label, not a URL."
                },
            )
            .note(markup! {
                "Use a proper URL or switch to reference link syntax [text][label]."
            }),
        )
    }
}
