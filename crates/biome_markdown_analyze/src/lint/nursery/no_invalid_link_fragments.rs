use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::fence_utils::FenceTracker;
use crate::utils::heading_utils::collect_heading_slugs;
use crate::utils::inline_utils::{find_code_spans, find_inline_links};

declare_lint_rule! {
    /// Disallow links with invalid fragment identifiers.
    ///
    /// Fragment links (e.g. `#heading`) should point to a heading that
    /// exists in the document.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// # Hello
    ///
    /// See [details](#nonexistent).
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// # Hello
    ///
    /// See [details](#hello).
    /// ```
    pub NoInvalidLinkFragments {
        version: "next",
        name: "noInvalidLinkFragments",
        language: "md",
        recommended: true,
        severity: Severity::Error,
    }
}

pub struct InvalidFragment {
    range: TextRange,
    fragment: String,
}

impl Rule for NoInvalidLinkFragments {
    type Query = Ast<MdDocument>;
    type State = InvalidFragment;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let heading_slugs = collect_heading_slugs(&text);

        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);

            if !tracker.is_inside_fence() {
                let code_spans = find_code_spans(line);
                let links = find_inline_links(line, &code_spans);

                for link in links {
                    if link.url.starts_with('#') {
                        let fragment = &link.url[1..];
                        if !fragment.is_empty() && !heading_slugs.iter().any(|s| s == fragment) {
                            signals.push(InvalidFragment {
                                range: TextRange::new(
                                    base + TextSize::from((offset + link.start) as u32),
                                    base + TextSize::from((offset + link.end) as u32),
                                ),
                                fragment: fragment.to_string(),
                            });
                        }
                    }
                }
            }

            offset += line.len() + 1;
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Link fragment \"#"{ &state.fragment }"\" does not match any heading in the document."
                },
            )
            .note(markup! {
                "Check that the fragment matches a heading slug in this document."
            }),
        )
    }
}
