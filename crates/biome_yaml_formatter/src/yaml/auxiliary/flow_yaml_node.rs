use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlFlowYamlNode;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFlowYamlNode;

impl FormatNodeRule<YamlFlowYamlNode> for FormatYamlFlowYamlNode {
    fn fmt_fields(&self, node: &YamlFlowYamlNode, f: &mut YamlFormatter) -> FormatResult<()> {
        if let Some(properties) = node.properties() {
            write!(f, [properties.format(), space()])?;
        }

        if let Some(content) = node.content() {
            write!(f, [content.format()])?;
        }

        Ok(())
    }
}
