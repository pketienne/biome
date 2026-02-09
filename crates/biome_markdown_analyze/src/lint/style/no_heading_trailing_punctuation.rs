use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdHeader};
use biome_rowan::{AstNode, AstNodeList, TextRange};

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
