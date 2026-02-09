use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{find_code_spans, find_inline_links, find_reference_links};

declare_lint_rule! {
    /// Require alt text for images.
    ///
    /// Images should have descriptive alt text for accessibility.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// ![](image.png)
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ![A description](image.png)
    /// ```
    pub NoMissingAltText {
        version: "next",
        name: "noMissingAltText",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct MissingAlt {
    range: TextRange,
}

impl Rule for NoMissingAltText {
    type Query = Ast<MdDocument>;
    type State = MissingAlt;
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

            // Check inline images
            for link in find_inline_links(line, &code_spans) {
                if link.is_image && link.text.trim().is_empty() {
                    signals.push(MissingAlt {
                        range: TextRange::new(
                            base + TextSize::from((line_offset + link.start) as u32),
                            base + TextSize::from((line_offset + link.end) as u32),
                        ),
                    });
                }
            }

            // Check reference images
            for link in find_reference_links(line, &code_spans) {
                if link.is_image && link.text.trim().is_empty() {
                    signals.push(MissingAlt {
                        range: TextRange::new(
                            base + TextSize::from((line_offset + link.start) as u32),
                            base + TextSize::from((line_offset + link.end) as u32),
                        ),
                    });
                }
            }
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Image is missing alt text."
                },
            )
            .note(markup! {
                "Add descriptive alt text to images for accessibility."
            }),
        )
    }
}
