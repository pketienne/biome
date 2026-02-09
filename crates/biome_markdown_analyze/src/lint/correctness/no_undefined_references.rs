use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::definition_utils::{collect_definitions, normalize_label};
use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{find_code_spans, find_reference_links};

declare_lint_rule! {
    /// Disallow reference links and images that use an undefined label.
    ///
    /// Reference links and images must use a label that has a corresponding
    /// definition somewhere in the document.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// This is [undefined][missing].
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// This is [defined][label].
    ///
    /// [label]: https://example.com
    /// ```
    pub NoUndefinedReferences {
        version: "next",
        name: "noUndefinedReferences",
        language: "md",
        recommended: true,
        severity: Severity::Error,
    }
}

pub struct UndefinedReference {
    range: TextRange,
    label: String,
}

impl Rule for NoUndefinedReferences {
    type Query = Ast<MdDocument>;
    type State = UndefinedReference;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let definitions = collect_definitions(&text);
        let defined_labels: std::collections::HashSet<String> =
            definitions.iter().map(|d| d.label.clone()).collect();

        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);

            if !tracker.is_inside_fence() {
                let code_spans = find_code_spans(line);
                let refs = find_reference_links(line, &code_spans);

                for ref_link in refs {
                    let lookup_label = if ref_link.label.is_empty() {
                        normalize_label(&ref_link.text)
                    } else {
                        normalize_label(&ref_link.label)
                    };

                    if !defined_labels.contains(&lookup_label) {
                        signals.push(UndefinedReference {
                            range: TextRange::new(
                                base + TextSize::from((offset + ref_link.start) as u32),
                                base + TextSize::from((offset + ref_link.end) as u32),
                            ),
                            label: lookup_label,
                        });
                    }
                }
            }

            offset += line.len() + 1;
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Reference label \""{ &state.label }"\" is not defined."
                },
            )
            .note(markup! {
                "Add a definition like "<Emphasis>"["{ &state.label }"]: url"</Emphasis>" to resolve this reference."
            }),
        )
    }
}
