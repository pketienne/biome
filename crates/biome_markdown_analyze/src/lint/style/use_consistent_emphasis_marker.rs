use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::use_consistent_emphasis_marker::UseConsistentEmphasisMarkerOptions;

use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{find_code_spans, find_emphasis_markers};

declare_lint_rule! {
    /// Enforce consistent emphasis markers.
    ///
    /// Emphasis can use either asterisks (`*text*`) or underscores (`_text_`).
    /// This rule enforces a consistent style.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// This is _emphasized_ text.
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// This is *emphasized* text.
    /// ```
    ///
    /// ## Options
    ///
    /// ### `marker`
    ///
    /// Which emphasis marker to enforce. Default: `"star"`.
    /// Allowed values: `"star"`, `"underscore"`, `"consistent"`.
    pub UseConsistentEmphasisMarker {
        version: "next",
        name: "useConsistentEmphasisMarker",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct InconsistentEmphasisMarker {
    range: TextRange,
    expected: char,
    actual: char,
}

impl Rule for UseConsistentEmphasisMarker {
    type Query = Ast<MdDocument>;
    type State = InconsistentEmphasisMarker;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentEmphasisMarkerOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let marker_option = ctx.options().marker();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;
        let mut first_seen: Option<char> = None;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);
            if tracker.is_inside_fence() {
                offset += line.len() + 1;
                continue;
            }

            let code_spans = find_code_spans(line);
            let markers = find_emphasis_markers(line, &code_spans);

            for m in &markers {
                // Only check single emphasis markers (not strong/double)
                if m.count != 1 {
                    continue;
                }

                let expected = match marker_option {
                    "star" => '*',
                    "underscore" => '_',
                    "consistent" => {
                        if let Some(first) = first_seen {
                            first
                        } else {
                            first_seen = Some(m.marker_char);
                            continue;
                        }
                    }
                    _ => '*',
                };

                if m.marker_char != expected {
                    signals.push(InconsistentEmphasisMarker {
                        range: TextRange::new(
                            base + TextSize::from((offset + m.start) as u32),
                            base + TextSize::from((offset + m.start + m.count) as u32),
                        ),
                        expected,
                        actual: m.marker_char,
                    });
                }
            }

            offset += line.len() + 1;
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let expected = if state.expected == '*' { "*" } else { "_" };
        let actual = if state.actual == '*' { "*" } else { "_" };
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Expected emphasis marker \""{ expected }"\" but found \""{ actual }"\"."
                },
            )
            .note(markup! {
                "Use consistent emphasis markers throughout the document."
            }),
        )
    }
}
