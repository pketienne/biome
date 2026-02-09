use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};

use crate::MarkdownRuleAction;
use crate::utils::fence_utils::FenceTracker;
use crate::utils::mdx_utils::find_jsx_elements;

declare_lint_rule! {
    /// Enforce sorted attributes on MDX JSX elements.
    ///
    /// Keeping attributes in alphabetical order improves readability and
    /// makes diffs cleaner.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// <Component zebra="1" alpha="2" />
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// <Component alpha="2" zebra="1" />
    /// ```
    pub UseSortedMdxJsxAttributes {
        version: "next",
        name: "useSortedMdxJsxAttributes",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
    }
}

pub struct UnsortedAttribute {
    range: TextRange,
    first_unsorted: String,
    previous: String,
    /// Range covering all attributes for the fix.
    attrs_range: TextRange,
    /// The corrected (sorted) attributes text.
    corrected: String,
}

impl Rule for UseSortedMdxJsxAttributes {
    type Query = Ast<MdDocument>;
    type State = UnsortedAttribute;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut byte_offset: usize = 0;

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);
            if !tracker.is_inside_fence() {
                let elements = find_jsx_elements(line, byte_offset);
                for elem in &elements {
                    if elem.attributes.len() >= 2 {
                        let mut is_unsorted = false;
                        let mut first_unsorted = String::new();
                        let mut previous = String::new();
                        let mut unsorted_range = TextRange::empty(TextSize::from(0u32));

                        for i in 1..elem.attributes.len() {
                            let prev_name = &elem.attributes[i - 1].name;
                            let curr_name = &elem.attributes[i].name;
                            if curr_name.to_lowercase() < prev_name.to_lowercase() {
                                is_unsorted = true;
                                first_unsorted = curr_name.clone();
                                previous = prev_name.clone();
                                unsorted_range = TextRange::new(
                                    base + TextSize::from(
                                        elem.attributes[i].byte_offset as u32,
                                    ),
                                    base + TextSize::from(
                                        (elem.attributes[i].byte_offset
                                            + elem.attributes[i].byte_len)
                                            as u32,
                                    ),
                                );
                                break;
                            }
                        }

                        if is_unsorted {
                            // Build sorted attributes text
                            let first_attr = &elem.attributes[0];
                            let last_attr = elem.attributes.last().unwrap();
                            let attrs_start = first_attr.byte_offset;
                            let attrs_end = last_attr.byte_offset + last_attr.byte_len;
                            let attrs_range = TextRange::new(
                                base + TextSize::from(attrs_start as u32),
                                base + TextSize::from(attrs_end as u32),
                            );

                            // Extract raw text for each attribute and sort
                            let full_text = &text;
                            let mut attr_texts: Vec<(String, String)> = elem
                                .attributes
                                .iter()
                                .map(|a| {
                                    let rel_start = a.byte_offset;
                                    let rel_end = a.byte_offset + a.byte_len;
                                    let raw = full_text
                                        .get(rel_start..rel_end)
                                        .unwrap_or("")
                                        .to_string();
                                    (a.name.to_lowercase(), raw)
                                })
                                .collect();
                            attr_texts.sort_by(|a, b| a.0.cmp(&b.0));
                            let corrected = attr_texts
                                .iter()
                                .map(|(_, raw)| raw.as_str())
                                .collect::<Vec<_>>()
                                .join(" ");

                            signals.push(UnsortedAttribute {
                                range: unsorted_range,
                                first_unsorted,
                                previous,
                                attrs_range,
                                corrected,
                            });
                        }
                    }
                }
            }
            byte_offset += line.len() + 1;
        }

        signals
    }

    fn action(ctx: &RuleContext<Self>, state: &Self::State) -> Option<MarkdownRuleAction> {
        let root = ctx.root();
        let mut token = root
            .syntax()
            .token_at_offset(state.attrs_range.start())
            .right_biased()?;
        let mut tokens = vec![token.clone()];
        while token.text_range().end() < state.attrs_range.end() {
            token = token.next_token()?;
            tokens.push(token.clone());
        }
        let first = &tokens[0];
        let last = tokens.last()?;
        let prefix_len =
            u32::from(state.attrs_range.start() - first.text_range().start()) as usize;
        let suffix_start =
            u32::from(state.attrs_range.end() - last.text_range().start()) as usize;
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
            markup! { "Sort JSX attributes alphabetically." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Attribute \""{ &state.first_unsorted }"\" should come before \""{ &state.previous }"\"."
                },
            )
            .note(markup! {
                "Sort JSX attributes alphabetically."
            }),
        )
    }
}
