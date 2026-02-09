use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;

use biome_rule_options::no_hard_tabs::NoHardTabsOptions;

use crate::utils::fence_utils::FenceTracker;

declare_lint_rule! {
    /// Disallow hard tabs in markdown files.
    ///
    /// Hard tabs can cause inconsistent rendering across different viewers.
    /// Use spaces instead of tabs for indentation.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// - Item
    /// 	- Indented with tab
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// - Item
    ///     - Indented with spaces
    /// ```
    ///
    /// ## Options
    ///
    /// ### `allowInCodeBlocks`
    ///
    /// Whether to allow hard tabs inside fenced code blocks. Default: `false`.
    pub NoHardTabs {
        version: "next",
        name: "noHardTabs",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct HardTab {
    range: TextRange,
}

impl Rule for NoHardTabs {
    type Query = Ast<MdDocument>;
    type State = HardTab;
    type Signals = Vec<Self::State>;
    type Options = NoHardTabsOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        // Use text_with_trivia() because tabs are trivia in the markdown parser
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let allow_in_code = ctx.options().allow_in_code_blocks();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);

            let skip = allow_in_code && tracker.is_inside_fence();
            if !skip {
                for (i, byte) in line.bytes().enumerate() {
                    if byte == b'\t' {
                        let tab_offset = offset + i;
                        signals.push(HardTab {
                            range: TextRange::new(
                                base + TextSize::from(tab_offset as u32),
                                base + TextSize::from((tab_offset + 1) as u32),
                            ),
                        });
                    }
                }
            }

            offset += line.len() + 1;
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();
        let token = root
            .syntax()
            .token_at_offset(state.range.start())
            .right_biased()?;
        // The tab is trivia, so use full token text (including trivia).
        let token_text = token.text().to_string();
        let token_start = token.text_range().start();
        let rel_start = u32::from(state.range.start() - token_start) as usize;
        let rel_end = u32::from(state.range.end() - token_start) as usize;
        // Replace the tab with 4 spaces
        let mut new_text = String::with_capacity(token_text.len() + 3);
        new_text.push_str(&token_text[..rel_start]);
        new_text.push_str("    ");
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
            markup! { "Replace tab with spaces." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Hard tabs are not allowed."
                },
            )
            .note(markup! {
                "Use spaces instead of tabs for indentation."
            }),
        )
    }
}
