use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::fence_utils::FenceTracker;
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
    }
}

pub struct ReferenceLikeUrl {
    range: TextRange,
    url: String,
}

impl Rule for NoReferenceLikeUrl {
    type Query = Ast<MdDocument>;
    type State = ReferenceLikeUrl;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);
            if tracker.is_inside_fence() {
                continue;
            }

            let line_offset: usize = text.lines().take(line_idx).map(|l| l.len() + 1).sum();
            let code_spans = find_code_spans(line);

            for link in find_inline_links(line, &code_spans) {
                if link.is_image {
                    continue;
                }
                // A reference-like URL: no scheme, no path separators, no dots
                let url = link.url.trim();
                if !url.is_empty()
                    && !url.contains("://")
                    && !url.contains('/')
                    && !url.contains('.')
                    && !url.starts_with('#')
                    && !url.starts_with("mailto:")
                    && url.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ')
                {
                    signals.push(ReferenceLikeUrl {
                        range: TextRange::new(
                            base + TextSize::from((line_offset + link.start) as u32),
                            base + TextSize::from((line_offset + link.end) as u32),
                        ),
                        url: url.to_string(),
                    });
                }
            }
        }

        signals
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
