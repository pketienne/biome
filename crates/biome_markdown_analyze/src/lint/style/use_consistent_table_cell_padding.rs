use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::use_consistent_table_cell_padding::UseConsistentTableCellPaddingOptions;

use crate::utils::table_utils::collect_tables;

declare_lint_rule! {
    /// Enforce consistent table cell padding.
    ///
    /// Table cells can be padded with spaces or compact without spaces.
    /// This rule enforces consistency.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"padded"` (default), compact cells are flagged:
    ///
    /// ```md
    /// |A|B|
    /// |---|---|
    /// |1|2|
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// | A | B |
    /// | --- | --- |
    /// | 1 | 2 |
    /// ```
    ///
    /// ## Options
    ///
    /// ### `style`
    ///
    /// Which padding style to enforce. Default: `"padded"`.
    /// Allowed values: `"padded"`, `"compact"`, `"consistent"`.
    pub UseConsistentTableCellPadding {
        version: "next",
        name: "useConsistentTableCellPadding",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct InconsistentCellPadding {
    range: TextRange,
    expected: &'static str,
}

impl Rule for UseConsistentTableCellPadding {
    type Query = Ast<MdDocument>;
    type State = InconsistentCellPadding;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentTableCellPaddingOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let style = ctx.options().style();
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
            let check_lines: Vec<usize> = std::iter::once(table.header_line)
                .chain(table.data_lines.iter().copied())
                .collect();

            // For consistent mode, check first header cell
            let effective_style = if style == "consistent" {
                let header = lines[table.header_line].trim();
                if header.contains("| ") || header.contains(" |") {
                    "padded"
                } else {
                    "compact"
                }
            } else {
                style
            };

            for &line_idx in &check_lines {
                let line = lines[line_idx];
                let trimmed = line.trim();

                let is_padded = self::has_cell_padding(trimmed);

                match effective_style {
                    "padded" => {
                        if !is_padded {
                            signals.push(InconsistentCellPadding {
                                range: TextRange::new(
                                    base + TextSize::from(offsets[line_idx] as u32),
                                    base + TextSize::from((offsets[line_idx] + line.len()) as u32),
                                ),
                                expected: "padded",
                            });
                        }
                    }
                    "compact" => {
                        if is_padded {
                            signals.push(InconsistentCellPadding {
                                range: TextRange::new(
                                    base + TextSize::from(offsets[line_idx] as u32),
                                    base + TextSize::from((offsets[line_idx] + line.len()) as u32),
                                ),
                                expected: "compact",
                            });
                        }
                    }
                    _ => {}
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
                    "Expected "{state.expected}" cell padding in table row."
                },
            )
            .note(markup! {
                "Use consistent cell padding throughout tables."
            }),
        )
    }
}

/// Check if a table row has padded cells (spaces around pipe delimiters).
fn has_cell_padding(row: &str) -> bool {
    // Strip leading/trailing pipes
    let s = row.strip_prefix('|').unwrap_or(row);
    let s = s.strip_suffix('|').unwrap_or(s);

    // Check if any cell has leading or trailing spaces
    for cell in s.split('|') {
        if !cell.is_empty() && (cell.starts_with(' ') || cell.ends_with(' ')) {
            return true;
        }
    }
    false
}
