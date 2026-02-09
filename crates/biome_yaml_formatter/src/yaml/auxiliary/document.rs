use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::YamlDocument;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlDocument;

impl FormatNodeRule<YamlDocument> for FormatYamlDocument {
    fn fmt_fields(&self, node: &YamlDocument, f: &mut YamlFormatter) -> FormatResult<()> {
        let directives = node.directives();
        let has_directives = !directives.is_empty();

        if let Some(bom) = node.bom_token() {
            write!(f, [format_removed(&bom)])?;
        }

        write!(f, [directives.format()])?;

        if let Some(dashdashdash) = node.dashdashdash_token() {
            if has_directives {
                write!(f, [hard_line_break()])?;
            }
            write!(f, [dashdashdash.format(), hard_line_break()])?;
        }

        if let Some(body) = node.node() {
            write!(f, [body.format()])?;
        }

        if let Some(dotdotdot) = node.dotdotdot_token() {
            write!(f, [hard_line_break(), dotdotdot.format()])?;
        }

        Ok(())
    }
}
