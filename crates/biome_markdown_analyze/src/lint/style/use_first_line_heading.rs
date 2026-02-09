use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::use_first_line_heading::UseFirstLineHeadingOptions;

declare_lint_rule! {
    /// Require a heading as the first content in a document.
    ///
    /// Documents should start with a heading to provide a clear title
    /// and structure. The first non-blank content should be a heading
    /// of the configured level.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// Some text before any heading.
    ///
    /// # Title
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// # Title
    ///
    /// Some text after the heading.
    /// ```
    ///
    /// ## Options
    ///
    /// ### `level`
    ///
    /// Required heading level for the first heading. Default: `1`.
    pub UseFirstLineHeading {
        version: "next",
        name: "useFirstLineHeading",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub enum FirstLineIssue {
    NoHeading { range: TextRange },
    WrongLevel { range: TextRange, actual: usize },
}

impl Rule for UseFirstLineHeading {
    type Query = Ast<MdDocument>;
    type State = FirstLineIssue;
    type Signals = Option<Self::State>;
    type Options = UseFirstLineHeadingOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_trimmed().to_string();
        let base = document.syntax().text_trimmed_range().start();
        let expected_level = ctx.options().level() as usize;

        // Find the first non-blank line
        let mut offset = 0usize;
        for line in text.lines() {
            if !line.trim().is_empty() {
                // Check if this line is a heading
                let trimmed = line.trim_start();
                if trimmed.starts_with('#') {
                    let level = trimmed.chars().take_while(|&c| c == '#').count();
                    if level != expected_level {
                        return Some(FirstLineIssue::WrongLevel {
                            range: TextRange::new(
                                base + TextSize::from(offset as u32),
                                base + TextSize::from((offset + line.len()) as u32),
                            ),
                            actual: level,
                        });
                    }
                    return None; // First content is a heading at the correct level
                }

                // First non-blank line is not a heading
                return Some(FirstLineIssue::NoHeading {
                    range: TextRange::new(
                        base + TextSize::from(offset as u32),
                        base + TextSize::from((offset + line.len()) as u32),
                    ),
                });
            }
            offset += line.len() + 1;
        }

        // Empty document — no diagnostic
        None
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let expected = ctx.options().level();
        match state {
            FirstLineIssue::NoHeading { range } => Some(
                RuleDiagnostic::new(
                    rule_category!(),
                    *range,
                    markup! {
                        "First content should be a heading."
                    },
                )
                .note(markup! {
                    "Add a level "{expected.to_string()}" heading as the first content."
                }),
            ),
            FirstLineIssue::WrongLevel { range, actual } => Some(
                RuleDiagnostic::new(
                    rule_category!(),
                    *range,
                    markup! {
                        "First heading is level "{actual.to_string()}", expected "{expected.to_string()}"."
                    },
                )
                .note(markup! {
                    "Use a level "{expected.to_string()}" heading as the first heading."
                }),
            ),
        }
    }
}
