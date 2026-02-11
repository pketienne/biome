use crate::prelude::*;
use biome_yaml_syntax::YamlDirective;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlDirective;

impl FormatNodeRule<YamlDirective> for FormatYamlDirective {
    fn fmt_fields(&self, node: &YamlDirective, f: &mut YamlFormatter) -> FormatResult<()> {
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
