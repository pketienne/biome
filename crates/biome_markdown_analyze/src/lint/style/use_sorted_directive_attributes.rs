use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDirective;
use biome_rowan::{AstNode, AstNodeList, TextRange};

use crate::MarkdownRuleAction;
use crate::utils::fix_utils::make_text_replacement;

declare_lint_rule! {
    /// Enforce sorted attributes on markdown directives.
    ///
    /// Keeping directive attributes in alphabetical order improves readability
    /// and makes diffs cleaner.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// ::video{zebra="1" alpha="2"}
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ::video{alpha="2" zebra="1"}
    /// ```
    pub UseSortedDirectiveAttributes {
        version: "next",
        name: "useSortedDirectiveAttributes",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct UnsortedDirAttribute {
    range: TextRange,
    first_unsorted: String,
    previous: String,
    attrs_range: TextRange,
    corrected: String,
}

/// Get the effective attribute name for sorting purposes.
fn sort_key(attr: &biome_markdown_syntax::MdDirectiveAttribute) -> String {
    let raw = attr.name().syntax().text_trimmed().to_string();
    if raw.starts_with('.') {
        "class".to_string()
    } else if raw.starts_with('#') {
        "id".to_string()
    } else {
        raw
    }
}

impl Rule for UseSortedDirectiveAttributes {
    type Query = Ast<MdDirective>;
    type State = UnsortedDirAttribute;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let directive = ctx.query();
        let attrs: Vec<_> = directive.attributes().iter().collect();
        let mut signals = Vec::new();

        if attrs.len() < 2 {
            return signals;
        }

        // Check if any attribute is out of order
        let mut is_unsorted = false;
        let mut first_unsorted = String::new();
        let mut previous = String::new();
        let mut unsorted_range = TextRange::empty(TextRange::new(0.into(), 0.into()).start());

        for i in 1..attrs.len() {
            let prev_name = sort_key(&attrs[i - 1]);
            let curr_name = sort_key(&attrs[i]);
            if curr_name.to_lowercase() < prev_name.to_lowercase() {
                is_unsorted = true;
                first_unsorted = curr_name;
                previous = prev_name;
                unsorted_range = attrs[i].syntax().text_trimmed_range();
                break;
            }
        }

        if is_unsorted {
            let first_attr = &attrs[0];
            let last_attr = attrs.last().unwrap();
            let attrs_range = TextRange::new(
                first_attr.syntax().text_trimmed_range().start(),
                last_attr.syntax().text_trimmed_range().end(),
            );

            // Build sorted attribute text
            let mut attr_texts: Vec<(String, String)> = attrs
                .iter()
                .map(|a| {
                    let key = sort_key(a).to_lowercase();
                    let raw = a.syntax().text_trimmed().to_string();
                    (key, raw)
                })
                .collect();
            attr_texts.sort_by(|a, b| a.0.cmp(&b.0));
            let corrected = attr_texts
                .iter()
                .map(|(_, raw)| raw.as_str())
                .collect::<Vec<_>>()
                .join(" ");

            signals.push(UnsortedDirAttribute {
                range: unsorted_range,
                first_unsorted,
                previous,
                attrs_range,
                corrected,
            });
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let mutation = make_text_replacement(&ctx.root(), state.attrs_range, &state.corrected)?;
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Sort directive attributes alphabetically." }.to_owned(),
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
                "Sort directive attributes alphabetically."
            }),
        )
    }
}
