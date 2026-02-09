use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdHeader};
use biome_rowan::{AstNode, AstNodeList, BatchMutationExt, TextRange};

use crate::MarkdownRuleAction;

use biome_rule_options::no_heading_trailing_punctuation::NoHeadingTrailingPunctuationOptions;

declare_lint_rule! {
    /// Disallow trailing punctuation in headings.
    ///
    /// Headings should not end with punctuation characters such as periods,
    /// commas, semicolons, colons, exclamation marks, or question marks.
    /// These are typically unnecessary in headings and indicate the text
    /// may not be a proper heading.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// # This is a heading.
    /// ## Another heading:
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// # This is a heading
    /// ## Another heading
    /// ```
    ///
    /// ## Options
    ///
    /// ### `punctuation`
    ///
    /// Characters considered trailing punctuation. Default: `".,;:!?"`.
    pub NoHeadingTrailingPunctuation {
        version: "next",
        name: "noHeadingTrailingPunctuation",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct TrailingPunctuation {
    range: TextRange,
    character: char,
}

impl Rule for NoHeadingTrailingPunctuation {
    type Query = Ast<MdDocument>;
    type State = TrailingPunctuation;
    type Signals = Vec<Self::State>;
    type Options = NoHeadingTrailingPunctuationOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let punctuation = ctx.options().punctuation();
        let mut signals = Vec::new();

        for node in document.syntax().descendants() {
            if let Some(header) = MdHeader::cast_ref(&node) {
                let content = header.before().syntax().parent().map(|p| {
                    // Get the full text of the header line after the hashes
                    let header_text = p.text_trimmed().to_string();
                    let hashes = header.before().len();
                    header_text
                        .get(hashes..)
                        .unwrap_or("")
                        .trim()
                        .to_string()
                });

                if let Some(text) = content {
                    if let Some(last_char) = text.trim_end().chars().last() {
                        if punctuation.contains(last_char) {
                            signals.push(TrailingPunctuation {
                                range: header.syntax().text_trimmed_range(),
                                character: last_char,
                            });
                        }
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
                // Find the content paragraph's last token and remove trailing punctuation
                let content = header.content()?;
                let last_token = content.syntax().last_token()?;
                let token_text = last_token.text().to_string();
                let trimmed_end = token_text.trim_end();
                if !trimmed_end.ends_with(state.character) {
                    return None;
                }
                let char_len = state.character.len_utf8();
                let new_text = format!(
                    "{}{}",
                    &trimmed_end[..trimmed_end.len() - char_len],
                    &token_text[trimmed_end.len()..]
                );
                let new_token = biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
                    last_token.kind(),
                    &new_text,
                    [],
                    [],
                );
                let mut mutation = ctx.root().begin();
                mutation.replace_element_discard_trivia(last_token.into(), new_token.into());
                return Some(RuleAction::new(
                    ctx.metadata().action_category(ctx.category(), ctx.group()),
                    ctx.metadata().applicability(),
                    markup! { "Remove trailing punctuation." }.to_owned(),
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
                    "Heading ends with trailing punctuation '"{ state.character.to_string() }"'."
                },
            )
            .note(markup! {
                "Remove trailing punctuation from headings."
            }),
        )
    }
}
