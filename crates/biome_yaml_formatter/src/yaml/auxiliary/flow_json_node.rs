use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlFlowJsonNode;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFlowJsonNode;

impl FormatNodeRule<YamlFlowJsonNode> for FormatYamlFlowJsonNode {
    fn fmt_fields(&self, node: &YamlFlowJsonNode, f: &mut YamlFormatter) -> FormatResult<()> {
        if let Some(properties) = node.properties() {
            write!(f, [properties.format(), space()])?;
        }

        if let Some(content) = node.content() {
            write!(f, [content.format()])?;
        }

        Ok(())
    }
}
