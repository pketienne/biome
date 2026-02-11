use crate::prelude::*;
use biome_yaml_syntax::YamlBlockStripIndicator;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlBlockStripIndicator;

impl FormatNodeRule<YamlBlockStripIndicator> for FormatYamlBlockStripIndicator {
    fn fmt_fields(&self, node: &YamlBlockStripIndicator, f: &mut YamlFormatter) -> FormatResult<()> {
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
