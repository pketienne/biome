use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdParagraph};
use biome_rowan::{AstNode, TextRange, TextSize};

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
    }
}

pub struct EmptyLink {
    range: TextRange,
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
                        if let Some(close_bracket) = find_matching(bytes, i, b'[', b']') {
                            // Check for empty parens immediately after
                            if close_bracket + 1 < bytes.len()
                                && bytes[close_bracket + 1] == b'('
                            {
                                if let Some(close_paren) =
                                    find_matching(bytes, close_bracket + 1, b'(', b')')
                                {
                                    let paren_content =
                                        &text[close_bracket + 2..close_paren];
                                    if paren_content.trim().is_empty() {
                                        let offset = TextSize::from(i as u32);
                                        let len =
                                            TextSize::from((close_paren - i + 1) as u32);
                                        signals.push(EmptyLink {
                                            range: TextRange::new(
                                                start + offset,
                                                start + offset + len,
                                            ),
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

fn find_matching(bytes: &[u8], start: usize, open: u8, close: u8) -> Option<usize> {
    let mut depth = 0;
    for i in start..bytes.len() {
        if bytes[i] == open {
            depth += 1;
        } else if bytes[i] == close {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}
