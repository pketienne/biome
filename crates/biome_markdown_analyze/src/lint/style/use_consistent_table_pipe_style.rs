use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::{MdTable, MdTableRow};
use biome_rowan::{AstNode, AstNodeList, BatchMutationExt, TextRange};

use biome_rule_options::use_consistent_table_pipe_style::UseConsistentTablePipeStyleOptions;

use crate::MarkdownRuleAction;

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

fn row_pipe_style(row: &MdTableRow) -> (bool, bool) {
    let text = row.syntax().text_trimmed().to_string();
    let trimmed = text.trim();
    (trimmed.starts_with('|'), trimmed.ends_with('|'))
}

impl Rule for UseConsistentTablePipeStyle {
    type Query = Ast<MdTable>;
    type State = InconsistentPipeStyle;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentTablePipeStyleOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let table = ctx.query();
        let style = ctx.options().style();
        let mut signals = Vec::new();

        let header = match table.header() {
            Ok(h) => h,
            Err(_) => return Vec::new(),
        };
        let separator = match table.separator() {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        // For "consistent" mode, determine from header row
        let effective_style = if style == "consistent" {
            let (has_leading, has_trailing) = row_pipe_style(&header);
            if has_leading && has_trailing {
                "both"
            } else if has_leading {
                "leading"
            } else if has_trailing {
                "trailing"
            } else {
                "both"
            }
        } else {
            style
        };

        // Check all rows: header, separator, data
        let data_rows: Vec<_> = table.rows().iter().collect();
        let all_rows: Vec<&MdTableRow> = std::iter::once(&header)
            .chain(std::iter::once(&separator))
            .chain(data_rows.iter())
            .collect();

        for row in all_rows {
            let (has_leading, has_trailing) = row_pipe_style(row);
            let issue = match effective_style {
                "both" => {
                    if !has_leading || !has_trailing {
                        Some("Missing leading or trailing pipe".to_string())
                    } else {
                        None
                    }
                }
                "leading" => {
                    if !has_leading {
                        Some("Missing leading pipe".to_string())
                    } else {
                        None
                    }
                }
                "trailing" => {
                    if !has_trailing {
                        Some("Missing trailing pipe".to_string())
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(issue) = issue {
                let row_text = row.syntax().text_trimmed().to_string();
                let trimmed = row_text.trim();
                let mut corrected = trimmed.to_string();
                if (effective_style == "both" || effective_style == "leading") && !has_leading {
                    corrected = format!("| {}", corrected);
                }
                if (effective_style == "both" || effective_style == "trailing") && !has_trailing {
                    corrected = format!("{} |", corrected);
                }
                signals.push(InconsistentPipeStyle {
                    range: row.syntax().text_trimmed_range(),
                    issue,
                    corrected,
                });
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
