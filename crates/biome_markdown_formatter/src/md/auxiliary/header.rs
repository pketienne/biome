use crate::prelude::*;
use biome_formatter::write;
use biome_markdown_syntax::MdHeader;
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
        // Format content using the paragraph formatter (handles trailing whitespace, newline normalization)
        if let Some(content) = node.content() {
            write!(f, [content.format()])?;
        }
        // Skip trailing hashes (node.after()) — remove them
        Ok(())
    }
}
