use crate::prelude::*;
use biome_formatter::{Expand, FormatContext, format_args, write};
use biome_yaml_syntax::YamlFlowMapping;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFlowMapping;

impl FormatNodeRule<YamlFlowMapping> for FormatYamlFlowMapping {
    fn fmt_fields(&self, node: &YamlFlowMapping, f: &mut YamlFormatter) -> FormatResult<()> {
        let entries = node.entries();
        let expand = f.context().options().expand();

        if entries.is_empty() {
            return write!(
                f,
                [
                    node.l_curly_token()?.format(),
                    node.r_curly_token()?.format(),
                ]
            );
        }

        match expand {
            Expand::Always => {
                // Convert to block mapping style: suppress braces, remove commas
                write!(f, [format_removed(&node.l_curly_token()?)])?;
                let block_entries = format_with(|f| {
                    for element in entries.elements() {
                        let node = element.node()?;
                        write!(f, [hard_line_break(), node.format()])?;
                        if let Some(separator) = element.trailing_separator()? {
                            write!(f, [format_removed(&separator)])?;
                        }
                    }
                    Ok(())
                });
                write!(f, [indent(&block_entries)])?;
                write!(f, [format_removed(&node.r_curly_token()?)])
            }
            Expand::Auto | Expand::Never => {
                // Flow style with group-based expansion
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
                        ]),
                        node.r_curly_token()?.format(),
                    ]
                )
            }
        }
    }
}
