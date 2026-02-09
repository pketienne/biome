use crate::prelude::*;
use biome_yaml_syntax::YamlDoubleQuotedScalar;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlDoubleQuotedScalar;

impl FormatNodeRule<YamlDoubleQuotedScalar> for FormatYamlDoubleQuotedScalar {
    fn fmt_fields(
        &self,
        node: &YamlDoubleQuotedScalar,
        f: &mut YamlFormatter,
    ) -> FormatResult<()> {
        node.value_token()?.format().fmt(f)
    }
}
