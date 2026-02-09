use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::{AnyYamlBlockNode, YamlBlockMapImplicitEntry};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlBlockMapImplicitEntry;

impl FormatNodeRule<YamlBlockMapImplicitEntry> for FormatYamlBlockMapImplicitEntry {
    fn fmt_fields(
        &self,
        node: &YamlBlockMapImplicitEntry,
        f: &mut YamlFormatter,
    ) -> FormatResult<()> {
        if let Some(key) = node.key() {
            write!(f, [key.format()])?;
        }

        let colon = node.colon_token()?;

        match node.value() {
            Some(AnyYamlBlockNode::YamlFlowInBlockNode(value)) => {
                write!(f, [colon.format(), space(), value.format()])
            }
            Some(value) => {
                write!(
                    f,
                    [colon.format(), hard_line_break(), block_indent(&value.format())]
                )
            }
            None => {
                write!(f, [colon.format()])
            }
        }
    }
}
