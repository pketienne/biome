use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use crate::MarkdownRuleAction;
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdParagraph};
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::utils::inline_utils::{find_matching_bracket, looks_like_url};

declare_lint_rule! {
    /// Disallow reversed link syntax.
    ///
    /// Detects `(text)[url]` which should be `[text](url)`. This is a common
    /// mistake when writing Markdown links.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// (click here)[https://example.com]
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [click here](https://example.com)
    /// ```
    pub NoReversedLinks {
        version: "next",
        name: "noReversedLinks",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct ReversedLink {
    range: TextRange,
    corrected: String,
}

impl Rule for NoReversedLinks {
    type Query = Ast<MdDocument>;
    type State = ReversedLink;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let mut signals = Vec::new();

        // Walk through all paragraphs looking for reversed link patterns
        for node in document.syntax().descendants() {
            if let Some(paragraph) = MdParagraph::cast_ref(&node) {
                let text = paragraph.syntax().text_trimmed().to_string();
                let start = paragraph.syntax().text_trimmed_range().start();
                // Simple text-based detection of reversed links: (text)[url]
                let bytes = text.as_bytes();
                let mut i = 0;
                while i < bytes.len() {
                    if bytes[i] == b'(' {
                        if let Some(close_paren) =
                            find_matching_bracket(bytes, i, b'(', b')')
                        {
                            // Check if immediately followed by [
                            if close_paren + 1 < bytes.len() && bytes[close_paren + 1] == b'[' {
                                if let Some(close_bracket) =
                                    find_matching_bracket(bytes, close_paren + 1, b'[', b']')
                                {
                                    let paren_content = &text[i + 1..close_paren];
                                    let bracket_content = &text[close_paren + 2..close_bracket];

                                    if !paren_content.is_empty()
                                        && !bracket_content.is_empty()
                                        && looks_like_url(bracket_content)
                                    {
                                        let offset = TextSize::from(i as u32);
                                        let len = TextSize::from((close_bracket - i + 1) as u32);
                                        let corrected = format!(
                                            "[{}]({})",
                                            paren_content, bracket_content
                                        );
                                        signals.push(ReversedLink {
                                            range: TextRange::new(
                                                start + offset,
                                                start + offset + len,
                                            ),
                                            corrected,
                                        });
                                    }
                                }
                            }
                        }
                    }
                    i += 1;
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
            markup! { "Fix the link syntax." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Reversed link syntax detected. Use "<Emphasis>"[text](url)"</Emphasis>" instead of "<Emphasis>"(text)[url]"</Emphasis>"."
                },
            )
            .note(markup! {
                "Swap the parentheses and brackets to fix the link syntax."
            }),
        )
    }
}
