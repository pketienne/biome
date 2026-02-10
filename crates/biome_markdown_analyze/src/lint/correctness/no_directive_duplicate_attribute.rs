use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDirective;
use biome_rowan::{AstNode, AstNodeList, TextRange};
use std::collections::HashSet;

use crate::MarkdownRuleAction;
use crate::utils::fix_utils::make_text_replacement;

declare_lint_rule! {
    /// Disallow duplicate attributes on markdown directives.
    ///
    /// Duplicate attributes on directives lead to unpredictable behavior
    /// and are always a mistake.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// ::video{src="a.mp4" src="b.mp4"}
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ::video{src="a.mp4" title="My Video"}
    /// ```
    pub NoDirectiveDuplicateAttribute {
        version: "next",
        name: "noDirectiveDuplicateAttribute",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct DuplicateDirAttribute {
    /// Range of the duplicate attribute (including leading whitespace for the fix).
    range: TextRange,
    /// Empty string — the fix removes the duplicate.
    corrected: String,
    /// Effective name of the duplicate attribute.
    name: String,
}

/// Get the effective attribute name from an `MdDirectiveAttribute`.
///
/// - `.foo` (class shorthand) → `"class"`
/// - `#foo` (id shorthand) → `"id"`
/// - `name` (regular) → `"name"`
fn effective_attr_name(attr: &biome_markdown_syntax::MdDirectiveAttribute) -> String {
    let raw = attr.name().syntax().text_trimmed().to_string();
    if raw.starts_with('.') {
        "class".to_string()
    } else if raw.starts_with('#') {
        "id".to_string()
    } else {
        raw
    }
}

impl Rule for NoDirectiveDuplicateAttribute {
    type Query = Ast<MdDirective>;
    type State = DuplicateDirAttribute;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let directive = ctx.query();
        let attrs = directive.attributes();
        let mut signals = Vec::new();
        let mut seen = HashSet::new();

        for attr in attrs.iter() {
            let name = effective_attr_name(&attr);
            if !seen.insert(name.clone()) {
                // Find range including leading whitespace for the fix.
                // The space between attributes is trailing trivia on the previous token,
                // so we extend the start back to the previous token's trimmed end.
                let attr_range = attr.syntax().text_trimmed_range();
                let start = attr
                    .syntax()
                    .first_token()
                    .and_then(|t| t.prev_token())
                    .map(|prev| prev.text_trimmed_range().end())
                    .unwrap_or(attr_range.start());
                let range = TextRange::new(start, attr_range.end());

                signals.push(DuplicateDirAttribute {
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
                    "Duplicate attribute \""{ &state.name }"\" on directive."
                },
            )
            .note(markup! {
                "Remove the duplicate attribute."
            }),
        )
    }
}
