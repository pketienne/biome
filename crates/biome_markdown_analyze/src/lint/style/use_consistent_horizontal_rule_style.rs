use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdThematicBreakBlock};
use biome_rowan::{AstNode, BatchMutationExt};

use biome_rule_options::use_consistent_horizontal_rule_style::UseConsistentHorizontalRuleStyleOptions;

use crate::MarkdownRuleAction;

declare_lint_rule! {
    /// Enforce a consistent style for horizontal rules.
    ///
    /// Horizontal rules (thematic breaks) can be written in many styles
    /// (`---`, `***`, `___`, etc.). Using a consistent style improves
    /// readability and maintainability.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// With the default style `"---"`:
    ///
    /// ```md
    /// ***
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ---
    /// ```
    ///
    /// ## Options
    ///
    /// ### `style`
    ///
    /// The expected horizontal rule style. Default: `"---"`.
    /// Common styles: `"---"`, `"***"`, `"___"`.
    pub UseConsistentHorizontalRuleStyle {
        version: "next",
        name: "useConsistentHorizontalRuleStyle",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentHorizontalRule {
    range: biome_rowan::TextRange,
    actual: String,
    corrected: String,
}

impl Rule for UseConsistentHorizontalRuleStyle {
    type Query = Ast<MdDocument>;
    type State = InconsistentHorizontalRule;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentHorizontalRuleStyleOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let expected_style = ctx.options().style();
        let mut signals = Vec::new();

        for node in document.syntax().descendants() {
            if let Some(thematic_break) = MdThematicBreakBlock::cast_ref(&node) {
                if let Ok(token) = thematic_break.value_token() {
                    let actual = token.text_trimmed().trim().to_string();
                    if actual != expected_style {
                        signals.push(InconsistentHorizontalRule {
                            range: thematic_break.syntax().text_trimmed_range(),
                            actual,
                            corrected: expected_style.to_string(),
                        });
                    }
                }
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();
        let mut token = root
            .syntax()
            .token_at_offset(state.range.start())
            .right_biased()?;
        let mut tokens = vec![token.clone()];
        while token.text_range().end() < state.range.end() {
            token = token.next_token()?;
            tokens.push(token.clone());
        }
        let first = &tokens[0];
        let last = tokens.last()?;
        let prefix_len = u32::from(state.range.start() - first.text_range().start()) as usize;
        let suffix_start = u32::from(state.range.end() - last.text_range().start()) as usize;
        let prefix = &first.text()[..prefix_len];
        let suffix = &last.text()[suffix_start..];
        let new_text = format!("{}{}{}", prefix, state.corrected, suffix);
        let new_token = biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
            first.kind(),
            &new_text,
            [],
            [],
        );
        let mut mutation = ctx.root().begin();
        mutation.replace_element_discard_trivia(first.clone().into(), new_token.into());
        for t in &tokens[1..] {
            let empty =
                biome_markdown_syntax::MarkdownSyntaxToken::new_detached(t.kind(), "", [], []);
            mutation.replace_element_discard_trivia(t.clone().into(), empty.into());
        }
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Use the consistent horizontal rule style." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let expected = ctx.options().style();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Horizontal rule style '"{&state.actual}"' does not match expected '"{expected}"'."
                },
            )
            .note(markup! {
                "Use '"{expected}"' for horizontal rules."
            }),
        )
    }
}
