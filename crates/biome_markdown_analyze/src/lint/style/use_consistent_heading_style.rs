use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use biome_rule_options::use_consistent_heading_style::UseConsistentHeadingStyleOptions;

use crate::MarkdownRuleAction;
use crate::utils::fence_utils::FenceTracker;

declare_lint_rule! {
    /// Enforce consistent heading style.
    ///
    /// Headings can use ATX style (`# heading`) or setext style
    /// (underlined with `===` or `---`). This rule enforces consistency.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"atx"` (default), setext headings are flagged:
    ///
    /// ```md
    /// Heading
    /// =======
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// # Heading
    /// ```
    ///
    /// ## Options
    ///
    /// ### `style`
    ///
    /// Which heading style to enforce. Default: `"consistent"`.
    pub UseConsistentHeadingStyle {
        version: "next",
        name: "useConsistentHeadingStyle",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct InconsistentHeadingStyle {
    range: TextRange,
    expected: &'static str,
    actual: &'static str,
    corrected: String,
}

impl Rule for UseConsistentHeadingStyle {
    type Query = Ast<MdDocument>;
    type State = InconsistentHeadingStyle;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentHeadingStyleOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let style = ctx.options().style();
        let lines: Vec<&str> = text.lines().collect();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();

        // Detect heading styles
        let mut headings: Vec<(&'static str, usize, usize)> = Vec::new(); // (style, line, byte_len)

        for (line_idx, line) in lines.iter().enumerate() {
            tracker.process_line(line_idx, line);
            if tracker.is_inside_fence() {
                continue;
            }

            // ATX heading
            if line.starts_with('#') {
                headings.push(("atx", line_idx, line.len()));
            }

            // Setext heading (line of = or - under a non-empty line)
            if line_idx > 0 && !lines[line_idx - 1].trim().is_empty() {
                let trimmed = line.trim();
                if !trimmed.is_empty()
                    && (trimmed.chars().all(|c| c == '=') || trimmed.chars().all(|c| c == '-'))
                    && trimmed.len() >= 2
                {
                    // This could be a setext heading underline
                    // The heading text is the previous line
                    headings.push(("setext", line_idx - 1, lines[line_idx - 1].len()));
                }
            }
        }

        if headings.is_empty() {
            return signals;
        }

        let expected_style = match style {
            "atx" => "atx",
            "setext" => "setext",
            _ => {
                // consistent: use the first heading's style
                headings[0].0
            }
        };

        for &(heading_style, line_idx, _) in &headings {
            if heading_style != expected_style {
                let line_offset: usize =
                    lines[..line_idx].iter().map(|l| l.len() + 1).sum();
                let line = lines[line_idx];

                // Compute corrected text and range
                let (corrected, range) = if heading_style == "setext" && expected_style == "atx" {
                    // Convert setext to ATX: heading text is on this line,
                    // underline is on the next line
                    let heading_text = line.trim();
                    let underline = lines.get(line_idx + 1).unwrap_or(&"");
                    let underline_trimmed = underline.trim();
                    let level = if underline_trimmed.chars().all(|c| c == '=') {
                        1
                    } else {
                        2
                    };
                    let hashes = "#".repeat(level);
                    let corrected = format!("{} {}", hashes, heading_text);
                    // Range spans from heading text line to end of underline
                    let underline_offset: usize =
                        lines[..line_idx + 1].iter().map(|l| l.len() + 1).sum();
                    let range_end = underline_offset + underline.len();
                    let range = TextRange::new(
                        base + TextSize::from(line_offset as u32),
                        base + TextSize::from(range_end as u32),
                    );
                    (corrected, range)
                } else {
                    // Convert ATX to setext: parse the heading level and text
                    let trimmed = line.trim_start();
                    let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
                    let content = trimmed[hash_count..].trim().trim_end_matches('#').trim();
                    let underline_char = if hash_count <= 1 { '=' } else { '-' };
                    let underline_len = content.len().max(3);
                    let underline: String =
                        std::iter::repeat(underline_char).take(underline_len).collect();
                    let corrected = format!("{}\n{}", content, underline);
                    let range = TextRange::new(
                        base + TextSize::from(line_offset as u32),
                        base + TextSize::from((line_offset + line.len()) as u32),
                    );
                    (corrected, range)
                };

                signals.push(InconsistentHeadingStyle {
                    range,
                    expected: expected_style,
                    actual: heading_style,
                    corrected,
                });
            }
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();
        // Collect all tokens overlapping the range
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
            markup! { "Convert to "{state.expected}" heading style." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Expected "{state.expected}" heading style but found "{state.actual}"."
                },
            )
            .note(markup! {
                "Use a consistent heading style throughout the document."
            }),
        )
    }
}
