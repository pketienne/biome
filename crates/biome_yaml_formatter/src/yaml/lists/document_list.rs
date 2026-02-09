use crate::prelude::*;
use biome_yaml_syntax::YamlDocumentList;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlDocumentList;

impl FormatRule<YamlDocumentList> for FormatYamlDocumentList {
    type Context = YamlFormatContext;
    fn fmt(&self, node: &YamlDocumentList, f: &mut YamlFormatter) -> FormatResult<()> {
        f.join_with(hard_line_break())
            .entries(node.iter().formatted())
            .finish()
    }
}
