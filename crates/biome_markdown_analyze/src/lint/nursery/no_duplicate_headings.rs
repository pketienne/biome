use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdHeader};
use biome_rowan::{AstNode, TextRange};
use std::collections::HashMap;

declare_lint_rule! {
    /// Disallow duplicate heading text.
    ///
    /// Duplicate headings make it harder to navigate a document, especially
    /// when generating a table of contents. Each heading should be unique.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// # Title
    /// ## Section
    /// ## Section
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// # Title
    /// ## Section A
    /// ## Section B
    /// ```
    pub NoDuplicateHeadings {
        version: "next",
        name: "noDuplicateHeadings",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct DuplicateHeading {
    text: String,
    range: TextRange,
}

impl Rule for NoDuplicateHeadings {
    type Query = Ast<MdDocument>;
    type State = DuplicateHeading;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let mut seen: HashMap<String, TextRange> = HashMap::new();
        let mut signals = Vec::new();

        for node in document.syntax().descendants() {
            if let Some(header) = MdHeader::cast_ref(&node) {
                let text = header
                    .syntax()
                    .text_trimmed()
                    .to_string();

                // Extract just the heading text (after the # characters)
                let text = text.trim().to_string();

                if text.is_empty() {
                    continue;
                }

                if seen.contains_key(&text) {
                    signals.push(DuplicateHeading {
                        text: text.clone(),
                        range: header.syntax().text_trimmed_range(),
                    });
                } else {
                    seen.insert(text, header.syntax().text_trimmed_range());
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
                    "Duplicate heading text \""{ &state.text }"\"."
                },
            )
            .note(markup! {
                "Each heading in a document should have unique text."
            }),
        )
    }
}
