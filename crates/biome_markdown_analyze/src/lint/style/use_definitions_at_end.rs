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
    /// Enforce that link reference definitions are placed at the end.
    ///
    /// Link reference definitions should be grouped at the end of the
    /// document for readability.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// Some text.
    ///
    /// More text.
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// Some text.
    ///
    /// More text.
    /// ```
    pub UseDefinitionsAtEnd {
        version: "next",
        name: "useDefinitionsAtEnd",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct DefinitionNotAtEnd {
    range: TextRange,
    label: String,
    corrected: String,
}

impl Rule for UseDefinitionsAtEnd {
    type Query = Ast<MdDocument>;
    type State = DefinitionNotAtEnd;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let definitions = collect_definitions(&text);
        let lines: Vec<&str> = text.lines().collect();
        let mut signals = Vec::new();

        if definitions.is_empty() {
            return signals;
        }

        // Find the last definition's line index
        let last_def_line = definitions.iter().map(|d| d.line_index).max().unwrap_or(0);

        // Check if there is non-blank, non-definition content after the last definition
        // If so, definitions that appear before that content are "not at end"
        let has_content_after = (last_def_line + 1..lines.len()).any(|l| {
            let line = lines[l];
            !line.trim().is_empty()
        });

        if !has_content_after {
            // All definitions are at the end, nothing to flag
            return signals;
        }

        // If there's content after definitions, flag all definitions that have
        // non-definition content after them
        for def in &definitions {
            // Check if there's non-definition, non-blank content after this definition
            let has_later_content = (def.line_index + 1..lines.len()).any(|l| {
                let line = lines[l];
                if line.trim().is_empty() {
                    return false;
                }
                // Check if it's also a definition
                !definitions.iter().any(|d| d.line_index == l)
            });

            if has_later_content {
                signals.push(DefinitionNotAtEnd {
                    range: TextRange::new(
                        base + TextSize::from(def.byte_offset as u32),
                        base + TextSize::from((def.byte_offset + def.byte_len) as u32),
                    ),
                    label: def.label.clone(),
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
        let label = &state.label;
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Definition \""{ label }"\" is not at the end of the document."
                },
            )
            .note(markup! {
                "Move link reference definitions to the end of the document."
            }),
        )
    }
}
