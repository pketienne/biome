use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;
use crate::utils::fence_utils::FenceTracker;

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

pub struct MissingLanguage {
    range: TextRange,
    corrected: String,
}

impl Rule for NoMissingLanguage {
    type Query = Ast<MdDocument>;
    type State = MissingLanguage;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_trimmed().to_string();
        let start = document.syntax().text_trimmed_range().start();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();

        for (line_idx, line) in text.lines().enumerate() {
            if let Some(fence_open) = tracker.process_line(line_idx, line) {
                if fence_open.info_string.is_empty() {
                    let line_offset: usize =
                        text.lines().take(line_idx).map(|l: &str| l.len() + 1).sum();
                    let offset = TextSize::from(line_offset as u32);
                    let len = TextSize::from(line.len() as u32);
                    // Build the corrected fence line by appending "text" to the fence marker
                    let corrected = format!("{}text", line);
                    signals.push(MissingLanguage {
                        range: TextRange::new(start + offset, start + offset + len),
                        corrected,
                    });
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
            markup! { "Add \"text\" as the language tag." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
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
