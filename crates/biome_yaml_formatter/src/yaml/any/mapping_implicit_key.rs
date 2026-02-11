use crate::prelude::*;
use biome_yaml_syntax::AnyYamlMappingImplicitKey;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatAnyYamlMappingImplicitKey;

impl FormatRule<AnyYamlMappingImplicitKey> for FormatAnyYamlMappingImplicitKey {
    type Context = YamlFormatContext;
    fn fmt(&self, node: &AnyYamlMappingImplicitKey, f: &mut YamlFormatter) -> FormatResult<()> {
        match node {
            AnyYamlMappingImplicitKey::YamlFlowJsonNode(node) => node.format().fmt(f),
            AnyYamlMappingImplicitKey::YamlFlowYamlNode(node) => node.format().fmt(f),
        }
    }
}
