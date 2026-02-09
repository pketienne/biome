use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use biome_rule_options::use_consistent_code_fence_marker::UseConsistentCodeFenceMarkerOptions;

use crate::MarkdownRuleAction;
use crate::utils::fence_utils::FenceTracker;

declare_lint_rule! {
    /// Enforce consistent code fence markers.
    ///
    /// Code fences can use either backticks (`` ` ``) or tildes (`~`).
    /// This rule enforces consistent usage.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ````md
    /// ~~~js
    /// code
    /// ~~~
    /// ````
    ///
    /// ### Valid
    ///
    /// ````md
    /// ```js
    /// code
    /// ```
    /// ````
    ///
    /// ## Options
    ///
    /// ### `marker`
    ///
    /// Which fence marker to enforce. Default: `"backtick"`.
    /// Allowed values: `"backtick"`, `"tilde"`.
    pub UseConsistentCodeFenceMarker {
        version: "next",
        name: "useConsistentCodeFenceMarker",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentFenceMarker {
    range: TextRange,
    expected: char,
    actual: char,
    corrected: String,
}

impl Rule for UseConsistentCodeFenceMarker {
    type Query = Ast<MdDocument>;
    type State = InconsistentFenceMarker;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentCodeFenceMarkerOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let expected_char = match ctx.options().marker() {
            "tilde" => '~',
            _ => '`',
        };
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();

        for (line_idx, line) in text.lines().enumerate() {
            if let Some(fence_open) = tracker.process_line(line_idx, line) {
                if fence_open.fence_char != expected_char {
                    let line_start: usize = text.lines().take(line_idx).map(|l| l.len() + 1).sum();
                    let trimmed_start = line.len() - line.trim_start().len();
                    let fence_byte_start = line_start + trimmed_start;
                    let fence_byte_end = fence_byte_start + fence_open.fence_count;
                    let corrected: String =
                        std::iter::repeat(expected_char).take(fence_open.fence_count).collect();
                    signals.push(InconsistentFenceMarker {
                        range: TextRange::new(
                            base + TextSize::from(fence_byte_start as u32),
                            base + TextSize::from(fence_byte_end as u32),
                        ),
                        expected: expected_char,
                        actual: fence_open.fence_char,
                        corrected,
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
            markup! { "Use the consistent code fence marker." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let expected_name = if state.expected == '`' {
            "backticks"
        } else {
            "tildes"
        };
        let actual_name = if state.actual == '`' {
            "backticks"
        } else {
            "tildes"
        };
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Expected "{expected_name}" but found "{actual_name}" for code fence marker."
                },
            )
            .note(markup! {
                "Use consistent code fence markers throughout the document."
            }),
        )
    }
}
