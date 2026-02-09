use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlFlowInBlockNode;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFlowInBlockNode;

impl FormatNodeRule<YamlFlowInBlockNode> for FormatYamlFlowInBlockNode {
    fn fmt_fields(
        &self,
        node: &YamlFlowInBlockNode,
        f: &mut YamlFormatter,
    ) -> FormatResult<()> {
        if let Some(properties) = node.properties() {
            write!(f, [properties.format(), space()])?;
        }
        write!(
            f,
            [
                format_synthetic_token(&node.flow_start_token()?),
                node.flow()?.format(),
                format_synthetic_token(&node.flow_end_token()?),
            ]
        )
    }
}
