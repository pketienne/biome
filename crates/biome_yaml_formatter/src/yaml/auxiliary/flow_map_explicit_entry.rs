use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlFlowMapExplicitEntry;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFlowMapExplicitEntry;

impl FormatNodeRule<YamlFlowMapExplicitEntry> for FormatYamlFlowMapExplicitEntry {
    fn fmt_fields(
        &self,
        node: &YamlFlowMapExplicitEntry,
        f: &mut YamlFormatter,
    ) -> FormatResult<()> {
        write!(f, [node.question_mark_token()?.format()])?;

        if let Some(key) = node.key() {
            write!(f, [space(), key.format()])?;
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
