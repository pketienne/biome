use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdFencedCodeBlock;
use biome_rowan::AstNodeList;

use crate::MarkdownRuleAction;
use crate::utils::fix_utils::make_text_replacement;

declare_lint_rule! {
    /// Require a language tag on fenced code blocks.
    ///
    /// Fenced code blocks without a language specifier make it harder for readers
    /// to understand the code and prevent syntax highlighting in rendered output.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ````md
    /// ```
    /// const x = 1;
    /// ```
    /// ````
    ///
    /// ### Valid
    ///
    /// ````md
    /// ```js
    /// const x = 1;
    /// ```
    /// ````
    pub NoMissingLanguage {
        version: "next",
        name: "noMissingLanguage",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

impl Rule for NoMissingLanguage {
    type Query = Ast<MdFencedCodeBlock>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let code_block = ctx.query();
        if code_block.code_list().is_empty() {
            Some(())
        } else {
            None
        }
    }

    fn action(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<MarkdownRuleAction> {
        let code_block = ctx.query();
        let l_fence = code_block.l_fence_token().ok()?;
        let fence_text = l_fence.text_trimmed();
        let corrected = format!("{}text", fence_text);
        let range = l_fence.text_trimmed_range();
        let mutation = make_text_replacement(&ctx.root(), range, &corrected)?;
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Add \"text\" as the language tag." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        let code_block = ctx.query();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                code_block.l_fence_token().ok()?.text_trimmed_range(),
                markup! {
                    "Fenced code blocks should have a language tag."
                },
            )
            .note(markup! {
                "Add a language identifier after the opening fence to enable syntax highlighting."
            }),
        )
    }
}
