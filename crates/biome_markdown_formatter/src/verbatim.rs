use crate::context::MarkdownFormatContext;
use biome_formatter::format_element::tag::VerbatimKind;
use biome_formatter::formatter::Formatter;
use biome_formatter::prelude::{Tag, text};
use biome_formatter::trivia::{FormatLeadingComments, FormatTrailingComments};
use biome_formatter::{
    Buffer, CstFormatContext, Format, FormatContext, FormatElement, FormatResult, LINE_TERMINATORS,
    normalize_newlines,
};
use biome_markdown_syntax::MarkdownSyntaxNode;
use biome_rowan::{AstNode, Direction, SyntaxElement, TextRange};

/// Alias used by generated formatter code.
pub fn format_verbatim_node(node: &MarkdownSyntaxNode) -> FormatMarkdownVerbatimNode<'_> {
    format_markdown_verbatim_node(node)
}

pub fn format_markdown_verbatim_node(node: &MarkdownSyntaxNode) -> FormatMarkdownVerbatimNode<'_> {
    FormatMarkdownVerbatimNode {
        node,
        kind: VerbatimKind::Verbatim {
            length: node.text_trimmed_range().len(),
        },
        format_comments: true,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct FormatMarkdownVerbatimNode<'node> {
    node: &'node MarkdownSyntaxNode,
    kind: VerbatimKind,
    format_comments: bool,
}

impl Format<MarkdownFormatContext> for FormatMarkdownVerbatimNode<'_> {
    fn fmt(&self, f: &mut Formatter<MarkdownFormatContext>) -> FormatResult<()> {
        for element in self.node.descendants_with_tokens(Direction::Next) {
            match element {
                SyntaxElement::Token(token) => f.state_mut().track_token(&token),
                SyntaxElement::Node(node) => {
                    let comments = f.context().comments();
                    comments.mark_suppression_checked(&node);

                    for comment in comments.leading_dangling_trailing_comments(&node) {
                        comment.mark_formatted();
                    }
                }
            }
        }

        let trimmed_source_range = f.context().source_map().map_or_else(
            || self.node.text_trimmed_range(),
            |source_map| source_map.trimmed_source_range(self.node),
        );

        f.write_element(FormatElement::Tag(Tag::StartVerbatim(self.kind)))?;

        fn source_range<Context>(f: &Formatter<Context>, range: TextRange) -> TextRange
        where
            Context: CstFormatContext,
        {
            f.context()
                .source_map()
                .map_or_else(|| range, |source_map| source_map.source_range(range))
        }

        if self.format_comments {
            let comments = f.context().comments().clone();
            let leading_comments = comments.leading_comments(self.node);

            let outside_trimmed_range = leading_comments.partition_point(|comment| {
                comment.piece().text_range().end() <= trimmed_source_range.start()
            });

            let (outside_trimmed_range, in_trimmed_range) =
                leading_comments.split_at(outside_trimmed_range);

            biome_formatter::write!(f, [FormatLeadingComments::Comments(outside_trimmed_range)])?;

            for comment in in_trimmed_range {
                comment.mark_formatted();
            }
        }

        let start_source = self
            .node
            .first_leading_trivia()
            .into_iter()
            .flat_map(|trivia| trivia.pieces())
            .filter(|trivia| trivia.is_skipped())
            .map(|trivia| source_range(f, trivia.text_range()).start())
            .take_while(|start| *start < trimmed_source_range.start())
            .next()
            .unwrap_or_else(|| trimmed_source_range.start());

        let original_source = f.context().source_map().map_or_else(
            || self.node.text_trimmed().to_string(),
            |source_map| {
                source_map
                    .source()
                    .text_slice(trimmed_source_range.cover_offset(start_source))
                    .to_string()
            },
        );

        text(
            &normalize_newlines(&original_source, LINE_TERMINATORS),
            self.node.text_trimmed_range().start(),
        )
        .fmt(f)?;

        for comment in f.context().comments().dangling_comments(self.node) {
            comment.mark_formatted();
        }

        if self.format_comments {
            let comments = f.context().comments().clone();

            let trailing_comments = comments.trailing_comments(self.node);

            let outside_trimmed_range_start = trailing_comments.partition_point(|comment| {
                source_range(f, comment.piece().text_range()).end() <= trimmed_source_range.end()
            });

            let (in_trimmed_range, outside_trimmed_range) =
                trailing_comments.split_at(outside_trimmed_range_start);

            for comment in in_trimmed_range {
                comment.mark_formatted();
            }

            biome_formatter::write!(f, [FormatTrailingComments::Comments(outside_trimmed_range)])?;
        }

        f.write_element(FormatElement::Tag(Tag::EndVerbatim))
    }
}

pub fn format_bogus_node(node: &MarkdownSyntaxNode) -> FormatMarkdownVerbatimNode<'_> {
    FormatMarkdownVerbatimNode {
        node,
        kind: VerbatimKind::Bogus,
        format_comments: true,
    }
}

pub fn format_suppressed_node(node: &MarkdownSyntaxNode) -> FormatMarkdownVerbatimNode<'_> {
    FormatMarkdownVerbatimNode {
        node,
        kind: VerbatimKind::Suppressed,
        format_comments: true,
    }
}
