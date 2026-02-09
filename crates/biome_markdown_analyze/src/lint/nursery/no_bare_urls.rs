use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::fence_utils::FenceTracker;
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
    }
}

pub struct BareUrl {
    range: TextRange,
}

impl Rule for NoBareUrls {
    type Query = Ast<MdDocument>;
    type State = BareUrl;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);

            if !tracker.is_inside_fence() {
                let code_spans = find_code_spans(line);
                let bare_urls = find_bare_urls(line, &code_spans);

                for (url_start, url_end) in bare_urls {
                    signals.push(BareUrl {
                        range: TextRange::new(
                            base + TextSize::from((offset + url_start) as u32),
                            base + TextSize::from((offset + url_end) as u32),
                        ),
                    });
                }
            }

            offset += line.len() + 1;
        }

        signals
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
