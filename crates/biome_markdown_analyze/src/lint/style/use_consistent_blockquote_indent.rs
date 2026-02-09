use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;
use crate::utils::blockquote_utils::collect_blockquote_blocks;

declare_lint_rule! {
    /// Enforce consistent blockquote indentation.
    ///
    /// All lines in a blockquote should have the same number of spaces
    /// after the `>` marker.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// > first line
    /// >  second line with extra space
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// > first line
    /// > second line
    /// ```
    pub UseConsistentBlockquoteIndent {
        version: "next",
        name: "useConsistentBlockquoteIndent",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentBlockquoteIndent {
    range: TextRange,
    expected: usize,
    actual: usize,
    corrected: String,
}

impl Rule for UseConsistentBlockquoteIndent {
    type Query = Ast<MdDocument>;
    type State = InconsistentBlockquoteIndent;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let blocks = collect_blockquote_blocks(&text);
        let lines: Vec<&str> = text.lines().collect();
        let mut signals = Vec::new();

        for block in &blocks {
            // Find the expected spacing (from the first marker line)
            let first_marker_line = block.lines.iter().find(|l| l.has_marker);
            let expected = match first_marker_line {
                Some(l) => l.spaces_after_marker,
                None => continue,
            };

            for line in &block.lines {
                if line.has_marker && line.spaces_after_marker != expected {
                    // Reconstruct the line with corrected spacing
                    let raw_line = if line.line_index < lines.len() {
                        lines[line.line_index]
                    } else {
                        continue;
                    };
                    // Find the `>` marker position
                    let marker_pos = raw_line.find('>').unwrap_or(0);
                    let before_marker = &raw_line[..marker_pos + 1]; // up to and including >
                    let spaces: String = " ".repeat(expected);
                    let corrected =
                        format!("{}{}{}", before_marker, spaces, line.content);

                    signals.push(InconsistentBlockquoteIndent {
                        range: TextRange::new(
                            base + TextSize::from(line.byte_offset as u32),
                            base + TextSize::from((line.byte_offset + line.byte_len) as u32),
                        ),
                        expected,
                        actual: line.spaces_after_marker,
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
            markup! { "Normalize blockquote indentation." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Expected "{state.expected}" space(s) after > but found "{state.actual}"."
                },
            )
            .note(markup! {
                "Use consistent spacing after the blockquote marker."
            }),
        )
    }
}
