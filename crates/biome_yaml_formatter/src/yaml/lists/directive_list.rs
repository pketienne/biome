use crate::prelude::*;
use biome_yaml_syntax::YamlDirectiveList;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlDirectiveList;

impl FormatRule<YamlDirectiveList> for FormatYamlDirectiveList {
    type Context = YamlFormatContext;
    fn fmt(&self, node: &YamlDirectiveList, f: &mut YamlFormatter) -> FormatResult<()> {
        let mut join = f.join();
        for item in node {
            join.entry(&item.format());
        }
        join.finish()
    }
}
