use crate::prelude::*;
use biome_yaml_syntax::YamlPlainScalar;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlPlainScalar;

impl FormatNodeRule<YamlPlainScalar> for FormatYamlPlainScalar {
    fn fmt_fields(&self, node: &YamlPlainScalar, f: &mut YamlFormatter) -> FormatResult<()> {
        node.value_token()?.format().fmt(f)
    }
}
