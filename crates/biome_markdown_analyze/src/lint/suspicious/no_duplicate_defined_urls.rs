use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdLinkBlock};
use biome_rowan::{AstNode, BatchMutationExt, TextRange};

use crate::MarkdownRuleAction;

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
        fix_kind: FixKind::Unsafe,
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

        let mut seen_urls = std::collections::HashSet::new();
        let mut signals = Vec::new();

        for link_block in document
            .syntax()
            .descendants()
            .filter_map(MdLinkBlock::cast)
        {
            let url_text = link_block.url().syntax().text_trimmed().to_string();
            let url_text = url_text.trim().to_string();
            if !url_text.is_empty() && !seen_urls.insert(url_text.clone()) {
                signals.push(DuplicateDefinedUrl {
                    range: link_block.syntax().text_trimmed_range(),
                    url: url_text,
                });
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();
        let node = root
            .syntax()
            .descendants()
            .filter_map(MdLinkBlock::cast)
            .find(|n| n.syntax().text_trimmed_range() == state.range)?;
        let mut mutation = root.begin();
        mutation.remove_node(node);
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove the duplicate URL definition." }.to_owned(),
            mutation,
        ))
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
