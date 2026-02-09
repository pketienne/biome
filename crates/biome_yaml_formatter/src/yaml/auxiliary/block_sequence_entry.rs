use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::{AnyYamlBlockNode, YamlBlockSequenceEntry};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlBlockSequenceEntry;

impl FormatNodeRule<YamlBlockSequenceEntry> for FormatYamlBlockSequenceEntry {
    fn fmt_fields(
        &self,
        node: &YamlBlockSequenceEntry,
        f: &mut YamlFormatter,
    ) -> FormatResult<()> {
        write!(f, [node.minus_token()?.format()])?;

        if let Some(value) = node.value() {
            match &value {
                AnyYamlBlockNode::AnyYamlBlockInBlockNode(_) => {
                    write!(f, [hard_line_break(), block_indent(&value.format())])?;
                }
                _ => {
                    write!(f, [space(), value.format()])?;
                }
            }
        }

        Ok(())
    }
}
