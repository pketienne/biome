use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::{YamlDocument, YamlDocumentFields};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlDocument;

impl FormatNodeRule<YamlDocument> for FormatYamlDocument {
    fn fmt_fields(&self, node: &YamlDocument, f: &mut YamlFormatter) -> FormatResult<()> {
        let YamlDocumentFields {
            bom_token,
            directives,
            dashdashdash_token,
            node: body,
            dotdotdot_token,
        } = node.as_fields();

        if let Some(bom) = bom_token {
            write!(f, [bom.format()])?;
        }
        write!(f, [directives.format()])?;
        if let Some(dash) = dashdashdash_token {
            write!(f, [dash.format(), hard_line_break()])?;
        }
        if let Some(body) = body {
            write!(f, [body.format()])?;
        }
        if let Some(dots) = dotdotdot_token {
            write!(f, [dots.format()])?;
        }
        Ok(())
    }
}
