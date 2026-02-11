use crate::prelude::*;
use biome_yaml_syntax::YamlFlowMapEntryList;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFlowMapEntryList;

impl FormatRule<YamlFlowMapEntryList> for FormatYamlFlowMapEntryList {
    type Context = YamlFormatContext;
    fn fmt(&self, node: &YamlFlowMapEntryList, f: &mut YamlFormatter) -> FormatResult<()> {
        let mut join = f.join();
        for item in node {
            join.entry(&item.format());
        }
        join.finish()
    }
}
