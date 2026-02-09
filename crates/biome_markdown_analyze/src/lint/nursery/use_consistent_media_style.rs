use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::use_consistent_media_style::UseConsistentMediaStyleOptions;

use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{find_code_spans, find_inline_links, find_reference_links};

declare_lint_rule! {
    /// Enforce consistent image/media style.
    ///
    /// Images can be written as inline (`![alt](url)`) or reference
    /// (`![alt][label]`). This rule enforces a consistent style.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"reference"`, inline images are flagged:
    ///
    /// ```md
    /// ![alt](image.png)
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ![alt][img]
    ///
    /// [img]: image.png
    /// ```
    ///
    /// ## Options
    ///
    /// ### `style`
    ///
    /// Which style to enforce. Default: `"inline"`.
    /// Allowed values: `"inline"`, `"reference"`.
    pub UseConsistentMediaStyle {
        version: "next",
        name: "useConsistentMediaStyle",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct InconsistentMediaStyle {
    range: TextRange,
    expected: &'static str,
    actual: &'static str,
}

impl Rule for UseConsistentMediaStyle {
    type Query = Ast<MdDocument>;
    type State = InconsistentMediaStyle;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentMediaStyleOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let style = ctx.options().style();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);

            if !tracker.is_inside_fence() {
                let code_spans = find_code_spans(line);

                if style == "reference" {
                    let inline = find_inline_links(line, &code_spans);
                    for link in inline {
                        if link.is_image {
                            signals.push(InconsistentMediaStyle {
                                range: TextRange::new(
                                    base + TextSize::from((offset + link.start) as u32),
                                    base + TextSize::from((offset + link.end) as u32),
                                ),
                                expected: "reference",
                                actual: "inline",
                            });
                        }
                    }
                } else {
                    let refs = find_reference_links(line, &code_spans);
                    for rlink in refs {
                        if rlink.is_image {
                            signals.push(InconsistentMediaStyle {
                                range: TextRange::new(
                                    base + TextSize::from((offset + rlink.start) as u32),
                                    base + TextSize::from((offset + rlink.end) as u32),
                                ),
                                expected: "inline",
                                actual: "reference",
                            });
                        }
                    }
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
                    "Expected "{state.expected}" image style but found "{state.actual}"."
                },
            )
            .note(markup! {
                "Use a consistent image style throughout the document."
            }),
        )
    }
}
