use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdHeader};
use biome_rowan::{AstNode, AstNodeList};

declare_lint_rule! {
    /// Disallow duplicate headings within the same section.
    ///
    /// Headings should be unique within their parent section to avoid
    /// ambiguous anchor links.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// ## Overview
    ///
    /// ## Overview
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ## Overview
    ///
    /// ## Details
    /// ```
    pub NoDuplicateHeadingsInSection {
        version: "next",
        name: "noDuplicateHeadingsInSection",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct DuplicateHeading {
    range: biome_rowan::TextRange,
    text: String,
}

impl Rule for NoDuplicateHeadingsInSection {
    type Query = Ast<MdDocument>;
    type State = DuplicateHeading;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let mut signals = Vec::new();

        // Collect all headings with their levels and text
        let mut headings: Vec<(usize, String, biome_rowan::TextRange)> = Vec::new();

        for node in document.syntax().descendants() {
            if let Some(header) = MdHeader::cast_ref(&node) {
                let level = header.before().len();
                // Get heading text from the parent node's text, stripping hashes
                let text = header
                    .before()
                    .syntax()
                    .parent()
                    .map(|p| {
                        let full_text = p.text_trimmed().to_string();
                        full_text.get(level..).unwrap_or("").trim().to_string()
                    })
                    .unwrap_or_default();
                let range = header.syntax().text_trimmed_range();
                headings.push((level, text, range));
            }
        }

        // Check for duplicates within the same section
        for i in 0..headings.len() {
            let (level_i, ref text_i, _) = headings[i];
            for j in (i + 1)..headings.len() {
                let (level_j, ref text_j, range_j) = headings[j];
                // Stop if we hit a heading of equal or higher level (different section)
                if level_j <= level_i && j > i + 1 {
                    break;
                }
                // Same level and same text = duplicate within section
                if level_j == level_i
                    && text_j.to_ascii_lowercase() == text_i.to_ascii_lowercase()
                {
                    signals.push(DuplicateHeading {
                        range: range_j,
                        text: text_j.clone(),
                    });
                }
            }
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let text = &state.text;
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Duplicate heading \""{ text }"\" in the same section."
                },
            )
            .note(markup! {
                "Use unique headings within each section."
            }),
        )
    }
}
