use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use biome_rule_options::use_consistent_table_pipe_style::UseConsistentTablePipeStyleOptions;

use crate::MarkdownRuleAction;
use crate::utils::table_utils::{collect_tables, parse_table_row};

declare_lint_rule! {
    /// Enforce consistent table pipe style.
    ///
    /// Table rows can have leading pipes, trailing pipes, both, or neither.
    /// This rule enforces consistency.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// When configured with `"both"` (default):
    ///
    /// ```md
    /// | A | B
    /// | --- | ---
    /// | 1 | 2
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// | A | B |
    /// | --- | --- |
    /// | 1 | 2 |
    /// ```
    ///
    /// ## Options
    ///
    /// ### `style`
    ///
    /// Which pipe style to enforce. Default: `"both"`.
    /// Allowed values: `"leading"`, `"trailing"`, `"both"`, `"consistent"`.
    pub UseConsistentTablePipeStyle {
        version: "next",
        name: "useConsistentTablePipeStyle",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentPipeStyle {
    range: TextRange,
    issue: String,
    corrected: String,
}

impl Rule for UseConsistentTablePipeStyle {
    type Query = Ast<MdDocument>;
    type State = InconsistentPipeStyle;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentTablePipeStyleOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let style = ctx.options().style();
        let tables = collect_tables(&text);
        let lines: Vec<&str> = text.lines().collect();

        let mut signals = Vec::new();
        let mut offsets = Vec::with_capacity(lines.len());
        let mut offset = 0usize;
        for line in &lines {
            offsets.push(offset);
            offset += line.len() + 1;
        }

        for table in &tables {
            let all_lines: Vec<usize> = std::iter::once(table.header_line)
                .chain(std::iter::once(table.separator_line))
                .chain(table.data_lines.iter().copied())
                .collect();

            // For "consistent" mode, determine from first row
            let effective_style = if style == "consistent" {
                let first_row = parse_table_row(lines[table.header_line]);
                if first_row.has_leading_pipe && first_row.has_trailing_pipe {
                    "both"
                } else if first_row.has_leading_pipe {
                    "leading"
                } else if first_row.has_trailing_pipe {
                    "trailing"
                } else {
                    "both"
                }
            } else {
                style
            };

            for &line_idx in &all_lines {
                let row = parse_table_row(lines[line_idx]);
                let issue = match effective_style {
                    "both" => {
                        if !row.has_leading_pipe || !row.has_trailing_pipe {
                            Some("Missing leading or trailing pipe".to_string())
                        } else {
                            None
                        }
                    }
                    "leading" => {
                        if !row.has_leading_pipe {
                            Some("Missing leading pipe".to_string())
                        } else {
                            None
                        }
                    }
                    "trailing" => {
                        if !row.has_trailing_pipe {
                            Some("Missing trailing pipe".to_string())
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some(issue) = issue {
                    let trimmed = lines[line_idx].trim();
                    let mut corrected = trimmed.to_string();
                    // Add missing leading pipe
                    if (effective_style == "both" || effective_style == "leading")
                        && !row.has_leading_pipe
                    {
                        corrected = format!("| {}", corrected);
                    }
                    // Add missing trailing pipe
                    if (effective_style == "both" || effective_style == "trailing")
                        && !row.has_trailing_pipe
                    {
                        corrected = format!("{} |", corrected);
                    }
                    signals.push(InconsistentPipeStyle {
                        range: TextRange::new(
                            base + TextSize::from(offsets[line_idx] as u32),
                            base + TextSize::from((offsets[line_idx] + lines[line_idx].len()) as u32),
                        ),
                        issue,
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
            markup! { "Normalize table pipe style." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    { &state.issue }" in table row."
                },
            )
            .note(markup! {
                "Use consistent pipe style throughout the table."
            }),
        )
    }
}
