use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use biome_rule_options::use_consistent_table_cell_padding::UseConsistentTableCellPaddingOptions;

use crate::MarkdownRuleAction;
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
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentCellPadding {
    range: TextRange,
    expected: &'static str,
    corrected: String,
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
                            let corrected = add_cell_padding(trimmed);
                            signals.push(InconsistentCellPadding {
                                range: TextRange::new(
                                    base + TextSize::from(offsets[line_idx] as u32),
                                    base + TextSize::from((offsets[line_idx] + line.len()) as u32),
                                ),
                                expected: "padded",
                                corrected,
                            });
                        }
                    }
                    "compact" => {
                        if is_padded {
                            let corrected = remove_cell_padding(trimmed);
                            signals.push(InconsistentCellPadding {
                                range: TextRange::new(
                                    base + TextSize::from(offsets[line_idx] as u32),
                                    base + TextSize::from((offsets[line_idx] + line.len()) as u32),
                                ),
                                expected: "compact",
                                corrected,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();
        let mut token = root
            .syntax()
            .token_at_offset(state.range.start())
            .right_biased()?;
        let mut tokens = vec![token.clone()];
        while token.text_range().end() < state.range.end() {
            token = token.next_token()?;
            tokens.push(token.clone());
        }
        let first = &tokens[0];
        let last = tokens.last()?;
        let prefix_len = u32::from(state.range.start() - first.text_range().start()) as usize;
        let suffix_start = u32::from(state.range.end() - last.text_range().start()) as usize;
        let prefix = &first.text()[..prefix_len];
        let suffix = &last.text()[suffix_start..];
        let new_text = format!("{}{}{}", prefix, state.corrected, suffix);
        let new_token = biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
            first.kind(),
            &new_text,
            [],
            [],
        );
        let mut mutation = ctx.root().begin();
        mutation.replace_element_discard_trivia(first.clone().into(), new_token.into());
        for t in &tokens[1..] {
            let empty =
                biome_markdown_syntax::MarkdownSyntaxToken::new_detached(t.kind(), "", [], []);
            mutation.replace_element_discard_trivia(t.clone().into(), empty.into());
        }
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Normalize table cell padding." }.to_owned(),
            mutation,
        ))
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

/// Add padding spaces around each cell in a table row.
fn add_cell_padding(row: &str) -> String {
    let has_leading = row.starts_with('|');
    let has_trailing = row.ends_with('|');
    let inner = row
        .strip_prefix('|')
        .unwrap_or(row)
        .strip_suffix('|')
        .unwrap_or(row.strip_prefix('|').unwrap_or(row));
    let cells: Vec<&str> = inner.split('|').collect();
    let padded: Vec<String> = cells
        .iter()
        .map(|c| {
            let trimmed = c.trim();
            if trimmed.is_empty() {
                " ".to_string()
            } else {
                format!(" {} ", trimmed)
            }
        })
        .collect();
    let mut result = String::new();
    if has_leading {
        result.push('|');
    }
    result.push_str(&padded.join("|"));
    if has_trailing {
        result.push('|');
    }
    result
}

/// Remove padding spaces from each cell in a table row.
fn remove_cell_padding(row: &str) -> String {
    let has_leading = row.starts_with('|');
    let has_trailing = row.ends_with('|');
    let inner = row
        .strip_prefix('|')
        .unwrap_or(row)
        .strip_suffix('|')
        .unwrap_or(row.strip_prefix('|').unwrap_or(row));
    let cells: Vec<&str> = inner.split('|').collect();
    let compact: Vec<String> = cells.iter().map(|c| c.trim().to_string()).collect();
    let mut result = String::new();
    if has_leading {
        result.push('|');
    }
    result.push_str(&compact.join("|"));
    if has_trailing {
        result.push('|');
    }
    result
}
