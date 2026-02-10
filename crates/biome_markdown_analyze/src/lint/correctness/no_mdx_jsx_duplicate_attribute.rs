use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdMdxJsxElement;
use biome_rowan::{AstNode, TextRange};
use std::collections::HashSet;

use crate::MarkdownRuleAction;
use crate::utils::fix_utils::make_text_replacement;

declare_lint_rule! {
    /// Disallow duplicate attributes on MDX JSX elements.
    ///
    /// Duplicate attributes on JSX elements lead to unpredictable behavior
    /// and are always a mistake.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// <Component name="a" name="b" />
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// <Component name="a" title="b" />
    /// ```
    pub NoMdxJsxDuplicateAttribute {
        version: "next",
        name: "noMdxJsxDuplicateAttribute",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct DuplicateAttribute {
    range: TextRange,
    corrected: String,
    name: String,
}

impl Rule for NoMdxJsxDuplicateAttribute {
    type Query = Ast<MdMdxJsxElement>;
    type State = DuplicateAttribute;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let node = ctx.query();
        let mut signals = Vec::new();
        let mut seen = HashSet::new();

        for attr in node.attributes() {
            let name = attr.name().syntax().text_trimmed().to_string();
            if !seen.insert(name.clone()) {
                // Range includes leading whitespace gap
                let attr_range = attr.syntax().text_trimmed_range();
                let start = attr
                    .syntax()
                    .first_token()
                    .and_then(|t| t.prev_token())
                    .map(|prev| prev.text_trimmed_range().end())
                    .unwrap_or(attr_range.start());
                let range = TextRange::new(start, attr_range.end());
                signals.push(DuplicateAttribute {
                    range,
                    corrected: String::new(),
                    name,
                });
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let mutation = make_text_replacement(&ctx.root(), state.range, &state.corrected)?;
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove the duplicate attribute." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Duplicate attribute \""{ &state.name }"\" on JSX element."
                },
            )
            .note(markup! {
                "Remove the duplicate attribute."
            }),
        )
    }
}
