use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdMdxJsxElement};
use biome_rowan::{AstNode, TextRange};
use biome_rule_options::use_consistent_mdx_jsx_quote_style::UseConsistentMdxJsxQuoteStyleOptions;

use crate::MarkdownRuleAction;
use crate::utils::fix_utils::make_text_replacement;

declare_lint_rule! {
    /// Enforce a consistent quote style for MDX JSX attributes.
    ///
    /// Using a consistent quote style makes the document easier to read
    /// and maintain. By default, the rule enforces consistency (whichever
    /// style appears first is expected for the rest of the file).
    ///
    /// ## Options
    ///
    /// ### `quote`
    ///
    /// The expected quote style: `"double"`, `"single"`, or `"consistent"` (default).
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// <Component name="hello" title='world' />
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// <Component name="hello" title="world" />
    /// ```
    pub UseConsistentMdxJsxQuoteStyle {
        version: "next",
        name: "useConsistentMdxJsxQuoteStyle",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentQuote {
    range: TextRange,
    actual: char,
    expected: char,
    corrected: String,
}

impl Rule for UseConsistentMdxJsxQuoteStyle {
    type Query = Ast<MdDocument>;
    type State = InconsistentQuote;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentMdxJsxQuoteStyleOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let mut signals = Vec::new();

        let quote_opt = ctx.options().quote();
        let mut expected_quote: Option<char> = match quote_opt {
            "double" => Some('"'),
            "single" => Some('\''),
            _ => None, // consistent
        };

        // Walk all MdMdxJsxElement descendants in document order
        for node in document.syntax().descendants() {
            let Some(jsx_elem) = MdMdxJsxElement::cast(node) else {
                continue;
            };
            for attr in jsx_elem.attributes() {
                let Some(val) = attr.value() else {
                    continue;
                };
                let Ok(delimiter) = val.delimiter_token() else {
                    continue;
                };
                let delim_text = delimiter.text_trimmed();
                let Some(q) = delim_text.chars().next() else {
                    continue;
                };
                if q != '"' && q != '\'' {
                    continue; // expression value like {true}, skip
                }

                match expected_quote {
                    None => {
                        expected_quote = Some(q);
                    }
                    Some(exp) => {
                        if q != exp {
                            let name = attr.name().syntax().text_trimmed().to_string();
                            let inner = val.content().syntax().text_trimmed().to_string();
                            let corrected = format!("{}={}{}{}", name, exp, inner, exp);
                            signals.push(InconsistentQuote {
                                range: attr.syntax().text_trimmed_range(),
                                actual: q,
                                expected: exp,
                                corrected,
                            });
                        }
                    }
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
            markup! { "Use consistent quote style." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let actual = if state.actual == '"' {
            "double"
        } else {
            "single"
        };
        let expected = if state.expected == '"' {
            "double"
        } else {
            "single"
        };
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Attribute uses "{ actual }" quotes but "{ expected }" quotes are expected."
                },
            )
            .note(markup! {
                "Use a consistent quote style for JSX attributes."
            }),
        )
    }
}
