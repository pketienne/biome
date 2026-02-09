use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;
use crate::utils::list_utils::collect_list_blocks;

declare_lint_rule! {
    /// Enforce consistent indentation for list items at the same level.
    ///
    /// All list items within the same list block should have the same
    /// indentation level.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// - item one
    ///   - item two
    /// - item three
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// - item one
    /// - item two
    /// - item three
    /// ```
    pub UseConsistentListIndent {
        version: "next",
        name: "useConsistentListIndent",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentListIndent {
    range: TextRange,
    expected: usize,
    actual: usize,
    corrected: String,
}

impl Rule for UseConsistentListIndent {
    type Query = Ast<MdDocument>;
    type State = InconsistentListIndent;
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
            if block.items.len() < 2 {
                continue;
            }

            // Find the most common indent level in this block
            let first_indent = block.items[0].indent;

            for item in block.items.iter().skip(1) {
                // Only flag items that should be at the same level as first
                // (same indent depth) but have different indent
                if item.indent != first_indent
                    && item.indent < first_indent + 2
                    && item.indent > first_indent.saturating_sub(2)
                {
                    // Build corrected line: replace leading whitespace with expected indent
                    let line = lines[item.line_index];
                    let trimmed = line.trim_start();
                    let corrected = format!("{}{}", " ".repeat(first_indent), trimmed);

                    signals.push(InconsistentListIndent {
                        range: TextRange::new(
                            base + TextSize::from(item.byte_offset as u32),
                            base + TextSize::from((item.byte_offset + item.byte_len) as u32),
                        ),
                        expected: first_indent,
                        actual: item.indent,
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
            markup! { "Fix list item indentation." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Expected indentation of "{state.expected}" but found "{state.actual}"."
                },
            )
            .note(markup! {
                "Use consistent indentation for list items at the same level."
            }),
        )
    }
}
