use crate::prelude::*;
use biome_yaml_syntax::YamlAnchorProperty;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlAnchorProperty;

impl FormatNodeRule<YamlAnchorProperty> for FormatYamlAnchorProperty {
    fn fmt_fields(&self, node: &YamlAnchorProperty, f: &mut YamlFormatter) -> FormatResult<()> {
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
