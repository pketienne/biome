use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdHeader};
use biome_rowan::{AstNode, AstNodeList, BatchMutationExt, TextRange};

use crate::MarkdownRuleAction;

declare_lint_rule! {
    /// Disallow multiple top-level headings in a document.
    ///
    /// A document should have a single top-level heading (`# heading`) that
    /// serves as the document title. Multiple top-level headings indicate
    /// a structural problem.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// # First Title
    /// # Second Title
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// # Title
    /// ## Section One
    /// ## Section Two
    /// ```
    pub NoMultipleTopLevelHeadings {
        version: "next",
        name: "noMultipleTopLevelHeadings",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct DuplicateTopLevel {
    range: TextRange,
    corrected: String,
}

impl Rule for NoMultipleTopLevelHeadings {
    type Query = Ast<MdDocument>;
    type State = DuplicateTopLevel;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let mut signals = Vec::new();
        let mut found_top_level = false;

        for node in document.syntax().descendants() {
            if let Some(header) = MdHeader::cast_ref(&node) {
                let level = header.before().len();
                if level == 1 {
                    if found_top_level {
                        let text = header.syntax().text_trimmed().to_string();
                        // Demote "# Heading" to "## Heading" by adding an extra "#"
                        let corrected = format!("#{}", text);
                        signals.push(DuplicateTopLevel {
                            range: header.syntax().text_trimmed_range(),
                            corrected,
                        });
                    }
                    found_top_level = true;
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
            markup! { "Demote to h2 heading." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Multiple top-level headings found."
                },
            )
            .note(markup! {
                "A document should contain only one top-level heading."
            }),
        )
    }
}
