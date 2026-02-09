use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdParagraph};
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;
use crate::utils::inline_utils::find_matching_bracket;

declare_lint_rule! {
    /// Disallow links with empty URLs.
    ///
    /// Links with empty URLs (`[text]()`) are broken and do not navigate anywhere.
    /// Either provide a valid URL or remove the link.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [click here]()
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [click here](https://example.com)
    /// ```
    pub NoEmptyLinks {
        version: "next",
        name: "noEmptyLinks",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct EmptyLink {
    range: TextRange,
    corrected: String,
}

impl Rule for NoEmptyLinks {
    type Query = Ast<MdDocument>;
    type State = EmptyLink;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let mut signals = Vec::new();

        for node in document.syntax().descendants() {
            if let Some(paragraph) = MdParagraph::cast_ref(&node) {
                let text = paragraph.syntax().text_trimmed().to_string();
                let start = paragraph.syntax().text_trimmed_range().start();
                let bytes = text.as_bytes();
                let mut i = 0;

                while i < bytes.len() {
                    // Look for [text]()
                    if bytes[i] == b'[' {
                        if let Some(close_bracket) =
                            find_matching_bracket(bytes, i, b'[', b']')
                        {
                            // Check for empty parens immediately after
                            if close_bracket + 1 < bytes.len()
                                && bytes[close_bracket + 1] == b'('
                            {
                                if let Some(close_paren) =
                                    find_matching_bracket(bytes, close_bracket + 1, b'(', b')')
                                {
                                    let paren_content =
                                        &text[close_bracket + 2..close_paren];
                                    if paren_content.trim().is_empty() {
                                        let link_text = &text[i + 1..close_bracket];
                                        let offset = TextSize::from(i as u32);
                                        let len =
                                            TextSize::from((close_paren - i + 1) as u32);
                                        signals.push(EmptyLink {
                                            range: TextRange::new(
                                                start + offset,
                                                start + offset + len,
                                            ),
                                            corrected: link_text.to_string(),
                                        });
                                    }
                                    i = close_paren + 1;
                                    continue;
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
            markup! { "Remove the empty link." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Link has an empty URL."
                },
            )
            .note(markup! {
                "Provide a valid URL for the link or remove it."
            }),
        )
    }
}
