use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::AstNode;

declare_lint_rule! {
    /// Disallow file names starting with an article.
    ///
    /// File names that start with articles like "a", "an", or "the" add noise
    /// without helping with discoverability or sorting.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// File name: `the-readme.md`
    ///
    /// ### Valid
    ///
    /// File name: `readme.md`
    pub NoFileNameArticles {
        version: "next",
        name: "noFileNameArticles",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct FileNameArticle {
    article: String,
}

impl Rule for NoFileNameArticles {
    type Query = Ast<MdDocument>;
    type State = FileNameArticle;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let file_path = ctx.file_path();
        let file_stem = file_path.file_stem()?;
        let lower = file_stem.to_lowercase();

        for article in &["a-", "an-", "the-"] {
            if lower.starts_with(article) {
                return Some(FileNameArticle {
                    article: article.trim_end_matches('-').to_string(),
                });
            }
        }

        None
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                ctx.query().syntax().text_trimmed_range(),
                markup! {
                    "File name starts with the article \""{ &state.article }"\"."
                },
            )
            .note(markup! {
                "Remove the leading article from the file name."
            }),
        )
    }
}
