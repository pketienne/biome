use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

use biome_rule_options::no_long_lines::NoLongLinesOptions;

use crate::utils::fence_utils::FenceTracker;

declare_lint_rule! {
    /// Enforce a maximum line length.
    ///
    /// Long lines make markdown files harder to read and review in
    /// side-by-side diffs. Keeping lines within a reasonable length
    /// improves readability.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// This is a very long line that exceeds the default maximum line length of eighty characters set by this lint rule.
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// This line fits within the limit.
    /// ```
    ///
    /// ## Options
    ///
    /// ### `maxLength`
    ///
    /// Maximum line length. Default: `80`.
    ///
    /// ### `allowInCodeBlocks`
    ///
    /// Whether to allow long lines inside fenced code blocks. Default: `true`.
    ///
    /// ### `allowUrls`
    ///
    /// Whether to allow long lines that contain URLs. Default: `true`.
    pub NoLongLines {
        version: "next",
        name: "noLongLines",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct LongLine {
    range: TextRange,
    length: usize,
}

fn line_contains_url(line: &str) -> bool {
    line.contains("http://") || line.contains("https://") || line.contains("ftp://")
}

impl Rule for NoLongLines {
    type Query = Ast<MdDocument>;
    type State = LongLine;
    type Signals = Vec<Self::State>;
    type Options = NoLongLinesOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_trimmed().to_string();
        let base = document.syntax().text_trimmed_range().start();
        let max_length = ctx.options().max_length() as usize;
        let allow_code = ctx.options().allow_in_code_blocks();
        let allow_urls = ctx.options().allow_urls();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut offset = 0usize;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);

            let length = line.len();
            if length > max_length {
                let skip_code = allow_code && tracker.is_inside_fence();
                let skip_url = allow_urls && line_contains_url(line);
                if !skip_code && !skip_url {
                    signals.push(LongLine {
                        range: TextRange::new(
                            base + TextSize::from(offset as u32),
                            base + TextSize::from((offset + length) as u32),
                        ),
                        length,
                    });
                }
            }

            offset += line.len() + 1;
        }

        signals
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let max = ctx.options().max_length();
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Line length "{state.length.to_string()}" exceeds maximum "{max.to_string()}"."
                },
            )
            .note(markup! {
                "Keep lines within "{max.to_string()}" characters."
            }),
        )
    }
}
