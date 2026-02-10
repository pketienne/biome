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
    /// Enforce shorthand boolean attributes in MDX JSX elements.
    ///
    /// In JSX, `prop={true}` can be written as just `prop`. The shorthand
    /// form is more idiomatic and concise.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// <Component disabled={true} />
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// <Component disabled />
    /// ```
    pub UseMdxJsxShorthandAttribute {
        version: "next",
        name: "useMdxJsxShorthandAttribute",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct LonghandBoolean {
    range: TextRange,
    name: String,
    corrected: String,
}

impl Rule for UseMdxJsxShorthandAttribute {
    type Query = Ast<MdMdxJsxElement>;
    type State = LonghandBoolean;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let node = ctx.query();
        let mut signals = Vec::new();

        for attr in node.attributes() {
            if let Some(val) = attr.value() {
                // Check if value is {true} — delimiter is "{", content is "true", closing is "}"
                let delimiter = val
                    .delimiter_token()
                    .ok()
                    .map(|t| t.text_trimmed().to_string())
                    .unwrap_or_default();
                let content = val.content().syntax().text_trimmed().to_string();
                let closing = val
                    .closing_delimiter_token()
                    .ok()
                    .map(|t| t.text_trimmed().to_string())
                    .unwrap_or_default();

                if delimiter == "{" && content == "true" && closing == "}" {
                    let name = attr.name().syntax().text_trimmed().to_string();
                    signals.push(LonghandBoolean {
                        range: attr.syntax().text_trimmed_range(),
                        name: name.clone(),
                        corrected: name,
                    });
                }
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let mutation = make_text_replacement(&ctx.root(), state.range, &state.corrected)?;
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Use shorthand boolean attribute." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Attribute \""{ &state.name }"\" uses longhand \"{true}\" instead of shorthand."
                },
            )
            .note(markup! {
                "Use the shorthand form: just \""{ &state.name }"\" without a value."
            }),
        )
    }
}
