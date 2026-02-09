use crate::prelude::*;
use biome_formatter::write;
use biome_markdown_syntax::MdSoftBreak;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdSoftBreak;

impl FormatNodeRule<MdSoftBreak> for FormatMdSoftBreak {
    fn fmt_fields(&self, node: &MdSoftBreak, f: &mut MarkdownFormatter) -> FormatResult<()> {
        write!(f, [node.value_token().format()])
    }
}
