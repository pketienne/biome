use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::no_inline_html::NoInlineHtmlOptions;

use crate::utils::fence_utils::FenceTracker;
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
    }
}

pub struct InlineHtml {
    range: TextRange,
    tag_name: String,
}

impl Rule for NoInlineHtml {
    type Query = Ast<MdDocument>;
    type State = InlineHtml;
    type Signals = Vec<Self::State>;
    type Options = NoInlineHtmlOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let allowed = ctx.options().allowed_elements();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);
            if tracker.is_inside_fence() {
                offset += line.len() + 1;
                continue;
            }

            let code_spans = find_code_spans(line);
            let tags = find_html_tags(line, &code_spans);

            for tag in &tags {
                // Skip allowed elements
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
