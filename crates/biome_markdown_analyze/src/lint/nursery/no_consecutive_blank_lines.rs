use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::no_consecutive_blank_lines::NoConsecutiveBlankLinesOptions;

declare_lint_rule! {
    /// Limit the number of consecutive blank lines.
    ///
    /// Too many consecutive blank lines create excessive whitespace that makes
    /// a document harder to read and maintain.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// First paragraph
    ///
    ///
    /// Second paragraph
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// First paragraph
    ///
    /// Second paragraph
    /// ```
    ///
    /// ## Options
    ///
    /// ### `maxConsecutive`
    ///
    /// Maximum number of consecutive blank lines allowed. Default: `1`.
    pub NoConsecutiveBlankLines {
        version: "next",
        name: "noConsecutiveBlankLines",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct ExcessiveBlankLines {
    range: TextRange,
    count: usize,
}

impl Rule for NoConsecutiveBlankLines {
    type Query = Ast<MdDocument>;
    type State = ExcessiveBlankLines;
    type Signals = Vec<Self::State>;
    type Options = NoConsecutiveBlankLinesOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_trimmed().to_string();
        let base = document.syntax().text_trimmed_range().start();
        let max = ctx.options().max_consecutive() as usize;
        let mut signals = Vec::new();
        let mut consecutive_blank = 0usize;
        let mut blank_run_start = 0usize;
        let mut offset = 0usize;

        for line in text.lines() {
            let is_blank = line.trim().is_empty();
            if is_blank {
                if consecutive_blank == 0 {
                    blank_run_start = offset;
                }
                consecutive_blank += 1;
            } else {
                if consecutive_blank > max {
                    let run_end = offset; // start of current non-blank line
                    signals.push(ExcessiveBlankLines {
                        range: TextRange::new(
                            base + TextSize::from(blank_run_start as u32),
                            base + TextSize::from(run_end as u32),
                        ),
                        count: consecutive_blank,
                    });
                }
                consecutive_blank = 0;
            }
            offset += line.len() + 1; // +1 for newline
        }

        // Check trailing blank lines
        if consecutive_blank > max {
            let run_end = text.len();
            signals.push(ExcessiveBlankLines {
                range: TextRange::new(
                    base + TextSize::from(blank_run_start as u32),
                    base + TextSize::from(run_end as u32),
                ),
                count: consecutive_blank,
            });
        }

        signals
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let max = ctx.options().max_consecutive();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Found "{state.count.to_string()}" consecutive blank lines."
                },
            )
            .note(markup! {
                "At most "{max.to_string()}" consecutive blank line(s) allowed."
            }),
        )
    }
}
