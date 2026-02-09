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
    /// Disallow dollar signs in shell code fence content.
    ///
    /// Shell commands in code blocks should not include the `$` prompt
    /// prefix, as it makes the code harder to copy and paste.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// In a shell code block, don't prefix commands with `$`:
    ///
    /// ````md
    /// ```sh
    /// $ npm install
    /// ```
    /// ````
    ///
    /// ### Valid
    ///
    /// ````md
    /// ```sh
    /// npm install
    /// ```
    /// ````
    pub NoShellDollarPrompt {
        version: "next",
        name: "noShellDollarPrompt",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

const SHELL_LANGUAGES: &[&str] = &[
    "sh", "shell", "bash", "zsh", "fish", "console", "terminal",
];

pub struct DollarPrompt {
    range: TextRange,
}

impl Rule for NoShellDollarPrompt {
    type Query = Ast<MdDocument>;
    type State = DollarPrompt;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut in_shell_fence = false;
        let mut offset = 0usize;

        for (line_idx, line) in text.lines().enumerate() {
            let prev_inside = tracker.is_inside_fence();
            let fence_result = tracker.process_line(line_idx, line);

            let is_fence_open = if let Some(ref fence_open) = fence_result {
                // Check if the language is a shell language
                let lang = fence_open.info_string.to_lowercase();
                in_shell_fence = SHELL_LANGUAGES.iter().any(|&s| lang == s);
                true
            } else {
                if prev_inside && !tracker.is_inside_fence() {
                    // Closing fence
                    in_shell_fence = false;
                }
                false
            };

            // Check lines inside shell fences for dollar prompts
            if tracker.is_inside_fence() && in_shell_fence && !is_fence_open {
                let trimmed = line.trim_start();
                if trimmed.starts_with("$ ") {
                    let leading = line.len() - trimmed.len();
                    let dollar_offset = offset + leading;
                    signals.push(DollarPrompt {
                        range: TextRange::new(
                            base + TextSize::from(dollar_offset as u32),
                            base + TextSize::from((dollar_offset + 2) as u32),
                        ),
                    });
                }
            }

            offset += line.len() + 1;
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();
        let token = root
            .syntax()
            .token_at_offset(state.range.start())
            .right_biased()?;
        let token_text = token.text().to_string();
        let token_start = token.text_range().start();
        let rel_start = u32::from(state.range.start() - token_start) as usize;
        let rel_end = u32::from(state.range.end() - token_start) as usize;
        // Remove the "$ " prefix
        let mut new_text = String::with_capacity(token_text.len());
        new_text.push_str(&token_text[..rel_start]);
        new_text.push_str(&token_text[rel_end..]);
        let new_token = biome_markdown_syntax::MarkdownSyntaxToken::new_detached(
            token.kind(),
            &new_text,
            [],
            [],
        );
        let mut mutation = ctx.root().begin();
        mutation.replace_element_discard_trivia(token.into(), new_token.into());
        Some(RuleAction::new(
            ctx.metadata().action_category(ctx.category(), ctx.group()),
            ctx.metadata().applicability(),
            markup! { "Remove dollar sign prompt." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Don't use dollar sign prompts in shell code blocks."
                },
            )
            .note(markup! {
                "Remove the dollar sign prefix to make the code easier to copy."
            }),
        )
    }
}
