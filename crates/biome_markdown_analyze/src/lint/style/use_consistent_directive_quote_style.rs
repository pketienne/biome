use biome_analyze::{
    Ast, FixKind, Rule, RuleAction, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_markdown_syntax::MdDocument;
use biome_rowan::{AstNode, BatchMutationExt, TextRange, TextSize};
use biome_rule_options::use_consistent_directive_quote_style::UseConsistentDirectiveQuoteStyleOptions;

use crate::MarkdownRuleAction;
use crate::utils::directive_utils::find_directives;
use crate::utils::fence_utils::FenceTracker;

declare_lint_rule! {
    /// Enforce a consistent quote style for directive attributes.
    ///
    /// Using a consistent quote style makes the document easier to read
    /// and maintain. By default, the rule enforces consistency (whichever
    /// style appears first is expected for the rest of the file).
    ///
    /// ## Options
    ///
    /// ### `quote`
    ///
    /// The expected quote style: `"double"`, `"single"`, or `"consistent"` (default).
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```md
    /// ::video{src="video.mp4" title='My Video'}
    /// ```
    ///
    /// ### Valid
    ///
    /// ```md
    /// ::video{src="video.mp4" title="My Video"}
    /// ```
    pub UseConsistentDirectiveQuoteStyle {
        version: "next",
        name: "useConsistentDirectiveQuoteStyle",
        language: "md",
        recommended: false,
        severity: Severity::Warning,
        fix_kind: FixKind::Safe,
    }
}

pub struct InconsistentDirQuote {
    range: TextRange,
    actual: char,
    expected: char,
    corrected: String,
}

impl Rule for UseConsistentDirectiveQuoteStyle {
    type Query = Ast<MdDocument>;
    type State = InconsistentDirQuote;
    type Signals = Vec<Self::State>;
    type Options = UseConsistentDirectiveQuoteStyleOptions;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let document = ctx.query();
        let text = document.syntax().text_with_trivia().to_string();
        let base = document.syntax().text_range_with_trivia().start();
        let mut signals = Vec::new();
        let mut tracker = FenceTracker::new();
        let mut byte_offset: usize = 0;

        let quote_opt = ctx.options().quote();
        let mut expected_quote: Option<char> = match quote_opt {
            "double" => Some('"'),
            "single" => Some('\''),
            _ => None,
        };

        for (line_idx, line) in text.lines().enumerate() {
            tracker.process_line(line_idx, line);
            if !tracker.is_inside_fence() {
                let directives = find_directives(line, byte_offset);
                for dir in &directives {
                    for attr in &dir.attributes {
                        if let Some(q) = attr.quote_char {
                            match expected_quote {
                                None => {
                                    expected_quote = Some(q);
                                }
                                Some(exp) => {
                                    if q != exp {
                                        // Directive attr value is without quotes
                                        let val = attr.value.as_deref().unwrap_or("");
                                        let corrected =
                                            format!("{}={}{}{}", attr.name, exp, val, exp);
                                        signals.push(InconsistentDirQuote {
                                            range: TextRange::new(
                                                base + TextSize::from(attr.byte_offset as u32),
                                                base + TextSize::from(
                                                    (attr.byte_offset + attr.byte_len) as u32,
                                                ),
                                            ),
                                            actual: q,
                                            expected: exp,
                                            corrected,
                                        });
                                    }
                                }
                            }
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
            markup! { "Use consistent quote style." }.to_owned(),
            mutation,
        ))
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let actual = if state.actual == '"' {
            "double"
        } else {
            "single"
        };
        let expected = if state.expected == '"' {
            "double"
        } else {
            "single"
        };
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "Attribute uses "{ actual }" quotes but "{ expected }" quotes are expected."
                },
            )
            .note(markup! {
                "Use a consistent quote style for directive attributes."
            }),
        )
    }
}
