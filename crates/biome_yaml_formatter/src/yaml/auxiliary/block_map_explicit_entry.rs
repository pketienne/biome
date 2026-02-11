use crate::prelude::*;
use biome_yaml_syntax::YamlBlockMapExplicitEntry;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlBlockMapExplicitEntry;

impl FormatNodeRule<YamlBlockMapExplicitEntry> for FormatYamlBlockMapExplicitEntry {
    fn fmt_fields(&self, node: &YamlBlockMapExplicitEntry, f: &mut YamlFormatter) -> FormatResult<()> {
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
