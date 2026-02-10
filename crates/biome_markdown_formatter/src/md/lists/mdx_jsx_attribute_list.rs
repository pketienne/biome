use crate::prelude::*;
use biome_markdown_syntax::MdMdxJsxAttributeList;
use biome_rowan::AstNodeList;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdMdxJsxAttributeList;

impl FormatRule<MdMdxJsxAttributeList> for FormatMdMdxJsxAttributeList {
    type Context = MarkdownFormatContext;

    fn fmt(
        &self,
        node: &MdMdxJsxAttributeList,
        f: &mut MarkdownFormatter,
    ) -> FormatResult<()> {
        let mut join = f.join();
        join.entries(node.iter().formatted());
        join.finish()
    }
}
