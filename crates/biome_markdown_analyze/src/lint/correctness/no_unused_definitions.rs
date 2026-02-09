use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use crate::utils::definition_utils::{collect_definitions, normalize_label};
use crate::utils::fence_utils::FenceTracker;
use crate::utils::inline_utils::{find_code_spans, find_reference_links};

declare_lint_rule! {
    /// Disallow unused link reference definitions.
    ///
    /// Link reference definitions that are not referenced anywhere in the
    /// document are unnecessary and should be removed.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// Some text.
    ///
    /// [unused]: https://example.com
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// See [used][] for details.
    ///
    /// [used]: https://example.com
    /// ```
    pub NoUnusedDefinitions {
        version: "next",
        name: "noUnusedDefinitions",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct UnusedDefinition {
    range: TextRange,
    label: String,
}

impl Rule for NoUnusedDefinitions {
    type Query = Ast<MdDocument>;
    type State = UnusedDefinition;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let definitions = collect_definitions(&text);

        if definitions.is_empty() {
            return Vec::new();
        }

        // Collect all referenced labels
        let mut referenced_labels = std::collections::HashSet::new();
        let mut tracker = FenceTracker::new();

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);

            if !tracker.is_inside_fence() {
                let code_spans = find_code_spans(line);
                let refs = find_reference_links(line, &code_spans);

                for rlink in refs {
                    let label = if rlink.label.is_empty() {
                        normalize_label(&rlink.text)
                    } else {
                        normalize_label(&rlink.label)
                    };
                    referenced_labels.insert(label);
                }
            }
        }

        // Report definitions that are not referenced
        let mut signals = Vec::new();
        for def in &definitions {
            if !referenced_labels.contains(&def.label) {
                signals.push(UnusedDefinition {
                    range: TextRange::new(
                        base + TextSize::from(def.byte_offset as u32),
                        base + TextSize::from((def.byte_offset + def.byte_len) as u32),
                    ),
                    label: def.label.clone(),
                });
            }
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Definition \""{ &state.label }"\" is not used anywhere in the document."
                },
            )
            .note(markup! {
                "Remove the unused definition or add a reference to it."
            }),
        )
    }
}
