use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange};
use biome_turtle_syntax::{TurtleRoot, TurtleSyntaxKind};

declare_lint_rule! {
    /// Enforce blank lines between triple statement blocks.
    ///
    /// Turtle documents are more readable when consecutive triple statement
    /// blocks (different subjects) are separated by at least one blank line.
    /// This rule flags consecutive `TurtleTriples` nodes that have different
    /// subjects and no blank line between them.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix ex: <http://example.org/> .
    /// ex:alice ex:name "Alice" .
    /// ex:bob ex:name "Bob" .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix ex: <http://example.org/> .
    ///
    /// ex:alice ex:name "Alice" .
    ///
    /// ex:bob ex:name "Bob" .
    /// ```
    ///
    pub UseBlanksAroundBlocks {
        version: "next",
        name: "useBlanksAroundBlocks",
        language: "turtle",
        recommended: false,
        severity: Severity::Information,
    }
}

pub struct MissingBlankLine {
    range: TextRange,
    subject: String,
}

impl Rule for UseBlanksAroundBlocks {
    type Query = Ast<TurtleRoot>;
    type State = MissingBlankLine;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut signals = Vec::new();

        // Collect all TurtleTriples nodes in document order
        let triples_nodes: Vec<_> = root
            .syntax()
            .children()
            .flat_map(|child| child.children())
            .filter(|node| node.kind() == TurtleSyntaxKind::TURTLE_TRIPLES)
            .collect();

        for pair in triples_nodes.windows(2) {
            let prev = &pair[0];
            let curr = &pair[1];

            let prev_subject = extract_subject_text(prev);
            let curr_subject = extract_subject_text(curr);

            // Only flag when subjects differ — same-subject adjacency is
            // handled by noMissingSemicolonBeforeDot / useGroupedSubjects
            if let (Some(prev_subj), Some(curr_subj)) = (&prev_subject, &curr_subject) {
                if prev_subj == curr_subj {
                    continue;
                }
            }

            if !has_blank_line_before(curr) {
                let subject_text = curr_subject.unwrap_or_default();
                signals.push(MissingBlankLine {
                    range: curr.text_trimmed_range(),
                    subject: subject_text,
                });
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
                    "Triple block for '"{ &state.subject }"' is not separated by a blank line from the previous block."
                },
            )
            .note(markup! {
                "Add a blank line between triple blocks with different subjects for readability."
            }),
        )
    }
}

use biome_rowan::SyntaxNode;
use biome_turtle_syntax::TurtleLanguage;

fn extract_subject_text(triples_node: &SyntaxNode<TurtleLanguage>) -> Option<String> {
    triples_node
        .children()
        .find(|child| child.kind() == TurtleSyntaxKind::TURTLE_SUBJECT)
        .map(|subject| subject.text_trimmed().to_string())
}

/// Check if there's a blank line (2+ newlines) in the trivia before a node.
fn has_blank_line_before(node: &SyntaxNode<TurtleLanguage>) -> bool {
    if let Some(first_token) = node.first_token() {
        let mut newline_count = 0;
        for piece in first_token.leading_trivia().pieces() {
            for ch in piece.text().chars() {
                if ch == '\n' {
                    newline_count += 1;
                }
            }
        }
        if let Some(prev_token) = first_token.prev_token() {
            for piece in prev_token.trailing_trivia().pieces() {
                for ch in piece.text().chars() {
                    if ch == '\n' {
                        newline_count += 1;
                    }
                }
            }
        }
        newline_count >= 2
    } else {
        false
    }
}
