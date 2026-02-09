use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdHeader};
use biome_rowan::{AstNode, AstNodeList};

use biome_rule_options::no_long_headings::NoLongHeadingsOptions;

declare_lint_rule! {
    /// Enforce a maximum heading length.
    ///
    /// Headings that are too long are hard to scan and may indicate
    /// that the content should be a paragraph instead.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// ## This is a heading that is way too long and should probably be shortened
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ## Short heading
    /// ```
    ///
    /// ## Options
    ///
    /// ### `maxLength`
    ///
    /// Maximum number of characters. Default: `60`.
    pub NoLongHeadings {
        version: "next",
        name: "noLongHeadings",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct LongHeading {
    range: biome_rowan::TextRange,
    length: usize,
    max: u32,
}

impl Rule for NoLongHeadings {
    type Query = Ast<MdDocument>;
    type State = LongHeading;
    type Signals = Vec<Self::State>;
    type Options = NoLongHeadingsOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let max_length = ctx.options().max_length();
        let mut signals = Vec::new();

        for node in document.syntax().descendants() {
            if let Some(header) = MdHeader::cast_ref(&node) {
                let level = header.before().len();
                let text = header
                    .before()
                    .syntax()
                    .parent()
                    .map(|p| {
                        let full_text = p.text_trimmed().to_string();
                        full_text.get(level..).unwrap_or("").trim().to_string()
                    })
                    .unwrap_or_default();

                let length = text.len();
                if length as u32 > max_length {
                    signals.push(LongHeading {
                        range: header.syntax().text_trimmed_range(),
                        length,
                        max: max_length,
                    });
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
                    "Heading is "{state.length}" characters long, exceeding the maximum of "{state.max}"."
                },
            )
            .note(markup! {
                "Shorten the heading or move content to the paragraph body."
            }),
        )
    }
}
