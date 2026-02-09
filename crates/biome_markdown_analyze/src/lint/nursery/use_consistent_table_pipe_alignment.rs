use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::table_utils::collect_tables;

declare_lint_rule! {
    /// Enforce aligned table pipe characters.
    ///
    /// Pipe characters in tables should be vertically aligned for readability.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// | A | B |
    /// | --- | --- |
    /// | long text | x |
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// | A         | B |
    /// | --------- | - |
    /// | long text | x |
    /// ```
    pub UseConsistentTablePipeAlignment {
        version: "next",
        name: "useConsistentTablePipeAlignment",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct MisalignedPipes {
    range: TextRange,
}

impl Rule for UseConsistentTablePipeAlignment {
    type Query = Ast<MdDocument>;
    type State = MisalignedPipes;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let tables = collect_tables(&text);
        let lines: Vec<&str> = text.lines().collect();

        let mut signals = Vec::new();
        let mut offsets = Vec::with_capacity(lines.len());
        let mut offset = 0usize;
        for line in &lines {
            offsets.push(offset);
            offset += line.len() + 1;
        }

        for table in &tables {
            let all_lines: Vec<usize> = std::iter::once(table.header_line)
                .chain(std::iter::once(table.separator_line))
                .chain(table.data_lines.iter().copied())
                .collect();

            if all_lines.len() < 2 {
                continue;
            }

            // Check if all rows have the same length (a simple alignment check)
            let first_len = lines[all_lines[0]].trim().len();
            let all_same_len = all_lines.iter().all(|&l| lines[l].trim().len() == first_len);

            if !all_same_len {
                // Find pipe positions in the first row
                let first_pipes: Vec<usize> = lines[all_lines[0]]
                    .char_indices()
                    .filter(|(_, c)| *c == '|')
                    .map(|(i, _)| i)
                    .collect();

                // Check subsequent rows for pipe alignment
                for &line_idx in all_lines.iter().skip(1) {
                    let this_pipes: Vec<usize> = lines[line_idx]
                        .char_indices()
                        .filter(|(_, c)| *c == '|')
                        .map(|(i, _)| i)
                        .collect();

                    if this_pipes != first_pipes {
                        signals.push(MisalignedPipes {
                            range: TextRange::new(
                                base + TextSize::from(offsets[line_idx] as u32),
                                base + TextSize::from((offsets[line_idx] + lines[line_idx].len()) as u32),
                            ),
                        });
                    }
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
                    "Table pipes are not aligned with other rows."
                },
            )
            .note(markup! {
                "Align pipe characters vertically for readability."
            }),
        )
    }
}
