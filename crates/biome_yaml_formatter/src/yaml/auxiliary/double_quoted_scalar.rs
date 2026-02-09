use crate::prelude::*;
use biome_formatter::QuoteStyle;
use biome_yaml_syntax::YamlDoubleQuotedScalar;
use std::borrow::Cow;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlDoubleQuotedScalar;

impl FormatNodeRule<YamlDoubleQuotedScalar> for FormatYamlDoubleQuotedScalar {
    fn fmt_fields(
        &self,
        node: &YamlDoubleQuotedScalar,
        f: &mut YamlFormatter,
    ) -> FormatResult<()> {
        let token = node.value_token()?;

        if f.options().quote_style() == QuoteStyle::Single {
            let text = token.text_trimmed();
            // Only convert if the content has no single quotes and no escape sequences
            // (single-quoted strings don't support backslash escapes)
            if text.len() >= 2 {
                let content = &text[1..text.len() - 1];
                if !content.contains('\'') && !content.contains('\\') {
                    let replaced = std::format!("'{content}'");
                    return format_replaced(
                        &token,
                        &syntax_token_cow_slice(
                            Cow::Owned(replaced),
                            &token,
                            token.text_trimmed_range().start(),
                        ),
                    )
                    .fmt(f);
                }
            }
        }

        token.format().fmt(f)
    }
}
