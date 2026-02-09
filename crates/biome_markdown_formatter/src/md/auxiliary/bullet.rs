use crate::prelude::*;
use biome_formatter::{CstFormatContext, LINE_TERMINATORS, normalize_newlines, write};
use biome_markdown_syntax::MdBullet;
use biome_rowan::{AstNode, Direction, SyntaxElement};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdBullet;

impl FormatNodeRule<MdBullet> for FormatMdBullet {
    fn fmt_fields(&self, node: &MdBullet, f: &mut MarkdownFormatter) -> FormatResult<()> {
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
        write!(
            f,
            [text(&normalized, node.syntax().text_trimmed_range().start())]
        )
    }
}
