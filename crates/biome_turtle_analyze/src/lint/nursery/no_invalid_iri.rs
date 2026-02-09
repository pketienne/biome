use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_diagnostics::Severity;
use biome_rowan::{AstNode, Direction, TextRange};
use biome_turtle_syntax::{TurtleRoot, TurtleSyntaxKind};

declare_lint_rule! {
    /// Disallow invalid characters in IRI references.
    ///
    /// IRIs enclosed in angle brackets (`<...>`) must not contain certain
    /// characters such as spaces, `<`, `>`, `{`, `}`, `|`, `^`, or backtick.
    /// These characters are forbidden by the Turtle specification and will
    /// cause processing errors.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```turtle,expect_diagnostic
    /// @prefix ex: <http://example.org/> .
    /// <http://example.org/hello world> ex:name "test" .
    /// ```
    ///
    /// ### Valid
    ///
    /// ```turtle
    /// @prefix ex: <http://example.org/> .
    /// <http://example.org/hello%20world> ex:name "test" .
    /// ```
    ///
    pub NoInvalidIri {
        version: "next",
        name: "noInvalidIri",
        language: "turtle",
        recommended: true,
        severity: Severity::Error,
    }
}

pub struct InvalidIri {
    character: char,
    range: TextRange,
}

const INVALID_IRI_CHARS: &[char] = &[' ', '\t', '<', '>', '{', '}', '|', '^', '`'];

impl Rule for NoInvalidIri {
    type Query = Ast<TurtleRoot>;
    type State = InvalidIri;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let root = ctx.query();
        let mut signals = Vec::new();

        for token in root.syntax().descendants_with_tokens(Direction::Next) {
            if let Some(token) = token.into_token() {
                if token.kind() == TurtleSyntaxKind::TURTLE_IRIREF_LITERAL {
                    let text = token.text_trimmed();
                    // Strip the < and > delimiters
                    let inner = if text.starts_with('<') && text.ends_with('>') {
                        &text[1..text.len() - 1]
                    } else {
                        text
                    };
                    for ch in inner.chars() {
                        if INVALID_IRI_CHARS.contains(&ch) || ch.is_control() {
                            signals.push(InvalidIri {
                                character: ch,
                                range: token.text_trimmed_range(),
                            });
                            break; // One diagnostic per IRI
                        }
                    }
                }
            }
        }

        signals
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let char_desc = match state.character {
            ' ' => "space".to_string(),
            '\t' => "tab".to_string(),
            c if c.is_control() => format!("control character U+{:04X}", c as u32),
            c => format!("'{c}'"),
        };
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.range,
                markup! {
                    "IRI contains invalid character: "{ &char_desc }"."
                },
            )
            .note(markup! {
                "Percent-encode invalid characters in IRIs."
            }),
        )
    }
}
