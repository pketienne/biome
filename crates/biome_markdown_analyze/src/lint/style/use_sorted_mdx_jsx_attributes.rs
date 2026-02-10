use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdMdxJsxElement;
use biome_rowan::{AstNode, TextRange};

use crate::MarkdownRuleAction;
use crate::utils::fix_utils::make_text_replacement;

declare_lint_rule! {
    /// Enforce sorted attributes on MDX JSX elements.
    ///
    /// Keeping attributes in alphabetical order improves readability and
    /// makes diffs cleaner.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// <Component zebra="1" alpha="2" />
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// <Component alpha="2" zebra="1" />
    /// ```
    pub UseSortedMdxJsxAttributes {
        version: "next",
        name: "useSortedMdxJsxAttributes",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct UnsortedAttribute {
    range: TextRange,
    first_unsorted: String,
    previous: String,
    /// Range covering all attributes for the fix.
    attrs_range: TextRange,
    /// The corrected (sorted) attributes text.
    corrected: String,
}

impl Rule for UseSortedMdxJsxAttributes {
    type Query = Ast<MdMdxJsxElement>;
    type State = UnsortedAttribute;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let node = ctx.query();
        let attrs: Vec<_> = node.attributes().into_iter().collect();
        if attrs.len() < 2 {
            return Vec::new();
        }

        // Collect attribute names and their raw text representations
        let attr_info: Vec<(String, String, TextRange)> = attrs
            .iter()
            .map(|attr| {
                let name = attr.name().syntax().text_trimmed().to_string();
                let raw = attr.syntax().text_trimmed().to_string();
                let range = attr.syntax().text_trimmed_range();
                (name, raw, range)
            })
            .collect();

        // Find the first out-of-order attribute
        for i in 1..attr_info.len() {
            let (prev_name, _, _) = &attr_info[i - 1];
            let (curr_name, _, curr_range) = &attr_info[i];
            if curr_name.to_lowercase() < prev_name.to_lowercase() {
                // Build sorted text
                let mut sorted: Vec<(String, String)> = attr_info
                    .iter()
                    .map(|(name, raw, _)| (name.to_lowercase(), raw.clone()))
                    .collect();
                sorted.sort_by(|a, b| a.0.cmp(&b.0));
                let corrected = sorted
                    .iter()
                    .map(|(_, raw)| raw.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");

                let first_range = attr_info[0].2;
                let last_range = attr_info.last().unwrap().2;
                let attrs_range = TextRange::new(first_range.start(), last_range.end());

                return vec![UnsortedAttribute {
                    range: *curr_range,
                    first_unsorted: curr_name.clone(),
                    previous: prev_name.clone(),
                    attrs_range,
                    corrected,
                }];
            }
        }

        Vec::new()
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let mutation = make_text_replacement(&ctx.root(), state.attrs_range, &state.corrected)?;
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Sort JSX attributes alphabetically." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Attribute \""{ &state.first_unsorted }"\" should come before \""{ &state.previous }"\"."
                },
            )
            .note(markup! {
                "Sort JSX attributes alphabetically."
            }),
        )
    }
}
