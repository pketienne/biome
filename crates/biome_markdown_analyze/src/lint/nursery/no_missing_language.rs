use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, TextRange, TextSize};

declare_lint_rule! {
    /// Require a language tag on fenced code blocks.
    ///
    /// Fenced code blocks without a language specifier make it harder for readers
    /// to understand the code and prevent syntax highlighting in rendered output.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ````md
    /// ```
    /// const x = 1;
    /// ```
    /// ````
    ///
    /// ### Valid
    ///
    /// ````md
    /// ```js
    /// const x = 1;
    /// ```
    /// ````
    pub NoMissingLanguage {
        version: "next",
        name: "noMissingLanguage",
        language: "md",
        recommended: true,
        severity: Severity::Warning,
    }
}

pub struct MissingLanguage {
    range: TextRange,
}

impl Rule for NoMissingLanguage {
    type Query = Ast<MdDocument>;
    type State = MissingLanguage;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_trimmed().to_string();
        let start = document.syntax().text_trimmed_range().start();
        let mut signals = Vec::new();
        // Track whether we're inside a fenced code block
        let mut in_fence: Option<(char, usize)> = None; // (fence_char, min_count)

        for (line_idx, line) in text.lines().enumerate() {
            let trimmed: &str = line.trim_start();

            // Detect fence lines (both opening and closing)
            let mut is_fence_line = false;
            if let Some(fence_char) = if trimmed.starts_with("```") {
                Some('`')
            } else if trimmed.starts_with("~~~") {
                Some('~')
            } else {
                None
            } {
                let fence_count = trimmed.chars().take_while(|&c| c == fence_char).count();
                if fence_count >= 3 {
                    is_fence_line = true;
                    if let Some((open_char, open_count)) = in_fence {
                        // We're inside a fence — check if this closes it
                        let rest = trimmed[fence_count..].trim();
                        if fence_char == open_char && fence_count >= open_count && rest.is_empty()
                        {
                            in_fence = None;
                            continue;
                        }
                    } else {
                        // Not inside a fence — this is an opening fence
                        let info_string = trimmed[fence_count..].trim();
                        in_fence = Some((fence_char, fence_count));

                        if info_string.is_empty() {
                            let line_offset: usize = text
                                .lines()
                                .take(line_idx)
                                .map(|l: &str| l.len() + 1)
                                .sum();
                            let offset = TextSize::from(line_offset as u32);
                            let len = TextSize::from(line.len() as u32);
                            signals.push(MissingLanguage {
                                range: TextRange::new(start + offset, start + offset + len),
                            });
                        }
                        continue;
                    }
                }
            }

            // Skip lines inside fenced code blocks
            if in_fence.is_some() && !is_fence_line {
                continue;
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
                    "Fenced code blocks should have a language tag."
                },
            )
            .note(markup! {
                "Add a language identifier after the opening fence to enable syntax highlighting."
            }),
        )
    }
}
