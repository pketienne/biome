use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::line_utils::is_blank_line;
use crate::utils::table_utils::collect_tables;

declare_lint_rule! {
    /// Require blank lines around tables.
    ///
    /// Tables should be surrounded by blank lines for readability.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// Some text
    /// | A | B |
    /// | --- | --- |
    /// | 1 | 2 |
    /// More text
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// Some text
    ///
    /// | A | B |
    /// | --- | --- |
    /// | 1 | 2 |
    ///
    /// More text
    /// ```
    pub UseBlanksAroundTables {
        version: "next",
        name: "useBlanksAroundTables",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct MissingBlankAroundTable {
    range: TextRange,
    position: &'static str,
}

impl Rule for UseBlanksAroundTables {
    type Query = Ast<MdDocument>;
    type State = MissingBlankAroundTable;
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
            // Check line before header
            if table.header_line > 0 && !is_blank_line(lines[table.header_line - 1]) {
                signals.push(MissingBlankAroundTable {
                    range: TextRange::new(
                        base + TextSize::from(offsets[table.header_line] as u32),
                        base + TextSize::from((offsets[table.header_line] + lines[table.header_line].len()) as u32),
                    ),
                    position: "before",
                });
            }

            // Check line after last row
            let last_line = if table.data_lines.is_empty() {
                table.separator_line
            } else {
                *table.data_lines.last().unwrap()
            };

            if last_line + 1 < lines.len() && !is_blank_line(lines[last_line + 1]) {
                signals.push(MissingBlankAroundTable {
                    range: TextRange::new(
                        base + TextSize::from(offsets[last_line] as u32),
                        base + TextSize::from((offsets[last_line] + lines[last_line].len()) as u32),
                    ),
                    position: "after",
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
                    "Missing blank line "{state.position}" table."
                },
            )
            .note(markup! {
                "Add a blank line around tables for readability."
            }),
        )
    }
}
