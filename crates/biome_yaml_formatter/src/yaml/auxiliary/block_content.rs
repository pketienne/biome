use crate::prelude::*;
use biome_yaml_syntax::YamlBlockContent;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlBlockContent;

impl FormatNodeRule<YamlBlockContent> for FormatYamlBlockContent {
    fn fmt_fields(&self, node: &YamlBlockContent, f: &mut YamlFormatter) -> FormatResult<()> {
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
