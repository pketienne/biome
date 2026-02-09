use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use crate::MarkdownRuleAction;
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::utils::fence_utils::FenceTracker;

declare_lint_rule! {
    /// Disallow spacing issues in link reference definitions.
    ///
    /// Definitions should not have extra whitespace between the label,
    /// colon, and URL.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [foo]:   https://example.com
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [foo]: https://example.com
    /// ```
    pub NoDefinitionSpacingIssues {
        version: "next",
        name: "noDefinitionSpacingIssues",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct DefinitionSpacingIssue {
    range: TextRange,
    corrected: String,
}

impl Rule for NoDefinitionSpacingIssues {
    type Query = Ast<MdDocument>;
    type State = DefinitionSpacingIssue;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);

            if !tracker.is_inside_fence() {
                let trimmed = line.trim_start();
                let indent = line.len() - trimmed.len();

                // Check if this looks like a definition
                if indent <= 3 && trimmed.starts_with('[') {
                    if let Some(bracket_end) = trimmed.find("]:") {
                        let after_colon = &trimmed[bracket_end + 2..];
                        // Check for multiple spaces/tabs after ":"
                        let space_count = after_colon
                            .bytes()
                            .take_while(|&b| b == b' ' || b == b'\t')
                            .count();

                        if space_count > 1 {
                            let url_part = after_colon.trim_start();
                            let corrected = format!(
                                "{}: {}",
                                &trimmed[..bracket_end + 1],
                                url_part
                            );
                            // Re-add original indent
                            let corrected = format!(
                                "{}{}",
                                &line[..indent],
                                corrected
                            );
                            signals.push(DefinitionSpacingIssue {
                                range: TextRange::new(
                                    base + TextSize::from(offset as u32),
                                    base + TextSize::from((offset + line.len()) as u32),
                                ),
                                corrected,
                            });
                        }
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
            markup! { "Normalize definition spacing." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Extra whitespace in link reference definition."
                },
            )
            .note(markup! {
                "Use a single space after the colon in definitions."
            }),
        )
    }
}
