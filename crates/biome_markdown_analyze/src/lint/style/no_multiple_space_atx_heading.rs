use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdHeader};
use biome_rowan::{AstNode, AstNodeList, BatchMutationExt, TextRange};

use crate::MarkdownRuleAction;

declare_lint_rule! {
    /// Disallow multiple spaces after hash characters in atx headings.
    ///
    /// Atx-style headings should have exactly one space between the hash characters
    /// and the heading text. Multiple spaces are likely a typo.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// #  Heading with extra space
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// # Heading with one space
    /// ```
    pub NoMultipleSpaceAtxHeading {
        version: "next",
        name: "noMultipleSpaceAtxHeading",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct MultipleSpaceAtxHeading {
    range: TextRange,
}

impl Rule for NoMultipleSpaceAtxHeading {
    type Query = Ast<MdDocument>;
    type State = MultipleSpaceAtxHeading;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let mut signals = Vec::new();

        for node in document.syntax().descendants() {
            if let Some(header) = MdHeader::cast_ref(&node) {
                let level = header.before().len();
                if level == 0 {
                    continue;
                }
                let text = header.syntax().text_trimmed().to_string();
                if text.len() > level + 1 {
                    let after_hashes = &text[level..];
                    // Check if there are multiple spaces after the hash
                    if after_hashes.starts_with("  ") {
                        signals.push(MultipleSpaceAtxHeading {
                            range: header.syntax().text_trimmed_range(),
                        });
                    }
                }
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        // Find the MdHeader node that corresponds to this state
        let document = ctx.query();
        for node in document.syntax().descendants() {
            if let Some(header) = MdHeader::cast_ref(&node) {
                if header.syntax().text_trimmed_range() != state.range {
                    continue;
                }
                // Find the content paragraph's first token -- the extra spaces are
                // in its leading trivia
                let content = header.content()?;
                let first_token = content.syntax().first_token()?;
                let token_text = first_token.text().to_string();
                // Count leading spaces in the full token text (including trivia)
                let space_count = token_text.bytes().take_while(|&b| b == b' ').count();
                if space_count <= 1 {
                    return None;
                }
                // Replace multiple leading spaces with a single space
                let new_text = format!(" {}", &token_text[space_count..]);
                let new_token = biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
                    first_token.kind(),
                    &new_text,
                    [],
                    [],
                );
                let mut mutation = ctx.root().begin();
                mutation.replace_element_discard_trivia(first_token.into(), new_token.into());
                return Some(RuleAction::new(
                    ctx.metadata().action_category(ctx.category(), ctx.group()),
                    ctx.metadata().applicability(),
                    markup! { "Use a single space after the hash characters." }.to_owned(),
                    mutation,
                ));
            }
        }
        None
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Multiple spaces after hash characters in atx heading."
                },
            )
            .note(markup! {
                "Use a single space between the hash characters and the heading text."
            }),
        )
    }
}
