use crate::prelude::*;
use biome_markdown_syntax::MdDirectiveAttributeList;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdDirectiveAttributeList;

impl FormatRule<MdDirectiveAttributeList> for FormatMdDirectiveAttributeList {
    type Context = MarkdownFormatContext;
    fn fmt(
        &self,
        node: &MdDirectiveAttributeList,
        f: &mut MarkdownFormatter,
    ) -> FormatResult<()> {
        f.join().entries(node.iter().formatted()).finish()
    }
}
