use crate::prelude::*;
use crate::trivia::format_removed;
use biome_formatter::write;
use biome_markdown_syntax::MdDocument;
use biome_rowan::AstNode;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdDocument;
impl FormatNodeRule<MdDocument> for FormatMdDocument {
    fn fmt_fields(&self, node: &MdDocument, f: &mut MarkdownFormatter) -> FormatResult<()> {
        if let Some(bom) = node.bom_token() {
            write!(f, [bom.format()])?;
        }
        write!(f, [format_verbatim_node(node.value().syntax())])?;
        if let Ok(eof) = node.eof_token() {
            write!(f, [format_removed(&eof)])?;
        }
        write!(f, [hard_line_break()])
    }
}
