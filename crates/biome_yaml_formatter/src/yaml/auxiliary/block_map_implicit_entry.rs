use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::{AnyYamlBlockInBlockNode, AnyYamlBlockNode, YamlBlockMapImplicitEntry};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlBlockMapImplicitEntry;

impl FormatNodeRule<YamlBlockMapImplicitEntry> for FormatYamlBlockMapImplicitEntry {
    fn fmt_fields(
        &self,
        node: &YamlBlockMapImplicitEntry,
        f: &mut YamlFormatter,
    ) -> FormatResult<()> {
        if let Some(key) = node.key() {
            write!(f, [key.format()])?;
        }

        let colon = node.colon_token()?;

        match node.value() {
            Some(AnyYamlBlockNode::YamlFlowInBlockNode(value)) => {
                write!(f, [colon.format(), space(), value.format()])
            }
            Some(AnyYamlBlockNode::AnyYamlBlockInBlockNode(inner)) => match &inner {
                // Block scalars: indicator (|/>) stays on same line as colon.
                // The scalar's content token already contains newlines and indentation.
                AnyYamlBlockInBlockNode::YamlLiteralScalar(_)
                | AnyYamlBlockInBlockNode::YamlFoldedScalar(_) => {
                    write!(f, [colon.format(), space(), inner.format()])
                }
                // Block mapping: properties on same line as colon, entries indented below
                AnyYamlBlockInBlockNode::YamlBlockMapping(mapping) => {
                    f.comments().mark_suppression_checked(mapping.syntax());
                    write!(f, [colon.format()])?;
                    if let Some(properties) = mapping.properties() {
                        write!(f, [space(), properties.format()])?;
                    }
                    write!(
                        f,
                        [hard_line_break(), block_indent(&format_with(|f| {
                            write!(f, [format_synthetic_token(&mapping.mapping_start_token()?)])?;
                            write!(f, [mapping.entries().format()])?;
                            write!(f, [format_synthetic_token(&mapping.mapping_end_token()?)])
                        }))]
                    )
                }
                // Block sequence: same pattern as block mapping
                AnyYamlBlockInBlockNode::YamlBlockSequence(sequence) => {
                    f.comments().mark_suppression_checked(sequence.syntax());
                    write!(f, [colon.format()])?;
                    if let Some(properties) = sequence.properties() {
                        write!(f, [space(), properties.format()])?;
                    }
                    write!(
                        f,
                        [hard_line_break(), block_indent(&format_with(|f| {
                            write!(
                                f,
                                [format_synthetic_token(&sequence.sequence_start_token()?)]
                            )?;
                            write!(f, [sequence.entries().format()])?;
                            write!(
                                f,
                                [format_synthetic_token(&sequence.sequence_end_token()?)]
                            )
                        }))]
                    )
                }
            },
            Some(value) => {
                // YamlBogusBlockNode fallback
                write!(
                    f,
                    [colon.format(), hard_line_break(), block_indent(&value.format())]
                )
            }
            None => {
                write!(f, [colon.format()])
            }
        }
    }
}
