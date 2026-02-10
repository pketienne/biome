use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use biome_rule_options::use_consistent_link_title_style::UseConsistentLinkTitleStyleOptions;

use crate::MarkdownRuleAction;
use crate::utils::definition_utils::collect_definitions_from_ast;
use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{find_code_spans, find_inline_links};

declare_lint_rule! {
    /// Enforce consistent link title delimiter style.
    ///
    /// Link titles can be wrapped in double quotes (`"`), single quotes (`'`),
    /// or parentheses (`(`). This rule enforces a consistent delimiter.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"double-quote"` (default):
    ///
    /// ```md
    /// [text](url 'title')
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [text](url "title")
    /// ```
    ///
    /// ## Options
    ///
    /// ### `style`
    ///
    /// Which delimiter to enforce. Default: `"double-quote"`.
    /// Allowed values: `"double-quote"`, `"single-quote"`, `"parentheses"`.
    pub UseConsistentLinkTitleStyle {
        version: "next",
        name: "useConsistentLinkTitleStyle",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentTitleStyle {
    range: TextRange,
    expected: &'static str,
    actual: char,
    corrected: String,
}

impl Rule for UseConsistentLinkTitleStyle {
    type Query = Ast<MdDocument>;
    type State = InconsistentTitleStyle;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentLinkTitleStyleOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let expected_char = match ctx.options().style() {
            "single-quote" => '\'',
            "parentheses" => '(',
            _ => '"',
        };
        let expected_name = match expected_char {
            '\'' => "single-quote",
            '(' => "parentheses",
            _ => "double-quote",
        };

        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;

        // Check inline links
        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);

            if !tracker.is_inside_fence() {
                let code_spans = find_code_spans(line);
                let links = find_inline_links(line, &code_spans);

                for link in links {
                    if let Some(delim) = link.title_delimiter {
                        if delim != expected_char {
                            // Extract the raw link text to compute corrected version
                            let link_text = &line[link.start..link.end];
                            let corrected = replace_title_delimiter(link_text, delim, expected_char);
                            signals.push(InconsistentTitleStyle {
                                range: TextRange::new(
                                    base + TextSize::from((offset + link.start) as u32),
                                    base + TextSize::from((offset + link.end) as u32),
                                ),
                                expected: expected_name,
                                actual: delim,
                                corrected,
                            });
                        }
                    }
                }
            }

            offset += line.len() + 1;
        }

        // Check definitions
        let definitions = collect_definitions_from_ast(document);
        for def in definitions {
            if let Some(delim) = def.title_delimiter {
                if delim != expected_char {
                    let lines_vec: Vec<&str> = text.lines().collect();
                    let raw_line = if def.line_index < lines_vec.len() {
                        lines_vec[def.line_index]
                    } else {
                        continue;
                    };
                    let def_text = &raw_line[..def.byte_len.min(raw_line.len())];
                    let corrected = replace_title_delimiter(def_text, delim, expected_char);
                    signals.push(InconsistentTitleStyle {
                        range: TextRange::new(
                            base + TextSize::from(def.byte_offset as u32),
                            base + TextSize::from((def.byte_offset + def.byte_len) as u32),
                        ),
                        expected: expected_name,
                        actual: delim,
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
            markup! { "Use the consistent link title delimiter." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let actual_name = match state.actual {
            '\'' => "single quotes",
            '(' => "parentheses",
            _ => "double quotes",
        };
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Expected "{state.expected}" for link title but found "{actual_name}"."
                },
            )
            .note(markup! {
                "Use consistent title delimiters throughout the document."
            }),
        )
    }
}

/// Replace the title delimiter in a link or definition string.
fn replace_title_delimiter(text: &str, old_delim: char, new_delim: char) -> String {
    let old_close = match old_delim {
        '(' => ')',
        c => c,
    };
    let new_open = match new_delim {
        '(' => '(',
        c => c,
    };
    let new_close = match new_delim {
        '(' => ')',
        c => c,
    };

    // Find the title delimiter positions and replace them
    let mut result = String::with_capacity(text.len());
    let mut found_first = false;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if !found_first && chars[i] == old_delim {
            // Check if this could be the opening title delimiter
            // (after a space inside the link parentheses)
            result.push(new_open);
            found_first = true;
        } else if found_first && chars[i] == old_close {
            // Check if there are no more old_close chars that could be the actual closer
            let remaining_has_close = chars[i + 1..].contains(&old_close);
            if !remaining_has_close {
                result.push(new_close);
            } else {
                result.push(chars[i]);
            }
        } else {
            result.push(chars[i]);
        }
        i += 1;
    }

    result
}
