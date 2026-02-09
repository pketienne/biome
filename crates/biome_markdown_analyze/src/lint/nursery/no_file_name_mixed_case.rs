use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::AstNode;

declare_lint_rule! {
    /// Disallow mixed case in file names.
    ///
    /// File names that mix uppercase and lowercase letters can cause confusion
    /// and issues on case-insensitive file systems. Use a consistent case
    /// for all file names.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// File name: `myFile.md`
    ///
    /// ### Valid
    ///
    /// File name: `my-file.md`
    /// File name: `MY-FILE.md`
    pub NoFileNameMixedCase {
        version: "next",
        name: "noFileNameMixedCase",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

impl Rule for NoFileNameMixedCase {
    type Query = Ast<MdDocument>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let file_path = ctx.file_path();
        let file_stem = file_path.file_stem()?;

        let has_upper = file_stem.chars().any(|c| c.is_uppercase());
        let has_lower = file_stem.chars().any(|c| c.is_lowercase());

        if has_upper && has_lower {
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
                    "File name uses mixed case."
                },
            )
            .note(markup! {
                "Use a consistent case for the file name (all lowercase or all uppercase)."
            }),
        )
    }
}
