use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::use_consistent_link_style::UseConsistentLinkStyleOptions;

use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{find_code_spans, find_inline_links, find_reference_links};

declare_lint_rule! {
    /// Enforce consistent link style.
    ///
    /// Links can be written as inline (`[text](url)`) or reference
    /// (`[text][label]`). This rule enforces a consistent style.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"reference"`, inline links are flagged:
    ///
    /// ```md
    /// [text](https://example.com)
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [text][label]
    ///
    /// [label]: https://example.com
    /// ```
    ///
    /// ## Options
    ///
    /// ### `style`
    ///
    /// Which link style to enforce. Default: `"inline"`.
    /// Allowed values: `"inline"`, `"reference"`, `"consistent"`.
    pub UseConsistentLinkStyle {
        version: "next",
        name: "useConsistentLinkStyle",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct InconsistentLinkStyle {
    range: TextRange,
    expected: &'static str,
    actual: &'static str,
}

impl Rule for UseConsistentLinkStyle {
    type Query = Ast<MdDocument>;
    type State = InconsistentLinkStyle;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentLinkStyleOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let style = ctx.options().style();

        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;

        // For "consistent" mode, determine from first link found
        let mut first_style: Option<&str> = if style == "consistent" {
            None
        } else {
            Some(style)
        };

        // First pass for consistent mode: find the first link style
        if first_style.is_none() {
            let mut temp_tracker = FenceTracker::new();
            let mut temp_offset = 0usize;
            'outer: for (line_idx, line) in text.lines().enumerate() {
                temp_tracker.process_line(line_idx, line);
                if !temp_tracker.is_inside_fence() {
                    let code_spans = find_code_spans(line);
                    let inline = find_inline_links(line, &code_spans);
                    let refs = find_reference_links(line, &code_spans);
                    // Check non-image links only
                    for link in &inline {
                        if !link.is_image {
                            first_style = Some("inline");
                            break 'outer;
                        }
                    }
                    for rlink in &refs {
                        if !rlink.is_image {
                            first_style = Some("reference");
                            break 'outer;
                        }
                    }
                }
                temp_offset += line.len() + 1;
            }
            let _ = temp_offset;
        }

        let expected_style = match first_style {
            Some(s) => s,
            None => return signals, // no links found
        };

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);

            if !tracker.is_inside_fence() {
                let code_spans = find_code_spans(line);

                if expected_style == "reference" {
                    let inline = find_inline_links(line, &code_spans);
                    for link in inline {
                        if !link.is_image {
                            signals.push(InconsistentLinkStyle {
                                range: TextRange::new(
                                    base + TextSize::from((offset + link.start) as u32),
                                    base + TextSize::from((offset + link.end) as u32),
                                ),
                                expected: "reference",
                                actual: "inline",
                            });
                        }
                    }
                } else if expected_style == "inline" {
                    let refs = find_reference_links(line, &code_spans);
                    for rlink in refs {
                        if !rlink.is_image {
                            signals.push(InconsistentLinkStyle {
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
                    "Expected "{state.expected}" link style but found "{state.actual}"."
                },
            )
            .note(markup! {
                "Use a consistent link style throughout the document."
            }),
        )
    }
}
