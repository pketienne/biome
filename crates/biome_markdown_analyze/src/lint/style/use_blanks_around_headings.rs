use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::line_utils::is_blank_line;

declare_lint_rule! {
    /// Require blank lines around headings.
    ///
    /// Headings should be surrounded by blank lines for readability.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// Some text
    /// # Heading
    /// More text
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// Some text
    ///
    /// # Heading
    ///
    /// More text
    /// ```
    pub UseBlanksAroundHeadings {
        version: "next",
        name: "useBlanksAroundHeadings",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct MissingBlankAroundHeading {
    range: TextRange,
    position: &'static str,
}

impl Rule for UseBlanksAroundHeadings {
    type Query = Ast<MdDocument>;
    type State = MissingBlankAroundHeading;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let lines: Vec<&str> = text.lines().collect();
        let mut signals = Vec::new();
        let mut offset = 0usize;

        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Detect ATX headings: lines starting with 1-6 '#' followed by space or end
            let is_heading = if trimmed.starts_with('#') {
                let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
                hash_count >= 1
                    && hash_count <= 6
                    && (trimmed.len() == hash_count
                        || trimmed.as_bytes().get(hash_count) == Some(&b' '))
            } else {
                false
            };

            if is_heading {
                // Check line before heading
                if line_idx > 0 && !is_blank_line(lines[line_idx - 1]) {
                    signals.push(MissingBlankAroundHeading {
                        range: TextRange::new(
                            base + TextSize::from(offset as u32),
                            base + TextSize::from((offset + line.len()) as u32),
                        ),
                        position: "before",
                    });
                }

                // Check line after heading
                if line_idx + 1 < lines.len() && !is_blank_line(lines[line_idx + 1]) {
                    signals.push(MissingBlankAroundHeading {
                        range: TextRange::new(
                            base + TextSize::from(offset as u32),
                            base + TextSize::from((offset + line.len()) as u32),
                        ),
                        position: "after",
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
                    "Missing blank line "{state.position}" heading."
                },
            )
            .note(markup! {
                "Add a blank line around headings for readability."
            }),
        )
    }
}
