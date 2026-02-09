use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::AstNode;

declare_lint_rule! {
    /// Disallow dashes at the start or end of file names.
    ///
    /// Dashes at the start or end of a file stem are often unintended
    /// and can cause issues with certain tools.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// File name: `-readme.md`
    /// File name: `readme-.md`
    ///
    /// ### Valid
    ///
    /// File name: `readme.md`
    pub NoFileNameOuterDashes {
        version: "next",
        name: "noFileNameOuterDashes",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

impl Rule for NoFileNameOuterDashes {
    type Query = Ast<MdDocument>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let file_path = ctx.file_path();
        let file_stem = file_path.file_stem()?;

        if file_stem.starts_with('-') || file_stem.ends_with('-') {
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
                    "File name starts or ends with a dash."
                },
            )
            .note(markup! {
                "Remove the leading or trailing dash from the file name."
            }),
        )
    }
}
