use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlPropertiesAnchorFirst;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlPropertiesAnchorFirst;

impl FormatNodeRule<YamlPropertiesAnchorFirst> for FormatYamlPropertiesAnchorFirst {
    fn fmt_fields(
        &self,
        node: &YamlPropertiesAnchorFirst,
        f: &mut YamlFormatter,
    ) -> FormatResult<()> {
        write!(f, [node.anchor().format()])?;
        if let Some(tag) = node.tag() {
            write!(f, [space(), tag.format()])?;
        }
        Ok(())
    }
}
