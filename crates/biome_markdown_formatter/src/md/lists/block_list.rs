use crate::prelude::*;
use biome_markdown_syntax::MdBlockList;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdBlockList;
impl FormatRule<MdBlockList> for FormatMdBlockList {
    type Context = MarkdownFormatContext;
    fn fmt(&self, node: &MdBlockList, f: &mut MarkdownFormatter) -> FormatResult<()> {
        let mut join = f.join_nodes_with_hardline();
        for block in node.iter() {
            join.entry(block.syntax(), &block.format());
        }
        join.finish()
    }
}
