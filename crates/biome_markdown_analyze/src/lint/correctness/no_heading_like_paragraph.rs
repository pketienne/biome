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
    /// Disallow heading-like lines that exceed the valid h1-h6 range.
    ///
    /// Markdown only supports headings from level 1 (`#`) to level 6 (`######`).
    /// Lines starting with 7 or more `#` characters are not valid headings
    /// and are rendered as plain paragraphs, which is likely a mistake.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// ####### This is not a valid heading
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ###### This is a valid h6 heading
    /// ```
    pub NoHeadingLikeParagraph {
        version: "next",
        name: "noHeadingLikeParagraph",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InvalidHeading {
    range: TextRange,
    corrected: String,
    hash_count: usize,
}

impl Rule for NoHeadingLikeParagraph {
    type Query = Ast<MdDocument>;
    type State = InvalidHeading;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let lines: Vec<&str> = text.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            tracker.process_line(line_idx, line);
            if tracker.is_inside_fence() {
                continue;
            }

            let trimmed = line.trim_start();

            // Count leading # characters
            let hash_count = trimmed.bytes().take_while(|&b| b == b'#').count();

            // Only flag lines with 7+ hashes that look like heading attempts
            // (followed by space or end of line)
            if hash_count >= 7 {
                let after_hashes = &trimmed[hash_count..];
                if after_hashes.is_empty() || after_hashes.starts_with(' ') {
                    let line_offset: usize =
                        lines[..line_idx].iter().map(|l| l.len() + 1).sum();
                    // Build corrected line: leading whitespace + 6 hashes + rest
                    let leading_ws_len = line.len() - trimmed.len();
                    let leading_ws = &line[..leading_ws_len];
                    let corrected =
                        format!("{}######{}", leading_ws, &trimmed[hash_count..]);
                    signals.push(InvalidHeading {
                        range: TextRange::new(
                            base + TextSize::from(line_offset as u32),
                            base + TextSize::from((line_offset + line.len()) as u32),
                        ),
                        corrected,
                        hash_count,
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
            markup! { "Convert to a valid heading level." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Line starts with "{state.hash_count}" '#' characters, but headings only support levels 1-6."
                },
            )
            .note(markup! {
                "Use a heading level between 1 and 6, or remove the leading '#' characters."
            }),
        )
    }
}
