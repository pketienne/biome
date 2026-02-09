use crate::prelude::*;
use biome_formatter::CstFormatContext;
use biome_formatter::prelude::syntax_token_cow_slice;
use biome_formatter::write;
use biome_rowan::{AstNode, Direction};
use biome_turtle_syntax::TurtleRdfLiteral;
use std::borrow::Cow;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleRdfLiteral;

impl FormatNodeRule<TurtleRdfLiteral> for FormatTurtleRdfLiteral {
    fn fmt_fields(&self, node: &TurtleRdfLiteral, f: &mut TurtleFormatter) -> FormatResult<()> {
        // Try literal short notation: "true"^^xsd:boolean -> true, "42"^^xsd:integer -> 42
        if let Some(short) = try_short_notation(node) {
            let string_node = node.value().map_err(|_| biome_formatter::FormatError::SyntaxError)?;
            let token = string_node.value().map_err(|_| biome_formatter::FormatError::SyntaxError)?;

            // Mark the TurtleString node as suppression-checked since we're bypassing its format rule
            f.context().comments().mark_suppression_checked(string_node.syntax());

            // Emit the short form in place of the string token
            write!(
                f,
                [format_replaced(
                    &token,
                    &syntax_token_cow_slice(
                        Cow::Owned(short),
                        &token,
                        token.text_trimmed_range().start(),
                    ),
                )]
            )?;

            // Mark all remaining tokens in the datatype annotation as removed
            if let Some(datatype_annotation) = node.datatype() {
                for descendant in datatype_annotation.syntax().descendants_with_tokens(Direction::Next) {
                    match &descendant {
                        biome_rowan::NodeOrToken::Token(tok) => {
                            write!(f, [format_removed(tok)])?;
                        }
                        biome_rowan::NodeOrToken::Node(node) => {
                            f.context().comments().mark_suppression_checked(node);
                        }
                    }
                }
            }

            return Ok(());
        }

        write!(f, [node.value()?.format()])?;
        if let Some(language_token) = node.language_token() {
            write!(f, [language_token.format()])?;
        }
        if let Some(datatype) = node.datatype() {
            write!(f, [datatype.format()])?;
        }
        Ok(())
    }
}

/// Try to convert a typed literal to its short notation.
///
/// Returns `Some(short_form)` if the literal can be expressed as a bare value:
/// - `"true"^^xsd:boolean` → `true`
/// - `"false"^^xsd:boolean` → `false`
/// - `"42"^^xsd:integer` → `42`
/// - `"3.14"^^xsd:decimal` → `3.14`
fn try_short_notation(node: &TurtleRdfLiteral) -> Option<String> {
    let datatype_annotation = node.datatype()?;
    let datatype_iri = datatype_annotation.datatype().ok()?;
    let datatype_text = datatype_iri.syntax().text_trimmed().to_string();

    let string_node = node.value().ok()?;
    let token = string_node.value().ok()?;
    let token_text = token.text_trimmed();

    // Extract the string content (strip quotes)
    let quote_len = if token_text.starts_with("\"\"\"") || token_text.starts_with("'''") {
        3
    } else if token_text.starts_with('"') || token_text.starts_with('\'') {
        1
    } else {
        return None;
    };
    let content = &token_text[quote_len..token_text.len() - quote_len];

    match datatype_text.as_str() {
        "xsd:boolean" | "<http://www.w3.org/2001/XMLSchema#boolean>" => {
            if content == "true" || content == "false" {
                Some(content.to_string())
            } else {
                None
            }
        }
        "xsd:integer" | "<http://www.w3.org/2001/XMLSchema#integer>" => {
            // Must match [+-]?\d+
            let s = content.strip_prefix(['+', '-']).unwrap_or(content);
            if !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) {
                Some(content.to_string())
            } else {
                None
            }
        }
        "xsd:decimal" | "<http://www.w3.org/2001/XMLSchema#decimal>" => {
            // Must match [+-]?\d*\.\d+
            let s = content.strip_prefix(['+', '-']).unwrap_or(content);
            if let Some(dot_pos) = s.find('.') {
                let before = &s[..dot_pos];
                let after = &s[dot_pos + 1..];
                if (before.is_empty() || before.chars().all(|c| c.is_ascii_digit()))
                    && !after.is_empty()
                    && after.chars().all(|c| c.is_ascii_digit())
                {
                    Some(content.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}
