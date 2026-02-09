use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;
use crate::utils::definition_utils::collect_definitions;

declare_lint_rule! {
    /// Disallow duplicate link reference definitions.
    ///
    /// Each label should only be defined once. Duplicate definitions are
    /// confusing and the second definition is ignored by parsers.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [foo]: https://example.com
    /// [foo]: https://other.com
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [foo]: https://example.com
    /// [bar]: https://other.com
    /// ```
    pub NoDuplicateDefinitions {
        version: "next",
        name: "noDuplicateDefinitions",
        language: "md",
        recommended: true,
        severity: Severity::Error,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct DuplicateDefinition {
    range: TextRange,
    label: String,
}

impl Rule for NoDuplicateDefinitions {
    type Query = Ast<MdDocument>;
    type State = DuplicateDefinition;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let definitions = collect_definitions(&text);

        let mut seen = std::collections::HashSet::new();
        let mut signals = Vec::new();

        for def in &definitions {
            if !seen.insert(def.label.clone()) {
                signals.push(DuplicateDefinition {
                    range: TextRange::new(
                        base + TextSize::from(def.byte_offset as u32),
                        base + TextSize::from((def.byte_offset + def.byte_len) as u32),
                    ),
                    label: def.label.clone(),
                });
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();
        // Remove the duplicate definition line (replace with empty string)
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
        // Remove the definition line - just keep prefix and suffix
        let new_text = format!("{}{}", prefix, suffix);
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
            markup! { "Remove the duplicate definition." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Duplicate definition for label \""{ &state.label }"\"."
                },
            )
            .note(markup! {
                "Remove the duplicate definition. Only the first definition is used."
            }),
        )
    }
}
