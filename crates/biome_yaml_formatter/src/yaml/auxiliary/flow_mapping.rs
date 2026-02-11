use crate::prelude::*;
use biome_yaml_syntax::YamlFlowMapping;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFlowMapping;

impl FormatNodeRule<YamlFlowMapping> for FormatYamlFlowMapping {
    fn fmt_fields(&self, node: &YamlFlowMapping, f: &mut YamlFormatter) -> FormatResult<()> {
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
