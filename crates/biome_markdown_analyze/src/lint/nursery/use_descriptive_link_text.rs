use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::use_descriptive_link_text::UseDescriptiveLinkTextOptions;

use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{find_code_spans, find_inline_links, find_reference_links};

declare_lint_rule! {
    /// Require descriptive link text.
    ///
    /// Link text should be descriptive and not use generic phrases
    /// like "click here" or "here".
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [click here](https://example.com)
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [Example documentation](https://example.com)
    /// ```
    ///
    /// ## Options
    ///
    /// ### `minimumLength`
    ///
    /// Minimum number of characters for link text. Default: `1`.
    pub UseDescriptiveLinkText {
        version: "next",
        name: "useDescriptiveLinkText",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

const FORBIDDEN_TEXTS: &[&str] = &[
    "click here",
    "here",
    "link",
    "read more",
    "more",
    "this",
    "page",
];

pub struct BadLinkText {
    range: TextRange,
    text: String,
}

impl Rule for UseDescriptiveLinkText {
    type Query = Ast<MdDocument>;
    type State = BadLinkText;
    type Signals = Vec<Self::State>;
    type Options = UseDescriptiveLinkTextOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let min_length = ctx.options().minimum_length();
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
                let link_text = link.text.trim().to_ascii_lowercase();
                if (link_text.len() as u32) < min_length
                    || FORBIDDEN_TEXTS.contains(&link_text.as_str())
                {
                    signals.push(BadLinkText {
                        range: TextRange::new(
                            base + TextSize::from((line_offset + link.start) as u32),
                            base + TextSize::from((line_offset + link.end) as u32),
                        ),
                        text: link.text.clone(),
                    });
                }
            }

            for link in find_reference_links(line, &code_spans) {
                if link.is_image {
                    continue;
                }
                let link_text = link.text.trim().to_ascii_lowercase();
                if (link_text.len() as u32) < min_length
                    || FORBIDDEN_TEXTS.contains(&link_text.as_str())
                {
                    signals.push(BadLinkText {
                        range: TextRange::new(
                            base + TextSize::from((line_offset + link.start) as u32),
                            base + TextSize::from((line_offset + link.end) as u32),
                        ),
                        text: link.text.clone(),
                    });
                }
            }
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let text = &state.text;
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Link text \""{ text }"\" is not descriptive."
                },
            )
            .note(markup! {
                "Use descriptive link text that explains the destination."
            }),
        )
    }
}
