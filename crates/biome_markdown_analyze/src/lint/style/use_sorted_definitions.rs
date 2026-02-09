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
    /// Enforce alphabetically sorted link reference definitions.
    ///
    /// Definitions should be sorted by their label for easier navigation
    /// and to avoid duplicates.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// [beta]: https://beta.com
    /// [alpha]: https://alpha.com
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [alpha]: https://alpha.com
    /// [beta]: https://beta.com
    /// ```
    pub UseSortedDefinitions {
        version: "next",
        name: "useSortedDefinitions",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct UnsortedDefinition {
    range: TextRange,
    label: String,
    expected_after: String,
    corrected: String,
}

impl Rule for UseSortedDefinitions {
    type Query = Ast<MdDocument>;
    type State = UnsortedDefinition;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let definitions = collect_definitions(&text);

        let mut signals = Vec::new();

        for window in definitions.windows(2) {
            let prev = &window[0];
            let curr = &window[1];

            if curr.label < prev.label {
                signals.push(UnsortedDefinition {
                    range: TextRange::new(
                        base + TextSize::from(curr.byte_offset as u32),
                        base + TextSize::from((curr.byte_offset + curr.byte_len) as u32),
                    ),
                    label: curr.label.clone(),
                    expected_after: prev.label.clone(),
                    corrected: String::new(),
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
            let empty =
                biome_markdown_syntax::MarkdownSyntaxToken::new_detached(t.kind(), "", [], []);
            mutation.replace_element_discard_trivia(t.clone().into(), empty.into());
        }
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove definition from current position." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Definition \""{ &state.label }"\" should come before \""{ &state.expected_after }"\"."
                },
            )
            .note(markup! {
                "Sort definitions alphabetically by their label."
            }),
        )
    }
}
