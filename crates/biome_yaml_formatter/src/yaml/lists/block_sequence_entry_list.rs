use crate::prelude::*;
use biome_yaml_syntax::YamlBlockSequenceEntryList;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlBlockSequenceEntryList;

impl FormatRule<YamlBlockSequenceEntryList> for FormatYamlBlockSequenceEntryList {
    type Context = YamlFormatContext;
    fn fmt(&self, node: &YamlBlockSequenceEntryList, f: &mut YamlFormatter) -> FormatResult<()> {
        f.join_with(hard_line_break())
            .entries(node.iter().formatted())
            .finish()
    }
}
