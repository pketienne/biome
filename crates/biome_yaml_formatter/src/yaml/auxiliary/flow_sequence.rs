use crate::prelude::*;
use biome_formatter::{Expand, FormatContext, format_args, write};
use biome_yaml_syntax::YamlFlowSequence;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFlowSequence;

impl FormatNodeRule<YamlFlowSequence> for FormatYamlFlowSequence {
    fn fmt_fields(&self, node: &YamlFlowSequence, f: &mut YamlFormatter) -> FormatResult<()> {
        let entries = node.entries();
        let expand = f.context().options().expand();

        if entries.is_empty() {
            return write!(
                f,
                [
                    node.l_brack_token()?.format(),
                    node.r_brack_token()?.format(),
                ]
            );
        }

        match expand {
            Expand::Always => {
                // Convert to block sequence style: suppress brackets, remove commas
                write!(f, [format_removed(&node.l_brack_token()?)])?;
                let block_entries = format_with(|f| {
                    for element in entries.elements() {
                        let node = element.node()?;
                        write!(f, [hard_line_break(), token("- "), node.format()])?;
                        if let Some(separator) = element.trailing_separator()? {
                            write!(f, [format_removed(&separator)])?;
                        }
                    }
                    Ok(())
                });
                write!(f, [indent(&block_entries)])?;
                write!(f, [format_removed(&node.r_brack_token()?)])
            }
            Expand::Auto | Expand::Never => {
                // Flow style with group-based expansion
                write!(
                    f,
                    [
                        node.l_brack_token()?.format(),
                        group(&format_args![
                            indent(&format_args![
                                soft_line_break_or_space(),
                                entries.format(),
                            ]),
                            soft_line_break_or_space(),
                        ]),
                        node.r_brack_token()?.format(),
                    ]
                )
            }
        }
    }
}
