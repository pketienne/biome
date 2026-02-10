use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdTable;
use biome_rowan::{AstNode, BatchMutationExt, TextRange};

use biome_rule_options::use_consistent_table_cell_padding::UseConsistentTableCellPaddingOptions;

use crate::MarkdownRuleAction;

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
    type Query = Ast<MdTable>;
    type State = InconsistentCellPadding;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentTableCellPaddingOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let table = ctx.query();
        let style = ctx.options().style();
        let mut signals = Vec::new();

        let header = match table.header() {
            Ok(h) => h,
            Err(_) => return Vec::new(),
        };

        let header_text = header.syntax().text_trimmed().to_string();

        // For consistent mode, determine style from header
        let effective_style = if style == "consistent" {
            let trimmed = header_text.trim();
            if trimmed.contains("| ") || trimmed.contains(" |") {
                "padded"
            } else {
                "compact"
            }
        } else {
            style
        };

        // Check header row
        check_row_padding(
            &header_text,
            header.syntax().text_trimmed_range(),
            effective_style,
            &mut signals,
        );

        // Check data rows
        for row in table.rows() {
            let row_text = row.syntax().text_trimmed().to_string();
            check_row_padding(
                &row_text,
                row.syntax().text_trimmed_range(),
                effective_style,
                &mut signals,
            );
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

fn check_row_padding(
    row_text: &str,
    range: TextRange,
    style: &str,
    signals: &mut Vec<InconsistentCellPadding>,
) {
    let trimmed = row_text.trim();
    let is_padded = has_cell_padding(trimmed);
    match style {
        "padded" => {
            if !is_padded {
                signals.push(InconsistentCellPadding {
                    range,
                    expected: "padded",
                    corrected: add_cell_padding(trimmed),
                });
            }
        }
        "compact" => {
            if is_padded {
                signals.push(InconsistentCellPadding {
                    range,
                    expected: "compact",
                    corrected: remove_cell_padding(trimmed),
                });
            }
        }
        _ => {}
    }
}

fn has_cell_padding(row: &str) -> bool {
    let s = row.strip_prefix('|').unwrap_or(row);
    let s = s.strip_suffix('|').unwrap_or(s);
    for cell in s.split('|') {
        if !cell.is_empty() && (cell.starts_with(' ') || cell.ends_with(' ')) {
            return true;
        }
    }
    false
}

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
