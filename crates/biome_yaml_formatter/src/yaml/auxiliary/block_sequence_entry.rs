use crate::prelude::*;
use biome_formatter::write;
use biome_yaml_syntax::{AnyYamlBlockInBlockNode, AnyYamlBlockNode, YamlBlockSequenceEntry};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlBlockSequenceEntry;

impl FormatNodeRule<YamlBlockSequenceEntry> for FormatYamlBlockSequenceEntry {
    fn fmt_fields(
        &self,
        node: &YamlBlockSequenceEntry,
        f: &mut YamlFormatter,
    ) -> FormatResult<()> {
        write!(f, [node.minus_token()?.format()])?;

        if let Some(value) = node.value() {
            match &value {
                AnyYamlBlockNode::AnyYamlBlockInBlockNode(inner) => match inner {
                    // Block scalars: indicator (|/>) stays on same line as dash
                    AnyYamlBlockInBlockNode::YamlLiteralScalar(_)
                    | AnyYamlBlockInBlockNode::YamlFoldedScalar(_) => {
                        write!(f, [space(), inner.format()])?;
                    }
                    // Block mapping: compact form (- key: value)
                    AnyYamlBlockInBlockNode::YamlBlockMapping(mapping) => {
                        f.comments().mark_suppression_checked(mapping.syntax());
                        write!(
                            f,
                            [format_synthetic_token(&mapping.mapping_start_token()?)]
                        )?;
                        if let Some(properties) = mapping.properties() {
                            write!(f, [space(), properties.format()])?;
                        }
                        write!(
                            f,
                            [align(2, &format_with(|f| {
                                write!(f, [space(), mapping.entries().format()])?;
                                write!(
                                    f,
                                    [format_synthetic_token(&mapping.mapping_end_token()?)]
                                )
                            }))]
                        )?;
                    }
                    // Block sequence: same pattern as block mapping
                    AnyYamlBlockInBlockNode::YamlBlockSequence(sequence) => {
                        f.comments().mark_suppression_checked(sequence.syntax());
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
                        )?;
                    }
                },
                _ => {
                    write!(f, [space(), value.format()])?;
                }
            }
        }

        Ok(())
    }
}
