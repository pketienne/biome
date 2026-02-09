use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;

use crate::utils::line_utils::leading_indent;
use crate::utils::table_utils::collect_tables;

declare_lint_rule! {
    /// Disallow indentation in table rows.
    ///
    /// Table rows should not be indented.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    ///   | A | B |
    ///   | --- | --- |
    ///   | 1 | 2 |
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// | A | B |
    /// | --- | --- |
    /// | 1 | 2 |
    /// ```
    pub NoTableIndentation {
        version: "next",
        name: "noTableIndentation",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct IndentedTableRow {
    range: TextRange,
}

impl Rule for NoTableIndentation {
    type Query = Ast<MdDocument>;
    type State = IndentedTableRow;
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

            for &line_idx in &all_lines {
                if leading_indent(lines[line_idx]) > 0 {
                    signals.push(IndentedTableRow {
                        range: TextRange::new(
                            base + TextSize::from(offsets[line_idx] as u32),
                            base + TextSize::from((offsets[line_idx] + lines[line_idx].len()) as u32),
                        ),
                    });
                }
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();
        let token = root
            .syntax()
            .token_at_offset(state.range.start())
            .right_biased()?;
        let token_text = token.text().to_string();
        let token_start = token.text_range().start();
        let rel_start = u32::from(state.range.start() - token_start) as usize;
        // Count leading spaces in the line portion within the token
        let indent = token_text[rel_start..]
            .bytes()
            .take_while(|&b| b == b' ')
            .count();
        if indent == 0 {
            return None;
        }
        let rel_end = rel_start + indent;
        let mut new_text = String::with_capacity(token_text.len());
        new_text.push_str(&token_text[..rel_start]);
        new_text.push_str(&token_text[rel_end..]);
        let new_token = biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
            token.kind(),
            &new_text,
            [],
            [],
        );
        let mut mutation = ctx.root().begin();
        mutation.replace_element_discard_trivia(token.into(), new_token.into());
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove table row indentation." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Table rows should not be indented."
                },
            )
            .note(markup! {
                "Remove indentation from table rows."
            }),
        )
    }
}
