use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdTable;
use biome_rowan::{AstNode, BatchMutationExt, TextRange};

use crate::MarkdownRuleAction;

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
        fix_kind: FixKind::Safe,
    }
}

pub struct HiddenTableCell {
    range: TextRange,
    extra_count: usize,
    corrected: String,
}

fn split_cells(row_text: &str) -> Vec<String> {
    let trimmed = row_text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let s = if trimmed.starts_with('|') {
        &trimmed[1..]
    } else {
        trimmed
    };
    let s = if s.ends_with('|') {
        &s[..s.len() - 1]
    } else {
        s
    };
    s.split('|').map(|c| c.trim().to_string()).collect()
}

fn build_corrected_row(row_text: &str, expected_columns: usize) -> String {
    let trimmed = row_text.trim();
    let has_leading_pipe = trimmed.starts_with('|');
    let has_trailing_pipe = trimmed.ends_with('|');
    let all_cells = split_cells(trimmed);
    let kept_cells: Vec<&str> = all_cells
        .iter()
        .take(expected_columns)
        .map(|s| s.as_str())
        .collect();
    let mut corrected = String::new();
    if has_leading_pipe {
        corrected.push_str("| ");
    }
    corrected.push_str(&kept_cells.join(" | "));
    if has_trailing_pipe {
        corrected.push_str(" |");
    }
    corrected
}

impl Rule for NoHiddenTableCell {
    type Query = Ast<MdTable>;
    type State = HiddenTableCell;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let table = ctx.query();
        let separator = match table.separator() {
            Ok(sep) => sep,
            Err(_) => return Vec::new(),
        };
        let expected = split_cells(&separator.syntax().text_trimmed().to_string()).len();

        let mut signals = Vec::new();
        for row in table.rows() {
            let row_text = row.syntax().text_trimmed().to_string();
            let actual = split_cells(&row_text).len();
            if actual > expected {
                signals.push(HiddenTableCell {
                    range: row.syntax().text_trimmed_range(),
                    extra_count: actual - expected,
                    corrected: build_corrected_row(&row_text, expected),
                });
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
            let empty = biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
                t.kind(),
                "",
                [],
                [],
            );
            mutation.replace_element_discard_trivia(t.clone().into(), empty.into());
        }
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove the extra table cells." }.to_owned(),
            mutation,
        ))
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
