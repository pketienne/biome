use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use crate::MarkdownRuleAction;
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::utils::definition_utils::collect_definitions;

declare_lint_rule! {
    /// Enforce lowercase labels in link reference definitions.
    ///
    /// Definition labels should be lowercase for consistency.
    /// While label matching is case-insensitive in markdown,
    /// using lowercase labels avoids confusion.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [FOO]: https://example.com
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [foo]: https://example.com
    /// ```
    pub UseLowercaseDefinitionLabels {
        version: "next",
        name: "useLowercaseDefinitionLabels",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct UppercaseLabel {
    range: TextRange,
    label: String,
    corrected: String,
}

impl Rule for UseLowercaseDefinitionLabels {
    type Query = Ast<MdDocument>;
    type State = UppercaseLabel;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let definitions = collect_definitions(&text);

        let mut signals = Vec::new();

        for def in &definitions {
            if def.raw_label != def.raw_label.to_lowercase() {
                // Compute corrected line by replacing label with lowercase
                let line_text = &text[def.byte_offset..def.byte_offset + def.byte_len];
                let corrected = if let Some(start) = line_text.find('[') {
                    if let Some(end) = line_text.find("]:") {
                        let label_part = &line_text[start + 1..end];
                        format!(
                            "{}[{}]{}",
                            &line_text[..start],
                            label_part.to_lowercase(),
                            &line_text[end + 1..]
                        )
                    } else {
                        line_text.to_string()
                    }
                } else {
                    line_text.to_string()
                };
                signals.push(UppercaseLabel {
                    range: TextRange::new(
                        base + TextSize::from(def.byte_offset as u32),
                        base + TextSize::from((def.byte_offset + def.byte_len) as u32),
                    ),
                    label: def.raw_label.clone(),
                    corrected,
                });
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
            let empty = biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
                t.kind(),
                "",
                [],
                [],
            );
            mutation.replace_element_discard_trivia(t.clone().into(), empty.into());
        }
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Lowercase the definition label." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Definition label \""{ &state.label }"\" should be lowercase."
                },
            )
            .note(markup! {
                "Use lowercase labels for consistency."
            }),
        )
    }
}
