use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlPropertiesTagFirst;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlPropertiesTagFirst;

impl FormatNodeRule<YamlPropertiesTagFirst> for FormatYamlPropertiesTagFirst {
    fn fmt_fields(
        &self,
        node: &YamlPropertiesTagFirst,
        f: &mut YamlFormatter,
    ) -> FormatResult<()> {
        write!(f, [node.tag().format()])?;
        if let Some(anchor) = node.anchor() {
            write!(f, [space(), anchor.format()])?;
        }
        Ok(())
    }
}
