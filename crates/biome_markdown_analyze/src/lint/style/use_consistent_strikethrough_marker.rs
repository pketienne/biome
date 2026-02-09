use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::use_consistent_strikethrough_marker::UseConsistentStrikethroughMarkerOptions;

use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{find_code_spans, is_in_code_span};

declare_lint_rule! {
    /// Enforce consistent strikethrough marker style.
    ///
    /// Strikethrough can use `~~text~~` (double tilde) or `~text~`
    /// (single tilde). This rule enforces consistency.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"double-tilde"` (default):
    ///
    /// ```md
    /// ~single tilde~
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ~~double tilde~~
    /// ```
    ///
    /// ## Options
    ///
    /// ### `marker`
    ///
    /// Which marker style to enforce. Default: `"consistent"`.
    pub UseConsistentStrikethroughMarker {
        version: "next",
        name: "useConsistentStrikethroughMarker",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct InconsistentStrikethrough {
    range: TextRange,
    expected: &'static str,
    actual: &'static str,
}

impl Rule for UseConsistentStrikethroughMarker {
    type Query = Ast<MdDocument>;
    type State = InconsistentStrikethrough;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentStrikethroughMarkerOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let marker_style = ctx.options().marker();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();

        // Find all strikethrough markers and their styles
        let mut found_markers: Vec<(&'static str, usize, usize)> = Vec::new(); // (style, line_offset+start, end)

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);
            if tracker.is_inside_fence() {
                continue;
            }

            let line_offset: usize = text.lines().take(line_idx).map(|l| l.len() + 1).sum();
            let code_spans = find_code_spans(line);
            let bytes = line.as_bytes();
            let mut i = 0;

            while i < bytes.len() {
                if is_in_code_span(i, &code_spans) {
                    i += 1;
                    continue;
                }

                if bytes[i] == b'~' {
                    let start = i;
                    let mut count = 0;
                    while i < bytes.len() && bytes[i] == b'~' {
                        count += 1;
                        i += 1;
                    }

                    let style = if count == 2 { "double-tilde" } else if count == 1 { "tilde" } else { i += 1; continue; };
                    found_markers.push((style, line_offset + start, line_offset + start + count));
                } else {
                    i += 1;
                }
            }
        }

        if found_markers.is_empty() {
            return signals;
        }

        let expected = match marker_style {
            "tilde" => "tilde",
            "double-tilde" => "double-tilde",
            _ => {
                // consistent: use first marker style
                found_markers[0].0
            }
        };

        for &(style, start, end) in &found_markers {
            if style != expected {
                signals.push(InconsistentStrikethrough {
                    range: TextRange::new(
                        base + TextSize::from(start as u32),
                        base + TextSize::from(end as u32),
                    ),
                    expected,
                    actual: style,
                });
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
                    "Expected "{state.expected}" strikethrough marker but found "{state.actual}"."
                },
            )
            .note(markup! {
                "Use a consistent strikethrough marker style."
            }),
        )
    }
}
