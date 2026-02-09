use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;

use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{find_code_spans, find_matching_bracket, is_in_code_span};

declare_lint_rule! {
    /// Disallow spaces inside link text brackets.
    ///
    /// Link text should not have leading or trailing spaces inside the brackets.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [ link text ](url)
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [link text](url)
    /// ```
    pub NoSpaceInLinks {
        version: "next",
        name: "noSpaceInLinks",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct SpaceInLink {
    range: TextRange,
    corrected: String,
}

impl Rule for NoSpaceInLinks {
    type Query = Ast<MdDocument>;
    type State = SpaceInLink;
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
            if tracker.is_inside_fence() {
                offset += line.len() + 1;
                continue;
            }

            let bytes = line.as_bytes();
            let code_spans = find_code_spans(line);
            let mut i = 0;

            while i < bytes.len() {
                if bytes[i] == b'[' && !is_in_code_span(i, &code_spans) {
                    // Check for image links starting with ![
                    let bracket_start = i;
                    if let Some(bracket_end) = find_matching_bracket(bytes, i, b'[', b']') {
                        // Check if followed by ( for inline link
                        if bracket_end + 1 < bytes.len() && bytes[bracket_end + 1] == b'(' {
                            let text_start = bracket_start + 1;
                            let text_end = bracket_end;

                            if text_start < text_end {
                                let has_leading = bytes[text_start] == b' ';
                                let has_trailing = bytes[text_end - 1] == b' ';

                                if has_leading || has_trailing {
                                    let inner = &line[text_start..text_end];
                                    let corrected = format!("[{}]", inner.trim());
                                    signals.push(SpaceInLink {
                                        range: TextRange::new(
                                            base + TextSize::from(
                                                (offset + bracket_start) as u32,
                                            ),
                                            base + TextSize::from(
                                                (offset + bracket_end + 1) as u32,
                                            ),
                                        ),
                                        corrected,
                                    });
                                }
                            }
                        }
                        i = bracket_end + 1;
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }

            offset += line.len() + 1;
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();

        // Collect all tokens overlapping the range
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
            markup! { "Remove spaces from link text." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Unexpected space inside link text brackets."
                },
            )
            .note(markup! {
                "Remove leading or trailing spaces from the link text."
            }),
        )
    }
}
