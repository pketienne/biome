use crate::prelude::*;
use biome_yaml_syntax::YamlFlowSequenceEntryList;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFlowSequenceEntryList;

impl FormatRule<YamlFlowSequenceEntryList> for FormatYamlFlowSequenceEntryList {
    type Context = YamlFormatContext;
    fn fmt(&self, node: &YamlFlowSequenceEntryList, f: &mut YamlFormatter) -> FormatResult<()> {
        let mut join = f.join();
        for item in node {
            join.entry(&item.format());
        }
        join.finish()
    }
}
