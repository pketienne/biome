use crate::prelude::*;
use biome_formatter::{Expand, FormatContext, format_args, write};
use biome_yaml_syntax::YamlFlowMapping;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFlowMapping;

impl FormatNodeRule<YamlFlowMapping> for FormatYamlFlowMapping {
    fn fmt_fields(&self, node: &YamlFlowMapping, f: &mut YamlFormatter) -> FormatResult<()> {
        let entries = node.entries();
        let should_expand = f.context().options().expand() == Expand::Always;

        if entries.is_empty() {
            write!(
                f,
                [
                    node.l_curly_token()?.format(),
                    node.r_curly_token()?.format(),
                ]
            )
        } else {
            write!(
                f,
                [
                    node.l_curly_token()?.format(),
                    group(&format_args![
                        indent(&format_args![
                            soft_line_break_or_space(),
                            entries.format(),
                        ]),
                        soft_line_break_or_space(),
                    ])
                    .should_expand(should_expand),
                    node.r_curly_token()?.format(),
                ]
            )
        }
    }
}
