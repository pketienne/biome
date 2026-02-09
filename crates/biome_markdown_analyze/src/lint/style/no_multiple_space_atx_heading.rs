use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdHeader};
use biome_rowan::{AstNode, AstNodeList, TextRange};

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
