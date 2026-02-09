use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::use_consistent_link_title_style::UseConsistentLinkTitleStyleOptions;

use crate::utils::definition_utils::collect_definitions;
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
    }
}

pub struct InconsistentTitleStyle {
    range: TextRange,
    expected: &'static str,
    actual: char,
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
                            signals.push(InconsistentTitleStyle {
                                range: TextRange::new(
                                    base + TextSize::from((offset + link.start) as u32),
                                    base + TextSize::from((offset + link.end) as u32),
                                ),
                                expected: expected_name,
                                actual: delim,
                            });
                        }
                    }
                }
            }

            offset += line.len() + 1;
        }

        // Check definitions
        let definitions = collect_definitions(&text);
        for def in definitions {
            if let Some(delim) = def.title_delimiter {
                if delim != expected_char {
                    signals.push(InconsistentTitleStyle {
                        range: TextRange::new(
                            base + TextSize::from(def.byte_offset as u32),
                            base + TextSize::from((def.byte_offset + def.byte_len) as u32),
                        ),
                        expected: expected_name,
                        actual: delim,
                    });
                }
            }
        }

        signals
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
