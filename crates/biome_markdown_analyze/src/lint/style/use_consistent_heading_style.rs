use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::use_consistent_heading_style::UseConsistentHeadingStyleOptions;

use crate::utils::fence_utils::FenceTracker;

declare_lint_rule! {
    /// Enforce consistent heading style.
    ///
    /// Headings can use ATX style (`# heading`) or setext style
    /// (underlined with `===` or `---`). This rule enforces consistency.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"atx"` (default), setext headings are flagged:
    ///
    /// ```md
    /// Heading
    /// =======
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// # Heading
    /// ```
    ///
    /// ## Options
    ///
    /// ### `style`
    ///
    /// Which heading style to enforce. Default: `"consistent"`.
    pub UseConsistentHeadingStyle {
        version: "next",
        name: "useConsistentHeadingStyle",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct InconsistentHeadingStyle {
    range: TextRange,
    expected: &'static str,
    actual: &'static str,
}

impl Rule for UseConsistentHeadingStyle {
    type Query = Ast<MdDocument>;
    type State = InconsistentHeadingStyle;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentHeadingStyleOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let style = ctx.options().style();
        let lines: Vec<&str> = text.lines().collect();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();

        // Detect heading styles
        let mut headings: Vec<(&'static str, usize, usize)> = Vec::new(); // (style, line, byte_len)

        for (line_idx, line) in lines.iter().enumerate() {
            tracker.process_line(line_idx, line);
            if tracker.is_inside_fence() {
                continue;
            }

            // ATX heading
            if line.starts_with('#') {
                headings.push(("atx", line_idx, line.len()));
            }

            // Setext heading (line of = or - under a non-empty line)
            if line_idx > 0 && !lines[line_idx - 1].trim().is_empty() {
                let trimmed = line.trim();
                if !trimmed.is_empty()
                    && (trimmed.chars().all(|c| c == '=') || trimmed.chars().all(|c| c == '-'))
                    && trimmed.len() >= 2
                {
                    // This could be a setext heading underline
                    // The heading text is the previous line
                    headings.push(("setext", line_idx - 1, lines[line_idx - 1].len()));
                }
            }
        }

        if headings.is_empty() {
            return signals;
        }

        let expected_style = match style {
            "atx" => "atx",
            "setext" => "setext",
            _ => {
                // consistent: use the first heading's style
                headings[0].0
            }
        };

        for &(heading_style, line_idx, _) in &headings {
            if heading_style != expected_style {
                let line_offset: usize =
                    lines[..line_idx].iter().map(|l| l.len() + 1).sum();
                let line = lines[line_idx];
                signals.push(InconsistentHeadingStyle {
                    range: TextRange::new(
                        base + TextSize::from(line_offset as u32),
                        base + TextSize::from((line_offset + line.len()) as u32),
                    ),
                    expected: expected_style,
                    actual: heading_style,
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
                    "Expected "{state.expected}" heading style but found "{state.actual}"."
                },
            )
            .note(markup! {
                "Use a consistent heading style throughout the document."
            }),
        )
    }
}
