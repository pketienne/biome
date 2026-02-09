use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdHeader};
use biome_rowan::{AstNode, AstNodeList, BatchMutationExt, TextRange};

use crate::MarkdownRuleAction;

declare_lint_rule! {
    /// Require a space after the hash characters in atx headings.
    ///
    /// Atx-style headings require a space between the hash characters and the
    /// heading text. Without the space, some parsers may not recognize the heading.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// #Heading without space
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// # Heading with space
    /// ```
    pub NoMissingSpaceAtxHeading {
        version: "next",
        name: "noMissingSpaceAtxHeading",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct MissingSpaceAtxHeading {
    range: TextRange,
}

impl Rule for NoMissingSpaceAtxHeading {
    type Query = Ast<MdDocument>;
    type State = MissingSpaceAtxHeading;
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
                // Get the full text of the header node
                let text = header.syntax().text_trimmed().to_string();
                // After the '#' characters, there should be a space
                if text.len() > level {
                    let after_hashes = &text[level..];
                    if !after_hashes.starts_with(' ') && !after_hashes.trim().is_empty() {
                        signals.push(MissingSpaceAtxHeading {
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
                // Find the content paragraph's first token and add a leading space
                let content = header.content()?;
                let first_token = content.syntax().first_token()?;
                let token_text = first_token.text().to_string();
                // Prepend a space to the full token text
                let new_text = format!(" {}", token_text);
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
                    markup! { "Add a space after the hash characters." }.to_owned(),
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
                    "Missing space after hash characters in atx heading."
                },
            )
            .note(markup! {
                "Add a space between the hash characters and the heading text."
            }),
        )
    }
}
