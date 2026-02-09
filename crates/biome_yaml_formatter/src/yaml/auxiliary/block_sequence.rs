use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlBlockSequence;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlBlockSequence;

impl FormatNodeRule<YamlBlockSequence> for FormatYamlBlockSequence {
    fn fmt_fields(&self, node: &YamlBlockSequence, f: &mut YamlFormatter) -> FormatResult<()> {
        write!(f, [format_synthetic_token(&node.sequence_start_token()?)])?;

        if let Some(properties) = node.properties() {
            write!(f, [properties.format(), hard_line_break()])?;
        }

        write!(f, [node.entries().format()])?;

        write!(f, [format_synthetic_token(&node.sequence_end_token()?)])
    }
}
