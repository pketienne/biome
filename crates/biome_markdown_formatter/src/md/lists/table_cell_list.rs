use crate::prelude::*;
use biome_markdown_syntax::MdTableCellList;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdTableCellList;
impl FormatRule<MdTableCellList> for FormatMdTableCellList {
    type Context = MarkdownFormatContext;
    fn fmt(&self, node: &MdTableCellList, f: &mut MarkdownFormatter) -> FormatResult<()> {
        f.join().entries(node.iter().formatted()).finish()
    }
}
