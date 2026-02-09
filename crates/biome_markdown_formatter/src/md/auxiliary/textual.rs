use crate::prelude::*;
use biome_formatter::write;
use biome_markdown_syntax::MdTextual;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdTextual;

impl FormatNodeRule<MdTextual> for FormatMdTextual {
    fn fmt_fields(&self, node: &MdTextual, f: &mut MarkdownFormatter) -> FormatResult<()> {
        write!(f, [node.value_token().format()])
    }
}
