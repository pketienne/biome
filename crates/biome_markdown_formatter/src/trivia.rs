use crate::prelude::MarkdownFormatContext;
use crate::{FormatMarkdownSyntaxToken, MarkdownFormatter};
use biome_formatter::formatter::Formatter;
use biome_formatter::prelude::syntax_token_cow_slice;
use biome_formatter::trivia::FormatToken;
use biome_formatter::{Argument, Format, FormatResult};
use biome_markdown_syntax::MarkdownSyntaxToken;

pub(crate) struct FormatRemoved<'a> {
    token: &'a MarkdownSyntaxToken,
}

pub(crate) fn format_removed(token: &MarkdownSyntaxToken) -> FormatRemoved<'_> {
    FormatRemoved { token }
}

impl<'a> Format<MarkdownFormatContext> for FormatRemoved<'a> {
    fn fmt(&self, f: &mut Formatter<MarkdownFormatContext>) -> FormatResult<()> {
        FormatMarkdownSyntaxToken.format_removed(self.token, f)
    }
}

pub(crate) struct FormatReplaced<'a> {
    token: &'a MarkdownSyntaxToken,
    content: Argument<'a, MarkdownFormatContext>,
}

pub(crate) fn format_replaced<'a>(
    token: &'a MarkdownSyntaxToken,
    content: &'a impl Format<MarkdownFormatContext>,
) -> FormatReplaced<'a> {
    FormatReplaced {
        token,
        content: Argument::new(content),
    }
}

impl<'a> Format<MarkdownFormatContext> for FormatReplaced<'a> {
    fn fmt(&self, f: &mut Formatter<MarkdownFormatContext>) -> FormatResult<()> {
        FormatMarkdownSyntaxToken.format_replaced(self.token, &self.content, f)
    }
}

pub(crate) fn on_skipped(
    token: &MarkdownSyntaxToken,
    f: &mut MarkdownFormatter,
) -> FormatResult<()> {
    FormatMarkdownSyntaxToken.format_skipped_token_trivia(token, f)
}

pub(crate) fn on_removed(
    token: &MarkdownSyntaxToken,
    f: &mut MarkdownFormatter,
) -> FormatResult<()> {
    FormatMarkdownSyntaxToken.format_removed(token, f)
}
