use crate::prelude::*;
use biome_formatter::QuoteStyle;
use biome_yaml_syntax::YamlSingleQuotedScalar;
use std::borrow::Cow;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlSingleQuotedScalar;

impl FormatNodeRule<YamlSingleQuotedScalar> for FormatYamlSingleQuotedScalar {
    fn fmt_fields(
        &self,
        node: &YamlSingleQuotedScalar,
        f: &mut YamlFormatter,
    ) -> FormatResult<()> {
        let token = node.value_token()?;

        if f.options().quote_style() == QuoteStyle::Double {
            let text = token.text_trimmed();
            // Only convert if the content has no double quotes
            // (would need escaping in double-quoted strings)
            if text.len() >= 2 {
                let content = &text[1..text.len() - 1];
                if !content.contains('"') {
                    // Unescape single-quote escapes ('') → single quote
                    let unescaped = content.replace("''", "'");
                    let replaced = std::format!("\"{unescaped}\"");
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
