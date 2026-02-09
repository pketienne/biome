use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdParagraph};
use biome_rowan::{AstNode, TextRange, TextSize};

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
    }
}

pub struct ReversedLink {
    range: TextRange,
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
                                        signals.push(ReversedLink {
                                            range: TextRange::new(
                                                start + offset,
                                                start + offset + len,
                                            ),
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
