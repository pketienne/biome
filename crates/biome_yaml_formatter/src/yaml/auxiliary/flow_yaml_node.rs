use crate::prelude::*;
use biome_yaml_syntax::YamlFlowYamlNode;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFlowYamlNode;

impl FormatNodeRule<YamlFlowYamlNode> for FormatYamlFlowYamlNode {
    fn fmt_fields(&self, node: &YamlFlowYamlNode, f: &mut YamlFormatter) -> FormatResult<()> {
        for slot in node.syntax().slots() {
            match slot {
                biome_rowan::SyntaxSlot::Node(node) => {
                    node.format().fmt(f)?;
                }
                biome_rowan::SyntaxSlot::Token(token) => {
                    token.format().fmt(f)?;
                }
                biome_rowan::SyntaxSlot::Empty { .. } => {}
            }
        }
        Ok(())
    }
}
