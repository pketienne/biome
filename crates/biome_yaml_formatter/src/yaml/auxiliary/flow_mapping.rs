use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlFlowMapping;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFlowMapping;

impl FormatNodeRule<YamlFlowMapping> for FormatYamlFlowMapping {
    fn fmt_fields(&self, node: &YamlFlowMapping, f: &mut YamlFormatter) -> FormatResult<()> {
        let entries = node.entries();
        write!(f, [node.l_curly_token()?.format()])?;
        if !entries.is_empty() {
            write!(f, [space(), entries.format(), space()])?;
        }
        write!(f, [node.r_curly_token()?.format()])
    }
}
