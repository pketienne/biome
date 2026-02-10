use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdParagraph;
use biome_rowan::{AstNode, TextRange, TextSize};

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
    type Query = Ast<MdParagraph>;
    type State = MissingAlt;
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
                if link.is_image && link.text.trim().is_empty() {
                    signals.push(MissingAlt {
                        range: TextRange::new(
                            base + TextSize::from((offset + link.start) as u32),
                            base + TextSize::from((offset + link.end) as u32),
                        ),
                    });
                }
            }

            for link in find_reference_links(line, &code_spans) {
                if link.is_image && link.text.trim().is_empty() {
                    signals.push(MissingAlt {
                        range: TextRange::new(
                            base + TextSize::from((offset + link.start) as u32),
                            base + TextSize::from((offset + link.end) as u32),
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
                    "Image is missing alt text."
                },
            )
            .note(markup! {
                "Add descriptive alt text to images for accessibility."
            }),
        )
    }
}
