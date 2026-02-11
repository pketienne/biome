use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::{
    AnyYamlBlockInBlockNode, AnyYamlBlockNode, YamlBlockSequenceEntry,
    YamlBlockSequenceEntryFields,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlBlockSequenceEntry;

impl FormatNodeRule<YamlBlockSequenceEntry> for FormatYamlBlockSequenceEntry {
    fn fmt_fields(
        &self,
        node: &YamlBlockSequenceEntry,
        f: &mut YamlFormatter,
    ) -> FormatResult<()> {
        let YamlBlockSequenceEntryFields { minus_token, value } = node.as_fields();

        let is_block_value = value.as_ref().is_some_and(|v| {
            matches!(
                v,
                AnyYamlBlockNode::AnyYamlBlockInBlockNode(
                    AnyYamlBlockInBlockNode::YamlBlockMapping(_)
                        | AnyYamlBlockInBlockNode::YamlBlockSequence(_)
                )
            )
        });

        if is_block_value {
            // In YAML compact notation, the first entry of a block mapping follows
            // the `-` on the same line, subsequent entries are indented to match.
            write!(f, [minus_token.format(), space(), indent(&value.format())])?;
        } else {
            write!(f, [minus_token.format(), space()])?;
            if let Some(value) = value {
                write!(f, [value.format()])?;
            }
        }
        Ok(())
    }
}
