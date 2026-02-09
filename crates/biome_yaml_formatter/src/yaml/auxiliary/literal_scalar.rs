use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlLiteralScalar;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlLiteralScalar;

impl FormatNodeRule<YamlLiteralScalar> for FormatYamlLiteralScalar {
    fn fmt_fields(&self, node: &YamlLiteralScalar, f: &mut YamlFormatter) -> FormatResult<()> {
        if let Some(properties) = node.properties() {
            write!(f, [properties.format(), space()])?;
        }

        write!(
            f,
            [
                node.bitwise_or_token()?.format(),
                node.headers().format(),
                node.content()?.format(),
            ]
        )
    }
}
