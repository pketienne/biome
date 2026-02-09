use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::AstNode;
use biome_rule_options::use_file_extension::UseFileExtensionOptions;

declare_lint_rule! {
    /// Enforce a specific file extension for markdown files.
    ///
    /// By default, this rule requires markdown files to use the `.md` extension.
    /// The expected extension can be configured.
    ///
    /// ## Options
    ///
    /// ### `extension`
    ///
    /// The expected file extension (default: `"md"`).
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// File name: `readme.markdown` (when expecting `.md`)
    ///
    /// ### Valid
    ///
    /// File name: `readme.md`
    pub UseFileExtension {
        version: "next",
        name: "useFileExtension",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct WrongExtension {
    actual: String,
    expected: String,
}

impl Rule for UseFileExtension {
    type Query = Ast<MdDocument>;
    type State = WrongExtension;
    type Signals = Option<Self::State>;
    type Options = UseFileExtensionOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let file_path = ctx.file_path();
        let expected = ctx.options().extension();
        let actual = file_path.extension()?;

        if actual != expected {
            Some(WrongExtension {
                actual: actual.to_string(),
                expected: expected.to_string(),
            })
        } else {
            None
        }
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                ctx.query().syntax().text_trimmed_range(),
                markup! {
                    "File has extension \"."{&state.actual}"\" but \"."{&state.expected}"\" is expected."
                },
            )
            .note(markup! {
                "Rename the file to use the \"."{&state.expected}"\" extension."
            }),
        )
    }
}
