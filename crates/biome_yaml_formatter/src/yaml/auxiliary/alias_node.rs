use crate::prelude::*;
use biome_yaml_syntax::YamlAliasNode;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlAliasNode;

impl FormatNodeRule<YamlAliasNode> for FormatYamlAliasNode {
    fn fmt_fields(&self, node: &YamlAliasNode, f: &mut YamlFormatter) -> FormatResult<()> {
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
