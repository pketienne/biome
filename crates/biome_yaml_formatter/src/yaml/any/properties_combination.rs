use crate::prelude::*;
use biome_yaml_syntax::AnyYamlPropertiesCombination;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatAnyYamlPropertiesCombination;

impl FormatRule<AnyYamlPropertiesCombination> for FormatAnyYamlPropertiesCombination {
    type Context = YamlFormatContext;
    fn fmt(&self, node: &AnyYamlPropertiesCombination, f: &mut YamlFormatter) -> FormatResult<()> {
        match node {
            AnyYamlPropertiesCombination::YamlPropertiesAnchorFirst(node) => node.format().fmt(f),
            AnyYamlPropertiesCombination::YamlPropertiesTagFirst(node) => node.format().fmt(f),
        }
    }
}
