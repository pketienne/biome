use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::AstNode;

declare_lint_rule! {
    /// Disallow irregular characters in file names.
    ///
    /// File names should only contain alphanumeric characters, dashes, dots,
    /// and underscores. Other characters can cause issues across operating
    /// systems and tools.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// File name: `my file (1).md`
    ///
    /// ### Valid
    ///
    /// File name: `my-file-1.md`
    pub NoFileNameIrregularCharacters {
        version: "next",
        name: "noFileNameIrregularCharacters",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct IrregularChar {
    character: char,
}

impl Rule for NoFileNameIrregularCharacters {
    type Query = Ast<MdDocument>;
    type State = IrregularChar;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let file_path = ctx.file_path();
        let file_name = file_path.file_name()?;

        for ch in file_name.chars() {
            if !ch.is_ascii_alphanumeric() && ch != '.' && ch != '-' && ch != '_' {
                return Some(IrregularChar { character: ch });
            }
        }

        None
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let ch_display = if state.character == ' ' {
            "space".to_string()
        } else {
            format!("'{}'", state.character)
        };
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                ctx.query().syntax().text_trimmed_range(),
                markup! {
                    "File name contains an irregular character: "{ ch_display }"."
                },
            )
            .note(markup! {
                "Use only alphanumeric characters, dashes, dots, and underscores in file names."
            }),
        )
    }
}
