use crate::prelude::*;
use biome_formatter::comments::{
    CommentKind, CommentPlacement, CommentStyle, Comments, SourceComment,
};
use biome_formatter::formatter::Formatter;
use biome_formatter::{FormatResult, FormatRule, write};
use biome_markdown_syntax::MarkdownLanguage;
use biome_rowan::SyntaxTriviaPieceComments;
use biome_suppression::parse_suppression_comment;

pub type MarkdownComments = Comments<MarkdownLanguage>;

#[derive(Default)]
pub struct FormatMarkdownLeadingComment;

impl FormatRule<SourceComment<MarkdownLanguage>> for FormatMarkdownLeadingComment {
    type Context = MarkdownFormatContext;

    fn fmt(
        &self,
        comment: &SourceComment<MarkdownLanguage>,
        f: &mut Formatter<Self::Context>,
    ) -> FormatResult<()> {
        write!(f, [comment.piece().as_piece()])
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Default)]
pub struct MarkdownCommentStyle;

impl CommentStyle for MarkdownCommentStyle {
    type Language = MarkdownLanguage;

    fn is_suppression(text: &str) -> bool {
        parse_suppression_comment(text)
            .filter_map(Result::ok)
            .flat_map(|suppression| suppression.categories)
            .any(|(key, ..)| key == biome_diagnostics::category!("format"))
    }

    fn get_comment_kind(_comment: &SyntaxTriviaPieceComments<Self::Language>) -> CommentKind {
        CommentKind::Line
    }

    fn place_comment(
        &self,
        comment: biome_formatter::comments::DecoratedComment<Self::Language>,
    ) -> CommentPlacement<Self::Language> {
        CommentPlacement::Default(comment)
    }
}
