use crate::prelude::*;
use biome_yaml_syntax::YamlBlockHeaderList;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlBlockHeaderList;

impl FormatRule<YamlBlockHeaderList> for FormatYamlBlockHeaderList {
    type Context = YamlFormatContext;
    fn fmt(&self, node: &YamlBlockHeaderList, f: &mut YamlFormatter) -> FormatResult<()> {
        let mut join = f.join();
        for item in node {
            join.entry(&item.format());
        }
        join.finish()
    }
}
