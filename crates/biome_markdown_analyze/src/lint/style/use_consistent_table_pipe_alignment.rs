use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;
use crate::utils::table_utils::collect_tables;

declare_lint_rule! {
    /// Enforce aligned table pipe characters.
    ///
    /// Pipe characters in tables should be vertically aligned for readability.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// | A | B |
    /// | --- | --- |
    /// | long text | x |
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// | A         | B |
    /// | --------- | - |
    /// | long text | x |
    /// ```
    pub UseConsistentTablePipeAlignment {
        version: "next",
        name: "useConsistentTablePipeAlignment",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct MisalignedPipes {
    range: TextRange,
    corrected: String,
}

impl Rule for UseConsistentTablePipeAlignment {
    type Query = Ast<MdDocument>;
    type State = MisalignedPipes;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
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

            if all_lines.len() < 2 {
                continue;
            }

            // Check if all rows have the same length (a simple alignment check)
            let first_len = lines[all_lines[0]].trim().len();
            let all_same_len = all_lines.iter().all(|&l| lines[l].trim().len() == first_len);

            if !all_same_len {
                // Find pipe positions in the first row
                let first_pipes: Vec<usize> = lines[all_lines[0]]
                    .char_indices()
                    .filter(|(_, c)| *c == '|')
                    .map(|(i, _)| i)
                    .collect();

                // Check subsequent rows for pipe alignment
                for &line_idx in all_lines.iter().skip(1) {
                    let this_pipes: Vec<usize> = lines[line_idx]
                        .char_indices()
                        .filter(|(_, c)| *c == '|')
                        .map(|(i, _)| i)
                        .collect();

                    if this_pipes != first_pipes {
                        // Compute corrected row: pad cells to match the widths from the first row
                        let corrected =
                            align_row_to_reference(lines[all_lines[0]], lines[line_idx]);

                        signals.push(MisalignedPipes {
                            range: TextRange::new(
                                base + TextSize::from(offsets[line_idx] as u32),
                                base + TextSize::from(
                                    (offsets[line_idx] + lines[line_idx].len()) as u32,
                                ),
                            ),
                            corrected,
                        });
                    }
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
            markup! { "Align table pipes." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Table pipes are not aligned with other rows."
                },
            )
            .note(markup! {
                "Align pipe characters vertically for readability."
            }),
        )
    }
}

/// Align a table row's cells to match the column widths of a reference row.
///
/// Both rows are expected to be pipe-delimited table rows. The function
/// parses cells from each row, then pads the target row's cells to match
/// the widths observed in the reference row.
fn align_row_to_reference(reference: &str, target: &str) -> String {
    let ref_trimmed = reference.trim();
    let tgt_trimmed = target.trim();

    let ref_has_leading = ref_trimmed.starts_with('|');
    let ref_has_trailing = ref_trimmed.ends_with('|');

    // Extract cell segments (including whitespace) from the reference row
    let ref_inner = strip_pipes(ref_trimmed);
    let tgt_inner = strip_pipes(tgt_trimmed);

    let ref_segments: Vec<&str> = ref_inner.split('|').collect();
    let tgt_segments: Vec<&str> = tgt_inner.split('|').collect();

    // Build the aligned row by padding each target cell to the width of the
    // corresponding reference cell segment (preserving the ` content ` pattern)
    let mut parts = Vec::new();
    for (i, tgt_seg) in tgt_segments.iter().enumerate() {
        if let Some(&ref_seg) = ref_segments.get(i) {
            let ref_width = ref_seg.len();
            let tgt_content = tgt_seg.trim();
            // Pad with space before and after, then fill to ref_width
            if ref_width >= tgt_content.len() + 2 {
                let pad = ref_width - tgt_content.len() - 1; // 1 for leading space
                parts.push(format!(" {}{}", tgt_content, " ".repeat(pad)));
            } else {
                // Can't shrink, just use space-padded content
                parts.push(format!(" {} ", tgt_content));
            }
        } else {
            parts.push(format!(" {} ", tgt_seg.trim()));
        }
    }

    let mut result = String::new();
    if ref_has_leading {
        result.push('|');
    }
    result.push_str(&parts.join("|"));
    if ref_has_trailing {
        result.push('|');
    }
    result
}

fn strip_pipes(s: &str) -> &str {
    let s = if s.starts_with('|') { &s[1..] } else { s };
    if s.ends_with('|') {
        &s[..s.len() - 1]
    } else {
        s
    }
}
