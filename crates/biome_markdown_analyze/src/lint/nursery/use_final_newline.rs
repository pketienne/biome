use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

declare_lint_rule! {
    /// Require files to end with a single newline character.
    ///
    /// Files should end with a single trailing newline to ensure
    /// consistency and compatibility with tools that expect POSIX line endings.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// # Hello World
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// # Hello World⏎
    /// ```
    pub UseFinalNewline {
        version: "next",
        name: "useFinalNewline",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct MissingFinalNewline {
    range: TextRange,
}

impl Rule for UseFinalNewline {
    type Query = Ast<MdDocument>;
    type State = MissingFinalNewline;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        // Use text_with_trivia() to include trailing whitespace/newlines
        let text = document.syntax().text_with_trivia().to_string();

        if text.is_empty() {
            return None;
        }

        if !text.ends_with('\n') {
            let range = document.syntax().text_range_with_trivia();
            let end = range.end();
            let start = end - TextSize::from(1);
            return Some(MissingFinalNewline {
                range: TextRange::new(start, end),
            });
        }

        None
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "File does not end with a newline."
                },
            )
            .note(markup! {
                "Add a trailing newline at the end of the file."
            }),
        )
    }
}
