use crate::FormatTurtleSyntaxToken;
use crate::prelude::TurtleFormatContext;
use biome_formatter::formatter::Formatter;
use biome_formatter::trivia::FormatToken;
use biome_formatter::{Format, FormatResult};
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
