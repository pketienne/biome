use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlFlowMapImplicitEntry;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFlowMapImplicitEntry;

impl FormatNodeRule<YamlFlowMapImplicitEntry> for FormatYamlFlowMapImplicitEntry {
    fn fmt_fields(
        &self,
        node: &YamlFlowMapImplicitEntry,
        f: &mut YamlFormatter,
    ) -> FormatResult<()> {
        if let Some(key) = node.key() {
            write!(f, [key.format()])?;
        }

        if let Some(colon) = node.colon_token() {
            write!(f, [colon.format()])?;
            if let Some(value) = node.value() {
                write!(f, [space(), value.format()])?;
            }
        }

        Ok(())
    }
}
