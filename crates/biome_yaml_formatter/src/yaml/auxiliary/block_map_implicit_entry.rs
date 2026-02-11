use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::{
    AnyYamlBlockInBlockNode, AnyYamlBlockNode, YamlBlockMapImplicitEntry,
    YamlBlockMapImplicitEntryFields,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlBlockMapImplicitEntry;

impl FormatNodeRule<YamlBlockMapImplicitEntry> for FormatYamlBlockMapImplicitEntry {
    fn fmt_fields(
        &self,
        node: &YamlBlockMapImplicitEntry,
        f: &mut YamlFormatter,
    ) -> FormatResult<()> {
        let YamlBlockMapImplicitEntryFields {
            key,
            colon_token,
            value,
        } = node.as_fields();

        if let Some(key) = key {
            write!(f, [key.format()])?;
        }

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
            write!(f, [colon_token.format()])?;
            write!(f, [block_indent(&value.format())])?;
        } else {
            write!(f, [colon_token.format(), space()])?;
            if let Some(value) = value {
                write!(f, [value.format()])?;
            }
        }
        Ok(())
    }
}
