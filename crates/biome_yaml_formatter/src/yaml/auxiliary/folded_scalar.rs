use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlFoldedScalar;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFoldedScalar;

impl FormatNodeRule<YamlFoldedScalar> for FormatYamlFoldedScalar {
    fn fmt_fields(&self, node: &YamlFoldedScalar, f: &mut YamlFormatter) -> FormatResult<()> {
        if let Some(properties) = node.properties() {
            write!(f, [properties.format(), space()])?;
        }

        write!(
            f,
            [
                node.r_angle_token()?.format(),
                node.headers().format(),
                node.content()?.format(),
            ]
        )
    }
}
