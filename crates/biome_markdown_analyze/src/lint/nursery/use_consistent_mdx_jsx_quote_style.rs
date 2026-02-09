use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};
use biome_rule_options::use_consistent_mdx_jsx_quote_style::UseConsistentMdxJsxQuoteStyleOptions;

use crate::utils::fence_utils::FenceTracker;
use crate::utils::mdx_utils::find_jsx_elements;

declare_lint_rule! {
    /// Enforce a consistent quote style for MDX JSX attributes.
    ///
    /// Using a consistent quote style makes the document easier to read
    /// and maintain. By default, the rule enforces consistency (whichever
    /// style appears first is expected for the rest of the file).
    ///
    /// ## Options
    ///
    /// ### `quote`
    ///
    /// The expected quote style: `"double"`, `"single"`, or `"consistent"` (default).
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// <Component name="hello" title='world' />
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// <Component name="hello" title="world" />
    /// ```
    pub UseConsistentMdxJsxQuoteStyle {
        version: "next",
        name: "useConsistentMdxJsxQuoteStyle",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct InconsistentQuote {
    range: TextRange,
    actual: char,
    expected: char,
}

impl Rule for UseConsistentMdxJsxQuoteStyle {
    type Query = Ast<MdDocument>;
    type State = InconsistentQuote;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentMdxJsxQuoteStyleOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut byte_offset: usize = 0;

        let quote_opt = ctx.options().quote();
        let mut expected_quote: Option<char> = match quote_opt {
            "double" => Some('"'),
            "single" => Some('\''),
            _ => None, // consistent
        };

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);
            if !tracker.is_inside_fence() {
                let elements = find_jsx_elements(line, byte_offset);
                for elem in &elements {
                    for attr in &elem.attributes {
                        if let Some(q) = attr.quote_char {
                            match expected_quote {
                                None => {
                                    expected_quote = Some(q);
                                }
                                Some(exp) => {
                                    if q != exp {
                                        signals.push(InconsistentQuote {
                                            range: TextRange::new(
                                                base + TextSize::from(attr.byte_offset as u32),
                                                base + TextSize::from(
                                                    (attr.byte_offset + attr.byte_len) as u32,
                                                ),
                                            ),
                                            actual: q,
                                            expected: exp,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
            byte_offset += line.len() + 1;
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let actual = if state.actual == '"' {
            "double"
        } else {
            "single"
        };
        let expected = if state.expected == '"' {
            "double"
        } else {
            "single"
        };
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Attribute uses "{ actual }" quotes but "{ expected }" quotes are expected."
                },
            )
            .note(markup! {
                "Use a consistent quote style for JSX attributes."
            }),
        )
    }
}
