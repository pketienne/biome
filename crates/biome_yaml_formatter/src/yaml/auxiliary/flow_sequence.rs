use crate::prelude::*;
use biome_formatter::{BestFitting, Expand, FormatContext, format_args, write};
use biome_yaml_syntax::YamlFlowSequence;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFlowSequence;

impl FormatNodeRule<YamlFlowSequence> for FormatYamlFlowSequence {
    fn fmt_fields(&self, node: &YamlFlowSequence, f: &mut YamlFormatter) -> FormatResult<()> {
        let entries = node.entries();
        let expand = f.context().options().expand();

        if entries.is_empty() {
            return write!(
                f,
                [
                    node.l_brack_token()?.format(),
                    node.r_brack_token()?.format(),
                ]
            );
        }

        match expand {
            Expand::Always => {
                // Convert to block sequence style: suppress brackets, remove commas
                write!(f, [format_removed(&node.l_brack_token()?)])?;
                let block_entries = format_with(|f| {
                    for element in entries.elements() {
                        let node = element.node()?;
                        write!(f, [hard_line_break(), token("- "), node.format()])?;
                        if let Some(separator) = element.trailing_separator()? {
                            write!(f, [format_removed(&separator)])?;
                        }
                    }
                    Ok(())
                });
                write!(f, [indent(&block_entries)])?;
                write!(f, [format_removed(&node.r_brack_token()?)])
            }
            Expand::Auto => {
                // Collect entry nodes so they live long enough for memoization
                let entry_nodes: Vec<_> = entries.iter().flatten().collect();
                let mut memo_entries: Vec<_> = entry_nodes
                    .iter()
                    .map(|e| e.format().memoized())
                    .collect();

                // Bind bracket tokens to locals before memoizing
                let l_brack_token = node.l_brack_token()?;
                let r_brack_token = node.r_brack_token()?;
                let mut l_brack = l_brack_token.format().memoized();
                let mut r_brack = r_brack_token.format().memoized();

                // Remove all comma separators upfront (tracked once as removed)
                for element in entries.elements() {
                    if let Some(separator) = element.trailing_separator()? {
                        write!(f, [format_removed(&separator)])?;
                    }
                }

                // Inspect memoized entries to trigger interning before best_fitting!
                for memo in &mut memo_entries {
                    memo.inspect(f)?;
                }
                l_brack.inspect(f)?;
                r_brack.inspect(f)?;

                write!(
                    f,
                    [best_fitting![
                        // Variant 1 (least expanded): compact flow [a, b, c]
                        format_with(|f| {
                            write!(f, [&l_brack, space()])?;
                            for (i, memo) in memo_entries.iter().enumerate() {
                                if i > 0 {
                                    write!(f, [token(","), space()])?;
                                }
                                write!(f, [memo])?;
                            }
                            write!(f, [space(), &r_brack])
                        }),
                        // Variant 2 (most expanded): block sequence style
                        format_with(|f| {
                            for memo in &memo_entries {
                                write!(f, [hard_line_break(), token("- "), memo])?;
                            }
                            Ok(())
                        }),
                    ]]
                )
            }
            Expand::Never => {
                // Flow style with group-based expansion
                write!(
                    f,
                    [
                        node.l_brack_token()?.format(),
                        group(&format_args![
                            indent(&format_args![
                                soft_line_break_or_space(),
                                entries.format(),
                            ]),
                            soft_line_break_or_space(),
                        ]),
                        node.r_brack_token()?.format(),
                    ]
                )
            }
        }
    }
}
