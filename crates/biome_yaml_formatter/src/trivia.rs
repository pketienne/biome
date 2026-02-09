use crate::prelude::YamlFormatContext;
use crate::{FormatYamlSyntaxToken, YamlFormatter};
use biome_formatter::formatter::Formatter;
use biome_formatter::trivia::FormatToken;
use biome_formatter::{Argument, Buffer, Format, FormatResult};
use biome_yaml_syntax::YamlSyntaxToken;

/// Formats a zero-width synthetic token (e.g. MAPPING_START, SEQUENCE_START,
/// FLOW_START) by skipping it entirely. These tokens exist in the CST but have
/// no text content, so we just need to ensure the token tracker doesn't panic
/// when it encounters them.
///
/// For zero-width tokens that share an offset with a real token, we temporarily
/// disable the printed-tokens tracker to avoid offset collisions.
pub(crate) struct FormatSyntheticToken<'a> {
    token: &'a YamlSyntaxToken,
}

pub(crate) fn format_synthetic_token(token: &YamlSyntaxToken) -> FormatSyntheticToken<'_> {
    FormatSyntheticToken { token }
}

impl<'a> Format<YamlFormatContext> for FormatSyntheticToken<'a> {
    fn fmt(&self, f: &mut Formatter<YamlFormatContext>) -> FormatResult<()> {
        let was_disabled = f.state().is_token_tracking_disabled();
        f.state_mut().set_token_tracking_disabled(true);
        FormatYamlSyntaxToken.format_removed(self.token, f)?;
        f.state_mut().set_token_tracking_disabled(was_disabled);
        Ok(())
    }
}

pub(crate) struct FormatRemoved<'a> {
    token: &'a YamlSyntaxToken,
}

pub(crate) fn format_removed(token: &YamlSyntaxToken) -> FormatRemoved<'_> {
    FormatRemoved { token }
}

impl<'a> Format<YamlFormatContext> for FormatRemoved<'a> {
    fn fmt(&self, f: &mut Formatter<YamlFormatContext>) -> FormatResult<()> {
        FormatYamlSyntaxToken.format_removed(self.token, f)
    }
}

#[allow(dead_code)]
pub(crate) struct FormatReplaced<'a> {
    token: &'a YamlSyntaxToken,
    content: Argument<'a, YamlFormatContext>,
}

#[allow(dead_code)]
pub(crate) fn format_replaced<'a>(
    token: &'a YamlSyntaxToken,
    content: &'a impl Format<YamlFormatContext>,
) -> FormatReplaced<'a> {
    FormatReplaced {
        token,
        content: Argument::new(content),
    }
}

impl<'a> Format<YamlFormatContext> for FormatReplaced<'a> {
    fn fmt(&self, f: &mut Formatter<YamlFormatContext>) -> FormatResult<()> {
        FormatYamlSyntaxToken.format_replaced(self.token, &self.content, f)
    }
}

pub(crate) fn on_skipped(token: &YamlSyntaxToken, f: &mut YamlFormatter) -> FormatResult<()> {
    FormatYamlSyntaxToken.format_skipped_token_trivia(token, f)
}

pub(crate) fn on_removed(token: &YamlSyntaxToken, f: &mut YamlFormatter) -> FormatResult<()> {
    FormatYamlSyntaxToken.format_removed(token, f)
}
