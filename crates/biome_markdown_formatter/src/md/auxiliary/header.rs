use crate::prelude::*;
use biome_formatter::write;
use biome_markdown_syntax::MdHeader;
use biome_rowan::AstNode;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdHeader;
impl FormatNodeRule<MdHeader> for FormatMdHeader {
    fn fmt_fields(&self, node: &MdHeader, f: &mut MarkdownFormatter) -> FormatResult<()> {
        // Format leading hashes (preserves trivia on first hash = blank lines before heading)
        write!(f, [node.before().format()])?;
        // Emit exactly one space between # and content
        if node.content().is_some() {
            write!(f, [space()])?;
        }
        // Format content verbatim if present
        if let Some(content) = node.content() {
            write!(f, [format_verbatim_node(content.syntax())])?;
        }
        // Skip trailing hashes (node.after()) — remove them
        Ok(())
    }
}
