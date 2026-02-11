use crate::context::YamlFormatContext;
use biome_formatter::format_element::tag::VerbatimKind;
use biome_formatter::formatter::Formatter;
use biome_formatter::prelude::{Tag, text};
use biome_formatter::trivia::{FormatLeadingComments, FormatTrailingComments};
use biome_formatter::{
    Buffer, CstFormatContext, Format, FormatContext, FormatElement, FormatError, FormatResult,
    FormatWithRule, LINE_TERMINATORS, normalize_newlines,
};
use biome_yaml_syntax::{YamlLanguage, YamlSyntaxNode};
use biome_rowan::{AstNode, Direction, SyntaxElement, TextRange};

/// "Formats" a node according to its original formatting in the source text.
pub fn format_yaml_verbatim_node(node: &YamlSyntaxNode) -> FormatYamlVerbatimNode<'_> {
    FormatYamlVerbatimNode {
        node,
        kind: VerbatimKind::Verbatim {
            length: node.text_range_with_trivia().len(),
        },
        format_comments: true,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct FormatYamlVerbatimNode<'node> {
    node: &'node YamlSyntaxNode,
    kind: VerbatimKind,
    format_comments: bool,
}

impl Format<YamlFormatContext> for FormatYamlVerbatimNode<'_> {
    fn fmt(&self, f: &mut Formatter<YamlFormatContext>) -> FormatResult<()> {
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

/// Formats bogus nodes.
pub fn format_bogus_node(node: &YamlSyntaxNode) -> FormatYamlVerbatimNode<'_> {
    FormatYamlVerbatimNode {
        node,
        kind: VerbatimKind::Bogus,
        format_comments: true,
    }
}

/// Format a node having formatter suppression comment applied to it
pub fn format_suppressed_node(node: &YamlSyntaxNode) -> FormatYamlVerbatimNode<'_> {
    FormatYamlVerbatimNode {
        node,
        kind: VerbatimKind::Suppressed,
        format_comments: true,
    }
}

/// Formats an object using its [`Format`] implementation but falls back to printing the object as
/// it is in the source document if formatting it returns an [`FormatError::SyntaxError`].
pub const fn format_or_verbatim<F>(inner: F) -> FormatNodeOrVerbatim<F> {
    FormatNodeOrVerbatim { inner }
}

/// Formats a node or falls back to verbatim printing if formatting this node fails.
#[derive(Copy, Clone)]
pub struct FormatNodeOrVerbatim<F> {
    inner: F,
}

impl<F, Item> Format<YamlFormatContext> for FormatNodeOrVerbatim<F>
where
    F: FormatWithRule<YamlFormatContext, Item = Item>,
    Item: AstNode<Language = YamlLanguage>,
{
    fn fmt(&self, f: &mut Formatter<YamlFormatContext>) -> FormatResult<()> {
        let snapshot = Formatter::state_snapshot(f);

        match self.inner.fmt(f) {
            Ok(result) => Ok(result),

            Err(FormatError::SyntaxError) => {
                f.restore_state_snapshot(snapshot);
                format_suppressed_node(self.inner.item().syntax()).fmt(f)
            }
            Err(err) => Err(err),
        }
    }
}
