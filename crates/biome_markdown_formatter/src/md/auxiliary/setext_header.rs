use crate::prelude::*;
use biome_formatter::{CstFormatContext, write};
use biome_markdown_syntax::MdSetextHeader;
use biome_rowan::{AstNode, Direction, SyntaxElement};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdSetextHeader;

impl FormatNodeRule<MdSetextHeader> for FormatMdSetextHeader {
    fn fmt_fields(&self, node: &MdSetextHeader, f: &mut MarkdownFormatter) -> FormatResult<()> {
        // Track all tokens and mark comments
        for element in node.syntax().descendants_with_tokens(Direction::Next) {
            match element {
                SyntaxElement::Token(token) => f.state_mut().track_token(&token),
                SyntaxElement::Node(child) => {
                    f.context().comments().mark_suppression_checked(&child);
                }
            }
        }

        // Determine heading level from the underline character
        let underline = node.underline()?;
        let underline_text = underline.value_token()?.text_trimmed().to_string();
        let prefix = if underline_text.starts_with('=') {
            "#"
        } else {
            "##"
        };

        // Get the paragraph content text
        let paragraph = node.content()?;
        let heading_text = paragraph.syntax().text_trimmed().to_string();

        // Build ATX-style heading: "# Heading" or "## Heading"
        let atx = std::format!("{prefix} {heading_text}");
        write!(
            f,
            [text(&atx, node.syntax().text_trimmed_range().start())]
        )
    }
}
