use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use biome_rule_options::use_consistent_link_style::UseConsistentLinkStyleOptions;

use crate::MarkdownRuleAction;
use crate::utils::definition_utils::{collect_definitions, normalize_label};
use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{find_code_spans, find_inline_links, find_reference_links};

declare_lint_rule! {
    /// Enforce consistent link style.
    ///
    /// Links can be written as inline (`[text](url)`) or reference
    /// (`[text][label]`). This rule enforces a consistent style.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"reference"`, inline links are flagged:
    ///
    /// ```md
    /// [text](https://example.com)
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// [text][label]
    ///
    /// [label]: https://example.com
    /// ```
    ///
    /// ## Options
    ///
    /// ### `style`
    ///
    /// Which link style to enforce. Default: `"inline"`.
    /// Allowed values: `"inline"`, `"reference"`, `"consistent"`.
    pub UseConsistentLinkStyle {
        version: "next",
        name: "useConsistentLinkStyle",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct InconsistentLinkStyle {
    range: TextRange,
    expected: &'static str,
    actual: &'static str,
    corrected: Option<String>,
}

impl Rule for UseConsistentLinkStyle {
    type Query = Ast<MdDocument>;
    type State = InconsistentLinkStyle;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentLinkStyleOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let style = ctx.options().style();

        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;

        // For "consistent" mode, determine from first link found
        let mut first_style: Option<&str> = if style == "consistent" {
            None
        } else {
            Some(style)
        };

        // First pass for consistent mode: find the first link style
        if first_style.is_none() {
            let mut temp_tracker = FenceTracker::new();
            let mut temp_offset = 0usize;
            'outer: for (line_idx, line) in text.lines().enumerate() {
                temp_tracker.process_line(line_idx, line);
                if !temp_tracker.is_inside_fence() {
                    let code_spans = find_code_spans(line);
                    let inline = find_inline_links(line, &code_spans);
                    let refs = find_reference_links(line, &code_spans);
                    // Check non-image links only
                    for link in &inline {
                        if !link.is_image {
                            first_style = Some("inline");
                            break 'outer;
                        }
                    }
                    for rlink in &refs {
                        if !rlink.is_image {
                            first_style = Some("reference");
                            break 'outer;
                        }
                    }
                }
                temp_offset += line.len() + 1;
            }
            let _ = temp_offset;
        }

        let expected_style = match first_style {
            Some(s) => s,
            None => return signals, // no links found
        };

        // Collect definitions for reference->inline conversion
        let definitions = collect_definitions(&text);

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);

            if !tracker.is_inside_fence() {
                let code_spans = find_code_spans(line);

                if expected_style == "reference" {
                    let inline = find_inline_links(line, &code_spans);
                    for link in inline {
                        if !link.is_image {
                            // Convert inline to reference: [text][text]
                            let corrected = Some(format!("[{}][{}]", link.text, link.text));
                            signals.push(InconsistentLinkStyle {
                                range: TextRange::new(
                                    base + TextSize::from((offset + link.start) as u32),
                                    base + TextSize::from((offset + link.end) as u32),
                                ),
                                expected: "reference",
                                actual: "inline",
                                corrected,
                            });
                        }
                    }
                } else if expected_style == "inline" {
                    let refs = find_reference_links(line, &code_spans);
                    for rlink in refs {
                        if !rlink.is_image {
                            // Convert reference to inline: look up definition URL
                            let label_key = if rlink.label.is_empty() {
                                normalize_label(&rlink.text)
                            } else {
                                normalize_label(&rlink.label)
                            };
                            let corrected = definitions
                                .iter()
                                .find(|d| d.label == label_key)
                                .map(|d| format!("[{}]({})", rlink.text, d.url));
                            signals.push(InconsistentLinkStyle {
                                range: TextRange::new(
                                    base + TextSize::from((offset + rlink.start) as u32),
                                    base + TextSize::from((offset + rlink.end) as u32),
                                ),
                                expected: "inline",
                                actual: "reference",
                                corrected,
                            });
                        }
                    }
                }
            }

            offset += line.len() + 1;
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let corrected = state.corrected.as_ref()?;
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
        let new_text = format!("{}{}{}", prefix, corrected, suffix);
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
            markup! { "Convert to "{state.expected}" link style." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Expected "{state.expected}" link style but found "{state.actual}"."
                },
            )
            .note(markup! {
                "Use a consistent link style throughout the document."
            }),
        )
    }
}
