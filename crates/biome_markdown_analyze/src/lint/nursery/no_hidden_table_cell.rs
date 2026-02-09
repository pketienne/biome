use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::table_utils::{collect_tables, split_table_cells};

declare_lint_rule! {
    /// Disallow hidden table cells.
    ///
    /// Extra cells beyond the column count are hidden from rendering.
    /// This flags rows with more cells than the separator defines.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// | A | B |
    /// | --- | --- |
    /// | 1 | 2 | hidden |
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// | A | B |
    /// | --- | --- |
    /// | 1 | 2 |
    /// ```
    pub NoHiddenTableCell {
        version: "next",
        name: "noHiddenTableCell",
        language: "md",
        recommended: true,
        severity: Severity::Error,
    }
}

pub struct HiddenTableCell {
    range: TextRange,
    extra_count: usize,
}

impl Rule for NoHiddenTableCell {
    type Query = Ast<MdDocument>;
    type State = HiddenTableCell;
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

            for &data_line in &table.data_lines {
                let data_cells = split_table_cells(lines[data_line]);
                if data_cells.len() > expected {
                    signals.push(HiddenTableCell {
                        range: TextRange::new(
                            base + TextSize::from(offsets[data_line] as u32),
                            base + TextSize::from((offsets[data_line] + lines[data_line].len()) as u32),
                        ),
                        extra_count: data_cells.len() - expected,
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
                    "Row has "{state.extra_count}" hidden cell(s) that won't be rendered."
                },
            )
            .note(markup! {
                "Remove extra cells or add columns to the table header and separator."
            }),
        )
    }
}
