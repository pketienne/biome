use crate::prelude::*;
use biome_formatter::{CstFormatContext, LINE_TERMINATORS, normalize_newlines, write};
use biome_markdown_syntax::MdFencedCodeBlock;
use biome_rowan::{AstNode, Direction, SyntaxElement};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdFencedCodeBlock;

impl FormatNodeRule<MdFencedCodeBlock> for FormatMdFencedCodeBlock {
    fn fmt_fields(
        &self,
        node: &MdFencedCodeBlock,
        f: &mut MarkdownFormatter,
    ) -> FormatResult<()> {
        // Track all tokens and mark comments as formatted
        for element in node.syntax().descendants_with_tokens(Direction::Next) {
            match element {
                SyntaxElement::Token(token) => f.state_mut().track_token(&token),
                SyntaxElement::Node(child) => {
                    f.context().comments().mark_suppression_checked(&child);
                }
            }
        }

        let l_fence = node.l_fence_token()?;
        let has_tildes = l_fence.text_trimmed().contains('~');

        // Get the trimmed text of the whole node (preserves internal newlines,
        // strips only leading trivia of first token and trailing trivia of last token)
        let node_text = node.syntax().text_trimmed().to_string();
        let normalized = normalize_newlines(&node_text, LINE_TERMINATORS);

        if has_tildes {
            // Replace all tildes with backticks
            let replaced = normalized.replace('~', "`");
            write!(
                f,
                [text(&replaced, node.syntax().text_trimmed_range().start())]
            )?;
        } else {
            write!(
                f,
                [text(&normalized, node.syntax().text_trimmed_range().start())]
            )?;
        }

        Ok(())
    }
}
