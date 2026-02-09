use crate::prelude::*;
use biome_yaml_syntax::YamlBlockMapEntryList;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlBlockMapEntryList;

impl FormatRule<YamlBlockMapEntryList> for FormatYamlBlockMapEntryList {
    type Context = YamlFormatContext;
    fn fmt(&self, node: &YamlBlockMapEntryList, f: &mut YamlFormatter) -> FormatResult<()> {
        f.join_with(hard_line_break())
            .entries(node.iter().formatted())
            .finish()
    }
}
