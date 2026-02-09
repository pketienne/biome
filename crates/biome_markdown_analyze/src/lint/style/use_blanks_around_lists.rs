use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;
use crate::utils::line_utils::is_blank_line;
use crate::utils::list_utils::collect_list_blocks;

declare_lint_rule! {
    /// Enforce blank lines around list blocks.
    ///
    /// Lists should be surrounded by blank lines to separate them
    /// from adjacent content.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// Text
    /// - item
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// Text
    ///
    /// - item
    /// ```
    pub UseBlanksAroundLists {
        version: "next",
        name: "useBlanksAroundLists",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct MissingBlankAroundList {
    range: TextRange,
    position: &'static str,
    corrected: String,
}

impl Rule for UseBlanksAroundLists {
    type Query = Ast<MdDocument>;
    type State = MissingBlankAroundList;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let blocks = collect_list_blocks(&text);
        let lines: Vec<&str> = text.lines().collect();
        let mut signals = Vec::new();

        for block in &blocks {
            // Get the text of the block for the corrected field
            let block_text = &text[block.byte_offset..block.byte_offset + block.byte_len];

            // Check line before the block
            if block.start_line > 0 {
                let prev_line = lines[block.start_line - 1];
                if !is_blank_line(prev_line) {
                    signals.push(MissingBlankAroundList {
                        range: TextRange::new(
                            base + TextSize::from(block.byte_offset as u32),
                            base + TextSize::from(
                                (block.byte_offset + block.byte_len) as u32,
                            ),
                        ),
                        position: "before",
                        corrected: format!("\n{}", block_text),
                    });
                }
            }

            // Check line after the block
            let next_line_idx = block.end_line + 1;
            if next_line_idx < lines.len() {
                let next_line = lines[next_line_idx];
                if !is_blank_line(next_line) {
                    signals.push(MissingBlankAroundList {
                        range: TextRange::new(
                            base + TextSize::from(block.byte_offset as u32),
                            base + TextSize::from(
                                (block.byte_offset + block.byte_len) as u32,
                            ),
                        ),
                        position: "after",
                        corrected: format!("{}\n", block_text),
                    });
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
            markup! { "Insert blank line "{state.position}" list block." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Missing blank line "{state.position}" list block."
                },
            )
            .note(markup! {
                "Add a blank line before and after list blocks."
            }),
        )
    }
}
