use crate::prelude::*;
use biome_markdown_syntax::MdTableRowList;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdTableRowList;
impl FormatRule<MdTableRowList> for FormatMdTableRowList {
    type Context = MarkdownFormatContext;
    fn fmt(&self, node: &MdTableRowList, f: &mut MarkdownFormatter) -> FormatResult<()> {
        f.join().entries(node.iter().formatted()).finish()
    }
}
