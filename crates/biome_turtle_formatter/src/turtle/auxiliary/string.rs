use crate::prelude::*;
use biome_formatter::QuoteStyle;
use biome_formatter::prelude::syntax_token_cow_slice;
use biome_formatter::write;
use biome_turtle_syntax::TurtleString;
use biome_turtle_syntax::TurtleSyntaxKind;
use std::borrow::Cow;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleString;

impl FormatNodeRule<TurtleString> for FormatTurtleString {
    fn fmt_fields(&self, node: &TurtleString, f: &mut TurtleFormatter) -> FormatResult<()> {
        let token = node.value()?;
        let quote_style = f.options().quote_style();
        let original = token.text_trimmed();
        let kind = token.kind();

        // Determine current quote style and quote length (1 for regular, 3 for long)
        let (current_quote, quote_len) = match kind {
            TurtleSyntaxKind::TURTLE_STRING_LITERAL_QUOTE => (QuoteStyle::Double, 1),
            TurtleSyntaxKind::TURTLE_STRING_LITERAL_SINGLE_QUOTE => (QuoteStyle::Single, 1),
            TurtleSyntaxKind::TURTLE_STRING_LITERAL_LONG_QUOTE => (QuoteStyle::Double, 3),
            TurtleSyntaxKind::TURTLE_STRING_LITERAL_LONG_SINGLE_QUOTE => (QuoteStyle::Single, 3),
            _ => return write!(f, [token.format()]),
        };

        // Extract content without quotes
        let content = &original[quote_len..original.len() - quote_len];

        // Normalize escape sequences in content
        let content = normalize_escapes(content);

        // Demote triple-quoted strings to single-quoted when content has no newlines
        let output_quote_len = if quote_len == 3 && !content.contains('\n') && !content.contains('\r') {
            1
        } else {
            quote_len
        };

        // Determine preferred quote style
        let preferred_char = quote_style.as_char();
        let output_quote = if current_quote != quote_style {
            // Check if swapping would require escaping
            let can_swap = if output_quote_len == 3 {
                let triple: String = core::iter::repeat(preferred_char).take(3).collect();
                !content.contains(&*triple)
            } else {
                !content.contains(preferred_char)
            };
            if can_swap { quote_style } else { current_quote }
        } else {
            current_quote
        };

        let output_char = output_quote.as_char();
        let quote_str: String = core::iter::repeat(output_char).take(output_quote_len).collect();
        let new_text = std::format!("{quote_str}{content}{quote_str}");

        if new_text == original {
            return write!(f, [token.format()]);
        }

        write!(
            f,
            [format_replaced(
                &token,
                &syntax_token_cow_slice(
                    Cow::Owned(new_text),
                    &token,
                    token.text_trimmed_range().start(),
                ),
            )]
        )
    }
}

/// Normalize unicode escape sequences to their literal characters when possible.
///
/// Converts `\uXXXX` and `\UXXXXXXXX` to the corresponding character when it's
/// a printable, non-control character that doesn't need escaping.
fn normalize_escapes(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('u') => {
                    // \uXXXX — 4 hex digits
                    let hex: String = chars.by_ref().take(4).collect();
                    if hex.len() == 4 {
                        if let Ok(code) = u32::from_str_radix(&hex, 16) {
                            if let Some(c) = char::from_u32(code) {
                                if is_normalizable(c) {
                                    result.push(c);
                                    continue;
                                }
                            }
                        }
                    }
                    // Keep original if not normalizable
                    result.push('\\');
                    result.push('u');
                    result.push_str(&hex);
                }
                Some('U') => {
                    // \UXXXXXXXX — 8 hex digits
                    let hex: String = chars.by_ref().take(8).collect();
                    if hex.len() == 8 {
                        if let Ok(code) = u32::from_str_radix(&hex, 16) {
                            if let Some(c) = char::from_u32(code) {
                                if is_normalizable(c) {
                                    result.push(c);
                                    continue;
                                }
                            }
                        }
                    }
                    // Keep original if not normalizable
                    result.push('\\');
                    result.push('U');
                    result.push_str(&hex);
                }
                Some(other) => {
                    // Keep other escape sequences as-is (\n, \t, \r, \\, \", \')
                    result.push('\\');
                    result.push(other);
                }
                None => {
                    result.push('\\');
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Returns true if a character can be used directly instead of a unicode escape.
///
/// Characters that need escaping (quotes, backslash, control chars) should not
/// be normalized to their literal form.
fn is_normalizable(c: char) -> bool {
    // Don't normalize control characters, quotes, or backslash
    !c.is_control() && c != '"' && c != '\'' && c != '\\'
}
