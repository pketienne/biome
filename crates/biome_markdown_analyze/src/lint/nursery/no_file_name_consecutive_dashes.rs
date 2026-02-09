use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::AstNode;

declare_lint_rule! {
    /// Disallow consecutive dashes in file names.
    ///
    /// Consecutive dashes in file names are often a mistake and make file names
    /// harder to read.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// File name: `my--file.md`
    ///
    /// ### Valid
    ///
    /// File name: `my-file.md`
    pub NoFileNameConsecutiveDashes {
        version: "next",
        name: "noFileNameConsecutiveDashes",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

impl Rule for NoFileNameConsecutiveDashes {
    type Query = Ast<MdDocument>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let file_path = ctx.file_path();
        let file_name = file_path.file_name()?;

        if file_name.contains("--") {
            Some(())
        } else {
            None
        }
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                ctx.query().syntax().text_trimmed_range(),
                markup! {
                    "File name contains consecutive dashes."
                },
            )
            .note(markup! {
                "Replace consecutive dashes with a single dash."
            }),
        )
    }
}
