use crate::prelude::*;
use biome_formatter::write;
use biome_markdown_syntax::MdIndent;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdIndent;

impl FormatNodeRule<MdIndent> for FormatMdIndent {
    fn fmt_fields(&self, node: &MdIndent, f: &mut MarkdownFormatter) -> FormatResult<()> {
        write!(f, [node.value_token().format()])
    }
}
