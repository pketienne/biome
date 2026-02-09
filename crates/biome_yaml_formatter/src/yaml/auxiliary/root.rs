use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlRoot;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlRoot;

impl FormatNodeRule<YamlRoot> for FormatYamlRoot {
    fn fmt_fields(&self, node: &YamlRoot, f: &mut YamlFormatter) -> FormatResult<()> {
        write!(
            f,
            [
                node.documents().format(),
                format_removed(&node.eof_token()?),
                hard_line_break(),
            ]
        )
    }
}
