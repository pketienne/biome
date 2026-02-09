use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlBlockMapping;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlBlockMapping;

impl FormatNodeRule<YamlBlockMapping> for FormatYamlBlockMapping {
    fn fmt_fields(&self, node: &YamlBlockMapping, f: &mut YamlFormatter) -> FormatResult<()> {
        write!(f, [format_synthetic_token(&node.mapping_start_token()?)])?;

        if let Some(properties) = node.properties() {
            write!(f, [properties.format(), hard_line_break()])?;
        }

        write!(f, [node.entries().format()])?;

        write!(f, [format_synthetic_token(&node.mapping_end_token()?)])
    }
}
