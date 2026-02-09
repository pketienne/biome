use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::use_consistent_code_fence_marker::UseConsistentCodeFenceMarkerOptions;

use crate::utils::fence_utils::FenceTracker;

declare_lint_rule! {
    /// Enforce consistent code fence markers.
    ///
    /// Code fences can use either backticks (`` ` ``) or tildes (`~`).
    /// This rule enforces consistent usage.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ````md
    /// ~~~js
    /// code
    /// ~~~
    /// ````
    ///
    /// ### Valid
    ///
    /// ````md
    /// ```js
    /// code
    /// ```
    /// ````
    ///
    /// ## Options
    ///
    /// ### `marker`
    ///
    /// Which fence marker to enforce. Default: `"backtick"`.
    /// Allowed values: `"backtick"`, `"tilde"`.
    pub UseConsistentCodeFenceMarker {
        version: "next",
        name: "useConsistentCodeFenceMarker",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct InconsistentFenceMarker {
    range: TextRange,
    expected: char,
    actual: char,
}

impl Rule for UseConsistentCodeFenceMarker {
    type Query = Ast<MdDocument>;
    type State = InconsistentFenceMarker;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentCodeFenceMarkerOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let expected_char = match ctx.options().marker() {
            "tilde" => '~',
            _ => '`',
        };
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();

        for (line_idx, line) in text.lines().enumerate() {
            if let Some(fence_open) = tracker.process_line(line_idx, line) {
                if fence_open.fence_char != expected_char {
                    let line_start: usize = text.lines().take(line_idx).map(|l| l.len() + 1).sum();
                    let trimmed_start = line.len() - line.trim_start().len();
                    let fence_byte_start = line_start + trimmed_start;
                    let fence_byte_end = fence_byte_start + fence_open.fence_count;
                    signals.push(InconsistentFenceMarker {
                        range: TextRange::new(
                            base + TextSize::from(fence_byte_start as u32),
                            base + TextSize::from(fence_byte_end as u32),
                        ),
                        expected: expected_char,
                        actual: fence_open.fence_char,
                    });
                }
            }
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let expected_name = if state.expected == '`' {
            "backticks"
        } else {
            "tildes"
        };
        let actual_name = if state.actual == '`' {
            "backticks"
        } else {
            "tildes"
        };
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Expected "{expected_name}" but found "{actual_name}" for code fence marker."
                },
            )
            .note(markup! {
                "Use consistent code fence markers throughout the document."
            }),
        )
    }
}
