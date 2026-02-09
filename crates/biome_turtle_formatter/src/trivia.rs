use crate::FormatTurtleSyntaxToken;
use crate::prelude::TurtleFormatContext;
use biome_formatter::formatter::Formatter;
use biome_formatter::trivia::FormatToken;
use biome_formatter::{Argument, Format, FormatResult};
use biome_turtle_syntax::TurtleSyntaxToken;

pub(crate) struct FormatRemoved<'a> {
    token: &'a TurtleSyntaxToken,
}

pub(crate) fn format_removed(token: &TurtleSyntaxToken) -> FormatRemoved<'_> {
    FormatRemoved { token }
}

impl<'a> Format<TurtleFormatContext> for FormatRemoved<'a> {
    fn fmt(&self, f: &mut Formatter<TurtleFormatContext>) -> FormatResult<()> {
        FormatTurtleSyntaxToken.format_removed(self.token, f)
    }
}

pub(crate) struct FormatReplaced<'a> {
    token: &'a TurtleSyntaxToken,
    content: Argument<'a, TurtleFormatContext>,
}

pub(crate) fn format_replaced<'a>(
    token: &'a TurtleSyntaxToken,
    content: &'a impl Format<TurtleFormatContext>,
) -> FormatReplaced<'a> {
    FormatReplaced {
        token,
        content: Argument::new(content),
    }
}

impl<'a> Format<TurtleFormatContext> for FormatReplaced<'a> {
    fn fmt(&self, f: &mut Formatter<TurtleFormatContext>) -> FormatResult<()> {
        FormatTurtleSyntaxToken.format_replaced(self.token, &self.content, f)
    }
}
