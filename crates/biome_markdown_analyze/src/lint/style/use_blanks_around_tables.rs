use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdTable;
use biome_rowan::{AstNode, AstNodeList, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;
use crate::utils::line_utils::is_blank_line;

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
        fix_kind: FixKind::Safe,
    }
}

pub struct MissingBlankAroundTable {
    range: TextRange,
    position: &'static str,
    corrected: String,
}

impl Rule for UseBlanksAroundTables {
    type Query = Ast<MdTable>;
    type State = MissingBlankAroundTable;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let table = ctx.query();
        let mut signals = Vec::new();

        let header = match table.header() {
            Ok(h) => h,
            Err(_) => return Vec::new(),
        };

        let root = ctx.root();
        let full_text = root.syntax().text_with_trivia().to_string();
        let base = root.syntax().text_range_with_trivia().start();
        let lines: Vec<&str> = full_text.lines().collect();
        let mut offsets = Vec::with_capacity(lines.len());
        let mut offset = 0usize;
        for line in &lines {
            offsets.push(offset);
            offset += line.len() + 1;
        }

        // Find the header line index
        let header_start =
            u32::from(header.syntax().text_trimmed_range().start() - base) as usize;
        let header_line_idx = match offsets.iter().rposition(|&o| o <= header_start) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        // Check line before header
        if header_line_idx > 0 && !is_blank_line(lines[header_line_idx - 1]) {
            let line = lines[header_line_idx];
            signals.push(MissingBlankAroundTable {
                range: TextRange::new(
                    base + TextSize::from(offsets[header_line_idx] as u32),
                    base + TextSize::from((offsets[header_line_idx] + line.len()) as u32),
                ),
                position: "before",
                corrected: format!("\n{}", line),
            });
        }

        // Find the last row line index
        let last_row = table.rows().iter().last();
        let last_row_range = if let Some(ref lr) = last_row {
            lr.syntax().text_trimmed_range()
        } else if let Ok(ref sep) = table.separator() {
            sep.syntax().text_trimmed_range()
        } else {
            header.syntax().text_trimmed_range()
        };

        let last_start = u32::from(last_row_range.start() - base) as usize;
        let last_line_idx = match offsets.iter().rposition(|&o| o <= last_start) {
            Some(idx) => idx,
            None => return signals,
        };

        // Check line after last row
        if last_line_idx + 1 < lines.len() && !is_blank_line(lines[last_line_idx + 1]) {
            let line = lines[last_line_idx];
            signals.push(MissingBlankAroundTable {
                range: TextRange::new(
                    base + TextSize::from(offsets[last_line_idx] as u32),
                    base + TextSize::from((offsets[last_line_idx] + line.len()) as u32),
                ),
                position: "after",
                corrected: format!("{}\n", line),
            });
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
            markup! { "Insert blank line "{state.position}" table." }.to_owned(),
            mutation,
        ))
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
