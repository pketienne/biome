use crate::prelude::*;
use biome_yaml_syntax::YamlSingleQuotedScalar;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlSingleQuotedScalar;

impl FormatNodeRule<YamlSingleQuotedScalar> for FormatYamlSingleQuotedScalar {
    fn fmt_fields(&self, node: &YamlSingleQuotedScalar, f: &mut YamlFormatter) -> FormatResult<()> {
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
