use crate::prelude::*;
use biome_yaml_syntax::YamlAliasNode;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlAliasNode;

impl FormatNodeRule<YamlAliasNode> for FormatYamlAliasNode {
    fn fmt_fields(&self, node: &YamlAliasNode, f: &mut YamlFormatter) -> FormatResult<()> {
        node.value_token()?.format().fmt(f)
    }
}
