use biome_diagnostics::category;
use biome_formatter::comments::{CommentKind, CommentStyle, Comments};
use biome_formatter::formatter::Formatter;
use biome_formatter::{FormatResult, FormatRule, write};
use biome_rowan::SyntaxTriviaPieceComments;
use biome_suppression::parse_suppression_comment;
use biome_turtle_syntax::TurtleLanguage;

use crate::prelude::*;

pub type TurtleComments = Comments<TurtleLanguage>;

#[derive(Default)]
pub struct FormatTurtleLeadingComment;

impl FormatRule<biome_formatter::comments::SourceComment<TurtleLanguage>>
    for FormatTurtleLeadingComment
{
    type Context = TurtleFormatContext;

    fn fmt(
        &self,
        comment: &biome_formatter::comments::SourceComment<TurtleLanguage>,
        f: &mut Formatter<Self::Context>,
    ) -> FormatResult<()> {
        // Turtle only has line comments (# ...), so just write the piece directly
        write!(f, [comment.piece().as_piece()])
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Default)]
pub struct TurtleCommentStyle;

impl CommentStyle for TurtleCommentStyle {
    type Language = TurtleLanguage;

    fn is_suppression(text: &str) -> bool {
        parse_suppression_comment(text)
            .filter_map(Result::ok)
            .flat_map(|suppression| suppression.categories)
            .any(|(key, ..)| key == category!("format"))
    }

    fn get_comment_kind(_comment: &SyntaxTriviaPieceComments<Self::Language>) -> CommentKind {
        // Turtle only has line comments (# ...)
        CommentKind::Line
    }
}
