use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;

use crate::utils::line_utils::leading_indent;

declare_lint_rule! {
    /// Disallow indentation before headings.
    ///
    /// Headings should not be indented. Leading spaces before the `#` characters
    /// can be confusing and may be interpreted differently by parsers.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    ///   # Indented heading
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// # Non-indented heading
    /// ```
    pub NoHeadingIndent {
        version: "next",
        name: "noHeadingIndent",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct IndentedHeading {
    range: TextRange,
}

impl Rule for NoHeadingIndent {
    type Query = Ast<MdDocument>;
    type State = IndentedHeading;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let mut signals = Vec::new();
        let mut offset = 0usize;

        for line in text.lines() {
            let indent = leading_indent(line);
            if indent > 0 {
                let trimmed = line.trim_start();
                if trimmed.starts_with('#') {
                    let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
                    if hash_count >= 1
                        && hash_count <= 6
                        && (trimmed.len() == hash_count
                            || trimmed.as_bytes().get(hash_count) == Some(&b' '))
                    {
                        signals.push(IndentedHeading {
                            range: TextRange::new(
                                base + TextSize::from(offset as u32),
                                base + TextSize::from((offset + indent) as u32),
                            ),
                        });
                    }
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
        // The range points to leading whitespace which is in the token's leading trivia.
        // Use the full token text (including trivia) to compute the fix.
        let token_text = token.text().to_string();
        let token_start = token.text_range().start();
        let rel_start = u32::from(state.range.start() - token_start) as usize;
        let rel_end = u32::from(state.range.end() - token_start) as usize;
        // Build replacement text with the whitespace removed
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
            markup! { "Remove leading whitespace." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Headings should not be indented."
                },
            )
            .note(markup! {
                "Remove the leading whitespace before the heading."
            }),
        )
    }
}
