use crate::prelude::*;
use biome_formatter::{CstFormatContext, LINE_TERMINATORS, normalize_newlines, write};
use biome_markdown_syntax::MdParagraph;
use biome_rowan::{AstNode, Direction, SyntaxElement};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdParagraph;

impl FormatNodeRule<MdParagraph> for FormatMdParagraph {
    fn fmt_fields(&self, node: &MdParagraph, f: &mut MarkdownFormatter) -> FormatResult<()> {
        // Track all tokens and mark comments as formatted
        for element in node.syntax().descendants_with_tokens(Direction::Next) {
            match element {
                SyntaxElement::Token(token) => f.state_mut().track_token(&token),
                SyntaxElement::Node(child) => {
                    f.context().comments().mark_suppression_checked(&child);
                }
            }
        }

        let node_text = node.syntax().text_trimmed().to_string();
        let normalized = normalize_newlines(&node_text, LINE_TERMINATORS);
        let cleaned = strip_trailing_whitespace(&normalized);

        write!(
            f,
            [text(&cleaned, node.syntax().text_trimmed_range().start())]
        )?;

        Ok(())
    }
}

/// Strip trailing whitespace from each line in the text.
///
/// Markdown hard breaks (2 or more consecutive trailing spaces immediately
/// before a newline) are preserved as exactly 2 spaces. All other trailing
/// whitespace (spaces and tabs) is removed. Hard break detection only applies
/// to non-final lines (there must be a newline after the spaces).
fn strip_trailing_whitespace(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut lines = input.split('\n').peekable();

    while let Some(line) = lines.next() {
        let is_last_line = lines.peek().is_none();
        let trimmed = line.trim_end_matches(|c: char| c == ' ' || c == '\t');

        result.push_str(trimmed);

        if !is_last_line {
            // Count consecutive trailing spaces (only spaces, not tabs) for
            // hard break detection. A hard break requires 2+ spaces at the
            // very end of the line before the newline.
            let trailing_space_count = line.len() - line.trim_end_matches(' ').len();

            if trailing_space_count >= 2 {
                // Markdown hard break: preserve exactly 2 spaces
                result.push_str("  ");
            }

            result.push('\n');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::strip_trailing_whitespace;

    #[test]
    fn no_trailing_whitespace() {
        assert_eq!(strip_trailing_whitespace("hello"), "hello");
    }

    #[test]
    fn strips_trailing_spaces_last_line() {
        // Last line has no newline after, so trailing spaces are always stripped
        assert_eq!(strip_trailing_whitespace("hello   "), "hello");
    }

    #[test]
    fn strips_trailing_tabs_single_line() {
        assert_eq!(strip_trailing_whitespace("hello\t\t"), "hello");
    }

    #[test]
    fn strips_mixed_trailing_whitespace_last_line() {
        // Mixed spaces/tabs on last line (no newline after) are stripped
        assert_eq!(strip_trailing_whitespace("hello \t "), "hello");
    }

    #[test]
    fn strips_trailing_spaces_multiline() {
        // 3 trailing spaces before newline = hard break (preserved as 2 spaces)
        // 2 trailing spaces on last line (no newline) = stripped
        assert_eq!(
            strip_trailing_whitespace("first   \nsecond  "),
            "first  \nsecond"
        );
    }

    #[test]
    fn preserves_hard_break_two_spaces() {
        // 2 trailing spaces before newline = hard break
        assert_eq!(
            strip_trailing_whitespace("first  \nsecond"),
            "first  \nsecond"
        );
    }

    #[test]
    fn normalizes_hard_break_many_spaces() {
        // More than 2 trailing spaces normalized to exactly 2
        assert_eq!(
            strip_trailing_whitespace("first     \nsecond"),
            "first  \nsecond"
        );
    }

    #[test]
    fn does_not_treat_single_space_as_hard_break() {
        assert_eq!(
            strip_trailing_whitespace("first \nsecond"),
            "first\nsecond"
        );
    }

    #[test]
    fn does_not_treat_tabs_only_as_hard_break() {
        // Tabs alone do not count as a hard break
        assert_eq!(
            strip_trailing_whitespace("first\t\t\nsecond"),
            "first\nsecond"
        );
    }

    #[test]
    fn last_line_trailing_spaces_stripped() {
        // Last line trailing spaces (no newline after) are always stripped
        assert_eq!(strip_trailing_whitespace("first\nsecond   "), "first\nsecond");
    }

    #[test]
    fn preserves_internal_whitespace() {
        assert_eq!(
            strip_trailing_whitespace("hello   world"),
            "hello   world"
        );
    }

    #[test]
    fn empty_string() {
        assert_eq!(strip_trailing_whitespace(""), "");
    }

    #[test]
    fn only_whitespace_last_line() {
        // Whitespace-only on last line is stripped
        assert_eq!(strip_trailing_whitespace("   "), "");
    }

    #[test]
    fn multiple_lines_mixed() {
        // line one: 3 trailing spaces before \n = hard break (normalized to 2)
        // line two: 2 trailing spaces before \n = hard break (kept as 2)
        // line three: trailing tab before \n = not a hard break (stripped)
        // line four: last line, no trailing whitespace
        let input = "line one   \nline two  \nline three\t\nline four";
        let expected = "line one  \nline two  \nline three\nline four";
        assert_eq!(strip_trailing_whitespace(input), expected);
    }
}
