use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlDocumentList;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlDocumentList;

impl FormatRule<YamlDocumentList> for FormatYamlDocumentList {
    type Context = YamlFormatContext;
    fn fmt(&self, node: &YamlDocumentList, f: &mut YamlFormatter) -> FormatResult<()> {
        let separator = hard_line_break();
        let mut join = f.join_with(&separator);
        for item in node {
            join.entry(&format_with(|f| write!(f, [item.format()])));
        }
        join.finish()
    }
}
