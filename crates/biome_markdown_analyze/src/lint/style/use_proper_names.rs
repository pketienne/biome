use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use crate::MarkdownRuleAction;
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use biome_rule_options::use_proper_names::UseProperNamesOptions;

use crate::utils::fence_utils::FenceTracker;

declare_lint_rule! {
    /// Enforce correct capitalization of proper names.
    ///
    /// Proper names such as product names, programming languages, and
    /// company names should use their official capitalization. For example,
    /// "JavaScript" instead of "javascript" or "Javascript".
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `names: ["JavaScript"]`:
    ///
    /// ```md
    /// Use javascript for web development.
    /// ```
    ///
    /// ### Valid
    ///
    /// When configured with `names: ["JavaScript"]`:
    ///
    /// ```md
    /// Use JavaScript for web development.
    /// ```
    ///
    /// ## Options
    ///
    /// ### `names`
    ///
    /// A list of proper names with their correct capitalization.
    /// Default: `[]` (no names checked).
    pub UseProperNames {
        version: "next",
        name: "useProperNames",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct ImproperName {
    range: TextRange,
    found: String,
    expected: String,
}

impl Rule for UseProperNames {
    type Query = Ast<MdDocument>;
    type State = ImproperName;
    type Signals = Vec<Self::State>;
    type Options = UseProperNamesOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let names = ctx.options().names();
        if names.is_empty() {
            return Vec::new();
        }

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

            let line_offset: usize = lines[..line_idx].iter().map(|l| l.len() + 1).sum();

            for name in names {
                let name_lower = name.to_lowercase();
                let line_lower = line.to_lowercase();

                let mut search_start = 0;
                while let Some(pos) = line_lower[search_start..].find(&name_lower) {
                    let abs_pos = search_start + pos;
                    let found = &line[abs_pos..abs_pos + name.len()];

                    // Only flag if the casing doesn't match
                    if found != name.as_str() {
                        // Check word boundaries
                        let before_ok = abs_pos == 0
                            || !line.as_bytes()[abs_pos - 1].is_ascii_alphanumeric();
                        let after_pos = abs_pos + name.len();
                        let after_ok = after_pos >= line.len()
                            || !line.as_bytes()[after_pos].is_ascii_alphanumeric();

                        if before_ok && after_ok {
                            // Skip matches inside code spans
                            if !is_inside_code_span(line, abs_pos) {
                                signals.push(ImproperName {
                                    range: TextRange::new(
                                        base + TextSize::from((line_offset + abs_pos) as u32),
                                        base + TextSize::from(
                                            (line_offset + abs_pos + name.len()) as u32,
                                        ),
                                    ),
                                    found: found.to_string(),
                                    expected: name.clone(),
                                });
                            }
                        }
                    }

                    search_start = abs_pos + name.len();
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
        let new_text = format!("{}{}{}", prefix, state.expected, suffix);
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
            markup! { "Use the correct capitalization." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Found \""{ &state.found }"\" instead of \""{ &state.expected }"\"."
                },
            )
            .note(markup! {
                "Use the correct capitalization for proper names."
            }),
        )
    }
}

/// Check if a position in a line is inside a backtick code span.
fn is_inside_code_span(line: &str, pos: usize) -> bool {
    let bytes = line.as_bytes();
    let mut in_code = false;
    let mut i = 0;

    while i < bytes.len() && i < pos {
        if bytes[i] == b'`' {
            in_code = !in_code;
        }
        i += 1;
    }

    in_code
}
