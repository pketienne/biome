use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::definition_utils::collect_definitions;

declare_lint_rule! {
    /// Disallow multiple definitions that resolve to the same URL.
    ///
    /// Having multiple definitions pointing to the same URL is usually
    /// unintentional and creates unnecessary duplication.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [foo]: https://example.com
    /// [bar]: https://example.com
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [foo]: https://example.com
    /// [bar]: https://other.com
    /// ```
    pub NoDuplicateDefinedUrls {
        version: "next",
        name: "noDuplicateDefinedUrls",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct DuplicateDefinedUrl {
    range: TextRange,
    url: String,
}

impl Rule for NoDuplicateDefinedUrls {
    type Query = Ast<MdDocument>;
    type State = DuplicateDefinedUrl;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let definitions = collect_definitions(&text);

        let mut seen_urls = std::collections::HashSet::new();
        let mut signals = Vec::new();

        for def in &definitions {
            if !def.url.is_empty() && !seen_urls.insert(def.url.clone()) {
                signals.push(DuplicateDefinedUrl {
                    range: TextRange::new(
                        base + TextSize::from(def.byte_offset as u32),
                        base + TextSize::from((def.byte_offset + def.byte_len) as u32),
                    ),
                    url: def.url.clone(),
                });
            }
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Multiple definitions point to the same URL: \""{ &state.url }"\"."
                },
            )
            .note(markup! {
                "Consider using a single definition to avoid duplication."
            }),
        )
    }
}
