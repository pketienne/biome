use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use biome_rule_options::use_consistent_code_block_style::UseConsistentCodeBlockStyleOptions;

use crate::MarkdownRuleAction;
use crate::utils::fence_utils::FenceTracker;

declare_lint_rule! {
    /// Enforce consistent code block style.
    ///
    /// Code blocks can be created with either fenced code blocks (using backticks
    /// or tildes) or indented code blocks (4+ spaces). This rule enforces a
    /// consistent style.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"fenced"` (default), indented code blocks are flagged:
    ///
    /// ```md
    ///     indented code block
    /// ```
    ///
    /// ### Valid
    ///
    /// ````md
    /// ```
    /// fenced code block
    /// ```
    /// ````
    ///
    /// ## Options
    ///
    /// ### `style`
    ///
    /// Which code block style to enforce. Default: `"fenced"`.
    /// Allowed values: `"fenced"`, `"indented"`.
    pub UseConsistentCodeBlockStyle {
        version: "next",
        name: "useConsistentCodeBlockStyle",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentCodeBlockStyle {
    range: TextRange,
    corrected: String,
}

impl Rule for UseConsistentCodeBlockStyle {
    type Query = Ast<MdDocument>;
    type State = InconsistentCodeBlockStyle;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentCodeBlockStyleOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let style = ctx.options().style();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);

            if style == "fenced" && !tracker.is_inside_fence() {
                // Check for indented code blocks (4+ spaces or 1+ tab at start)
                let trimmed = line.trim_start();
                if !trimmed.is_empty() {
                    let indent = line.len() - trimmed.len();
                    let is_indented_code =
                        line.starts_with("    ") || line.starts_with('\t');
                    if is_indented_code && indent >= 4 {
                        // Remove the 4-space indent to produce the code content
                        let corrected = if line.starts_with("    ") {
                            line[4..].to_string()
                        } else if line.starts_with('\t') {
                            line[1..].to_string()
                        } else {
                            line.to_string()
                        };
                        signals.push(InconsistentCodeBlockStyle {
                            range: TextRange::new(
                                base + TextSize::from(offset as u32),
                                base + TextSize::from((offset + line.len()) as u32),
                            ),
                            corrected,
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
            markup! { "Remove indentation from code block." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Use fenced code blocks instead of indented code blocks."
                },
            )
            .note(markup! {
                "Fenced code blocks are more readable and support language specification."
            }),
        )
    }
}
