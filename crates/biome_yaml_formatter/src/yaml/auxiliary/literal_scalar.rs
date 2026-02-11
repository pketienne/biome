use crate::prelude::*;
use biome_yaml_syntax::YamlLiteralScalar;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlLiteralScalar;

impl FormatNodeRule<YamlLiteralScalar> for FormatYamlLiteralScalar {
    fn fmt_fields(&self, node: &YamlLiteralScalar, f: &mut YamlFormatter) -> FormatResult<()> {
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
