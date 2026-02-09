use crate::prelude::*;
use biome_yaml_syntax::YamlBlockContent;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlBlockContent;

impl FormatNodeRule<YamlBlockContent> for FormatYamlBlockContent {
    fn fmt_fields(&self, node: &YamlBlockContent, f: &mut YamlFormatter) -> FormatResult<()> {
        node.value_token()?.format().fmt(f)
    }
}
