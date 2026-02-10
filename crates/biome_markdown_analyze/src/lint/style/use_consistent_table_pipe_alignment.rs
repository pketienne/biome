use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdTable;
use biome_rowan::{AstNode, AstNodeList, BatchMutationExt, TextRange};

use crate::MarkdownRuleAction;

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
    type Query = Ast<MdTable>;
    type State = MisalignedPipes;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let table = ctx.query();
        let mut signals = Vec::new();

        let header = match table.header() {
            Ok(h) => h,
            Err(_) => return Vec::new(),
        };
        let separator = match table.separator() {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        // Collect all row texts
        let header_text = header.syntax().text_trimmed().to_string();
        let separator_text = separator.syntax().text_trimmed().to_string();
        let data_rows: Vec<_> = table.rows().iter().collect();
        let data_texts: Vec<String> = data_rows
            .iter()
            .map(|r| r.syntax().text_trimmed().to_string())
            .collect();

        let all_texts: Vec<&str> = std::iter::once(header_text.as_str())
            .chain(std::iter::once(separator_text.as_str()))
            .chain(data_texts.iter().map(|s| s.as_str()))
            .collect();

        if all_texts.len() < 2 {
            return Vec::new();
        }

        // Check if all rows have the same trimmed length
        let first_len = all_texts[0].trim().len();
        let all_same_len = all_texts.iter().all(|t| t.trim().len() == first_len);
        if all_same_len {
            return Vec::new();
        }

        // Find pipe positions in the first row (header)
        let first_pipes: Vec<usize> = all_texts[0]
            .char_indices()
            .filter(|(_, c)| *c == '|')
            .map(|(i, _)| i)
            .collect();

        // Check separator
        let sep_pipes: Vec<usize> = separator_text
            .char_indices()
            .filter(|(_, c)| *c == '|')
            .map(|(i, _)| i)
            .collect();
        if sep_pipes != first_pipes {
            signals.push(MisalignedPipes {
                range: separator.syntax().text_trimmed_range(),
                corrected: align_row_to_reference(&header_text, &separator_text),
            });
        }

        // Check data rows
        for (i, row) in data_rows.iter().enumerate() {
            let this_pipes: Vec<usize> = data_texts[i]
                .char_indices()
                .filter(|(_, c)| *c == '|')
                .map(|(i, _)| i)
                .collect();
            if this_pipes != first_pipes {
                signals.push(MisalignedPipes {
                    range: row.syntax().text_trimmed_range(),
                    corrected: align_row_to_reference(&header_text, &data_texts[i]),
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

fn align_row_to_reference(reference: &str, target: &str) -> String {
    let ref_trimmed = reference.trim();
    let tgt_trimmed = target.trim();

    let ref_has_leading = ref_trimmed.starts_with('|');
    let ref_has_trailing = ref_trimmed.ends_with('|');

    let ref_inner = strip_pipes(ref_trimmed);
    let tgt_inner = strip_pipes(tgt_trimmed);

    let ref_segments: Vec<&str> = ref_inner.split('|').collect();
    let tgt_segments: Vec<&str> = tgt_inner.split('|').collect();

    let mut parts = Vec::new();
    for (i, tgt_seg) in tgt_segments.iter().enumerate() {
        if let Some(&ref_seg) = ref_segments.get(i) {
            let ref_width = ref_seg.len();
            let tgt_content = tgt_seg.trim();
            if ref_width >= tgt_content.len() + 2 {
                let pad = ref_width - tgt_content.len() - 1;
                parts.push(format!(" {}{}", tgt_content, " ".repeat(pad)));
            } else {
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
