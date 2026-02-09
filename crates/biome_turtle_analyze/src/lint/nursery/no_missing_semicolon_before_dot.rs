use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, TextRange};
use biome_turtle_syntax::{TurtleRoot, TurtleSyntaxKind};

declare_lint_rule! {
    /// Detect likely missing semicolons where a dot was used instead.
    ///
    /// When consecutive triple statements share the same subject and appear
    /// on adjacent lines (no blank line between them), it often indicates
    /// the author typed `.` when they meant `;`. This rule flags such cases
    /// as likely mistakes.
    ///
    /// This differs from `useGroupedSubjects` which is a general style
    /// suggestion. This rule is a correctness heuristic targeting a common
    /// editing mistake.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix ex: <http://example.org/> .
    /// ex:alice ex:name "Alice" .
    /// ex:alice ex:age "30" .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix ex: <http://example.org/> .
    /// ex:alice ex:name "Alice" ;
    ///     ex:age "30" .
    /// ```
    ///
    pub NoMissingSemicolonBeforeDot {
        version: "next",
        name: "noMissingSemicolonBeforeDot",
        language: "turtle",
        recommended: false,
        severity: Severity::Warning,
    }
}

pub struct MissingSemicolon {
    range: TextRange,
    subject: String,
}

impl Rule for NoMissingSemicolonBeforeDot {
    type Query = Ast<TurtleRoot>;
    type State = MissingSemicolon;
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

            if let (Some(prev_subj), Some(curr_subj)) = (&prev_subject, &curr_subject) {
                if prev_subj == curr_subj && !has_blank_line_before(curr) {
                    signals.push(MissingSemicolon {
                        range: curr.text_trimmed_range(),
                        subject: curr_subj.clone(),
                    });
                }
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
                    "Consecutive triple for '"{ &state.subject }"' may be missing a ';' separator."
                },
            )
            .note(markup! {
                "If these triples share a subject, use ';' instead of '.' to separate predicate-object pairs."
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
    // Get the first token of this node and check its leading trivia
    if let Some(first_token) = node.first_token() {
        let mut newline_count = 0;
        for piece in first_token.leading_trivia().pieces() {
            for ch in piece.text().chars() {
                if ch == '\n' {
                    newline_count += 1;
                }
            }
        }
        // Also check trailing trivia of the previous token
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
