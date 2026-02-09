use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlFlowMapping;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFlowMapping;

impl FormatNodeRule<YamlFlowMapping> for FormatYamlFlowMapping {
    fn fmt_fields(&self, node: &YamlFlowMapping, f: &mut YamlFormatter) -> FormatResult<()> {
        write!(
            f,
            [
                node.l_curly_token()?.format(),
                soft_block_indent(&node.entries().format()),
                node.r_curly_token()?.format(),
            ]
        )
    }
}
