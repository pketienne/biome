use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::table_utils::{collect_tables, split_table_cells};

declare_lint_rule! {
    /// Disallow table rows with mismatched column counts.
    ///
    /// All rows in a GFM table should have the same number of cells
    /// as defined by the separator row.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// | A | B |
    /// | --- | --- |
    /// | 1 | 2 | 3 |
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// | A | B |
    /// | --- | --- |
    /// | 1 | 2 |
    /// ```
    pub NoMismatchedTableColumnCount {
        version: "next",
        name: "noMismatchedTableColumnCount",
        language: "md",
        recommended: true,
        severity: Severity::Error,
    }
}

pub struct MismatchedColumnCount {
    range: TextRange,
    expected: usize,
    actual: usize,
}

impl Rule for NoMismatchedTableColumnCount {
    type Query = Ast<MdDocument>;
    type State = MismatchedColumnCount;
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
            let expected = table.column_count;

            // Check header row
            let header_cells = split_table_cells(lines[table.header_line]);
            if header_cells.len() != expected {
                signals.push(MismatchedColumnCount {
                    range: TextRange::new(
                        base + TextSize::from(offsets[table.header_line] as u32),
                        base + TextSize::from((offsets[table.header_line] + lines[table.header_line].len()) as u32),
                    ),
                    expected,
                    actual: header_cells.len(),
                });
            }

            // Check data rows
            for &data_line in &table.data_lines {
                let data_cells = split_table_cells(lines[data_line]);
                if data_cells.len() != expected {
                    signals.push(MismatchedColumnCount {
                        range: TextRange::new(
                            base + TextSize::from(offsets[data_line] as u32),
                            base + TextSize::from((offsets[data_line] + lines[data_line].len()) as u32),
                        ),
                        expected,
                        actual: data_cells.len(),
                    });
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
                    "Expected "{state.expected}" columns but found "{state.actual}"."
                },
            )
            .note(markup! {
                "All rows in a table should have the same number of cells."
            }),
        )
    }
}
