use crate::prelude::*;
use crate::trivia::format_replaced;
use biome_formatter::write;
use biome_markdown_syntax::MdThematicBreakBlock;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdThematicBreakBlock;
impl FormatNodeRule<MdThematicBreakBlock> for FormatMdThematicBreakBlock {
    fn fmt_fields(
        &self,
        node: &MdThematicBreakBlock,
        f: &mut MarkdownFormatter,
    ) -> FormatResult<()> {
        let token = node.value_token()?;
        write!(f, [format_replaced(&token, &text("---", token.text_range().start()))])
    }
}
