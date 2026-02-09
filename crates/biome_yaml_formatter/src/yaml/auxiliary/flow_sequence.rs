use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlFlowSequence;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFlowSequence;

impl FormatNodeRule<YamlFlowSequence> for FormatYamlFlowSequence {
    fn fmt_fields(&self, node: &YamlFlowSequence, f: &mut YamlFormatter) -> FormatResult<()> {
        write!(
            f,
            [
                node.l_brack_token()?.format(),
                node.entries().format(),
                node.r_brack_token()?.format(),
            ]
        )
    }
}
