use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlBlockMapExplicitEntry;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlBlockMapExplicitEntry;

impl FormatNodeRule<YamlBlockMapExplicitEntry> for FormatYamlBlockMapExplicitEntry {
    fn fmt_fields(
        &self,
        node: &YamlBlockMapExplicitEntry,
        f: &mut YamlFormatter,
    ) -> FormatResult<()> {
        write!(f, [node.question_mark_token()?.format()])?;

        if let Some(key) = node.key() {
            write!(f, [space(), key.format()])?;
        }

        if let Some(colon) = node.colon_token() {
            write!(f, [hard_line_break(), colon.format()])?;
            if let Some(value) = node.value() {
                write!(f, [space(), value.format()])?;
            }
        }

        Ok(())
    }
}
