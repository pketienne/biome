use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

declare_lint_rule! {
    /// Disallow extra spaces between heading markers and content.
    ///
    /// ATX headings should have exactly one space between the `#` markers
    /// and the heading content. This rule catches multiple spaces after
    /// the `#` markers.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// #  Too many spaces
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// # Just right
    /// ```
    pub NoHeadingContentIndent {
        version: "next",
        name: "noHeadingContentIndent",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct ExtraContentIndent {
    range: TextRange,
}

impl Rule for NoHeadingContentIndent {
    type Query = Ast<MdDocument>;
    type State = ExtraContentIndent;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let mut signals = Vec::new();

        // Walk the raw text for ATX headings with multiple spaces after #
        let mut offset = 0usize;
        for line in text.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
                if hash_count >= 1 && hash_count <= 6 {
                    let after_hashes = &trimmed[hash_count..];
                    // Count spaces after the hashes
                    let space_count = after_hashes
                        .bytes()
                        .take_while(|&b| b == b' ')
                        .count();
                    if space_count > 1 && !after_hashes.trim().is_empty() {
                        let indent = line.len() - trimmed.len();
                        let extra_start = offset + indent + hash_count + 1; // after first space
                        let extra_end = offset + indent + hash_count + space_count;
                        signals.push(ExtraContentIndent {
                            range: TextRange::new(
                                base + TextSize::from(extra_start as u32),
                                base + TextSize::from(extra_end as u32),
                            ),
                        });
                    }
                }
            }
            offset += line.len() + 1;
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Extra spaces between heading marker and content."
                },
            )
            .note(markup! {
                "Use exactly one space between the heading marker and content."
            }),
        )
    }
}
