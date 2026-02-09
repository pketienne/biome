use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;
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
        fix_kind: FixKind::Safe,
    }
}

pub struct HiddenTableCell {
    range: TextRange,
    extra_count: usize,
    expected_columns: usize,
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
                        expected_columns: expected,
                    });
                }
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let line_start = u32::from(state.range.start() - base) as usize;
        let line_end = u32::from(state.range.end() - base) as usize;
        let line_text = &text[line_start..line_end];

        let trimmed = line_text.trim();
        let has_leading_pipe = trimmed.starts_with('|');
        let has_trailing_pipe = trimmed.ends_with('|');

        // Split the row into cells and keep only the expected number
        let all_cells = split_table_cells(trimmed);
        let kept_cells: Vec<&str> = all_cells
            .iter()
            .take(state.expected_columns)
            .map(|s| s.as_str())
            .collect();

        // Reconstruct the row preserving the original pipe style
        let mut corrected = String::new();
        if has_leading_pipe {
            corrected.push_str("| ");
        }
        corrected.push_str(&kept_cells.join(" | "));
        if has_trailing_pipe {
            corrected.push_str(" |");
        }

        // Replace the line content in the token
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
        let new_text = format!("{}{}{}", prefix, corrected, suffix);
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
