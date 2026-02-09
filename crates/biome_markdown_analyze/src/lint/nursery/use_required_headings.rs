use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdDocument, MdHeader};
use biome_rowan::{AstNode, AstNodeList};

use biome_rule_options::use_required_headings::UseRequiredHeadingsOptions;

use crate::utils::fence_utils::FenceTracker;

declare_lint_rule! {
    /// Enforce a required heading structure in the document.
    ///
    /// This rule checks that the document contains the specified headings
    /// in order. Use `*` as a wildcard to match any heading.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `headings: ["# Title", "## Introduction"]`:
    ///
    /// ```md
    /// # Title
    ///
    /// ## Summary
    /// ```
    ///
    /// ### Valid
    ///
    /// When configured with `headings: ["# Title", "## Introduction"]`:
    ///
    /// ```md
    /// # Title
    ///
    /// ## Introduction
    /// ```
    ///
    /// ## Options
    ///
    /// ### `headings`
    ///
    /// A list of required heading strings. Use `*` as a wildcard
    /// to match any heading at any level. Default: `[]` (no requirements).
    pub UseRequiredHeadings {
        version: "next",
        name: "useRequiredHeadings",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct MissingHeading {
    expected: String,
    position: usize,
}

impl Rule for UseRequiredHeadings {
    type Query = Ast<MdDocument>;
    type State = MissingHeading;
    type Signals = Vec<Self::State>;
    type Options = UseRequiredHeadingsOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let required = ctx.options().headings();
        if required.is_empty() {
            return Vec::new();
        }

        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();

        // Collect actual headings from the document
        let actual_headings = collect_headings(document, &text);

        let mut signals = Vec::new();
        let mut actual_idx = 0;

        for (req_idx, required_heading) in required.iter().enumerate() {
            if required_heading == "*" {
                // Wildcard: skip one actual heading
                if actual_idx < actual_headings.len() {
                    actual_idx += 1;
                }
                continue;
            }

            if actual_idx < actual_headings.len() {
                if actual_headings[actual_idx] == *required_heading {
                    actual_idx += 1;
                    continue;
                }
            }

            // Missing or mismatched heading
            signals.push(MissingHeading {
                expected: required_heading.clone(),
                position: req_idx + 1,
            });
        }

        signals
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let document = ctx.query();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                document.syntax().text_trimmed_range(),
                markup! {
                    "Required heading at position "{state.position}" is missing: \""{ &state.expected }"\"."
                },
            )
            .note(markup! {
                "Add the required heading to match the expected document structure."
            }),
        )
    }
}

fn collect_headings(document: &MdDocument, text: &str) -> Vec<String> {
    let mut headings = Vec::new();
    let mut tracker = FenceTracker::new();
    let lines: Vec<&str> = text.lines().collect();

    // First try to collect from AST
    for node in document.syntax().descendants() {
        if let Some(header) = MdHeader::cast_ref(&node) {
            let level = header.before().len();
            let content = header
                .before()
                .syntax()
                .parent()
                .map(|p| {
                    let full_text = p.text_trimmed().to_string();
                    full_text.get(level..).unwrap_or("").trim().to_string()
                })
                .unwrap_or_default();
            let prefix: String = "#".repeat(level);
            headings.push(format!("{prefix} {content}"));
        }
    }

    if !headings.is_empty() {
        return headings;
    }

    // Fallback to text-based extraction
    for (line_idx, line) in lines.iter().enumerate() {
        tracker.process_line(line_idx, line);
        if tracker.is_inside_fence() {
            continue;
        }
        let trimmed = line.trim_start();
        let hash_count = trimmed.bytes().take_while(|&b| b == b'#').count();
        if hash_count >= 1 && hash_count <= 6 {
            let after = &trimmed[hash_count..];
            if after.is_empty() || after.starts_with(' ') {
                let content = after.trim();
                let prefix: String = "#".repeat(hash_count);
                headings.push(format!("{prefix} {content}"));
            }
        }
    }

    headings
}
