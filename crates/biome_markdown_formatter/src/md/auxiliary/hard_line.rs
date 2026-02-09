use crate::prelude::*;
use biome_formatter::write;
use biome_markdown_syntax::MdHardLine;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdHardLine;

impl FormatNodeRule<MdHardLine> for FormatMdHardLine {
    fn fmt_fields(&self, node: &MdHardLine, f: &mut MarkdownFormatter) -> FormatResult<()> {
        write!(f, [node.value_token().format()])
    }
}
