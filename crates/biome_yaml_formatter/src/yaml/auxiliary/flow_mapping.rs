use crate::prelude::*;
use biome_formatter::{BestFitting, Expand, FormatContext, format_args, write};
use biome_yaml_syntax::YamlFlowMapping;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatYamlFlowMapping;

impl FormatNodeRule<YamlFlowMapping> for FormatYamlFlowMapping {
    fn fmt_fields(&self, node: &YamlFlowMapping, f: &mut YamlFormatter) -> FormatResult<()> {
        let entries = node.entries();
        let expand = f.context().options().expand();

        if entries.is_empty() {
            return write!(
                f,
                [
                    node.l_curly_token()?.format(),
                    node.r_curly_token()?.format(),
                ]
            );
        }

        match expand {
            Expand::Always => {
                // Convert to block mapping style: suppress braces, remove commas
                write!(f, [format_removed(&node.l_curly_token()?)])?;
                let block_entries = format_with(|f| {
                    for element in entries.elements() {
                        let node = element.node()?;
                        write!(f, [hard_line_break(), node.format()])?;
                        if let Some(separator) = element.trailing_separator()? {
                            write!(f, [format_removed(&separator)])?;
                        }
                    }
                    Ok(())
                });
                write!(f, [indent(&block_entries)])?;
                write!(f, [format_removed(&node.r_curly_token()?)])
            }
            Expand::Auto => {
                // Collect entry nodes so they live long enough for memoization
                let entry_nodes: Vec<_> = entries.iter().flatten().collect();
                let mut memo_entries: Vec<_> = entry_nodes
                    .iter()
                    .map(|e| e.format().memoized())
                    .collect();

                // Bind brace tokens to locals before memoizing
                let l_curly_token = node.l_curly_token()?;
                let r_curly_token = node.r_curly_token()?;
                let mut l_curly = l_curly_token.format().memoized();
                let mut r_curly = r_curly_token.format().memoized();

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
                l_curly.inspect(f)?;
                r_curly.inspect(f)?;

                write!(
                    f,
                    [best_fitting![
                        // Variant 1 (least expanded): compact flow {a: 1, b: 2}
                        format_with(|f| {
                            write!(f, [&l_curly, space()])?;
                            for (i, memo) in memo_entries.iter().enumerate() {
                                if i > 0 {
                                    write!(f, [token(","), space()])?;
                                }
                                write!(f, [memo])?;
                            }
                            write!(f, [space(), &r_curly])
                        }),
                        // Variant 2 (most expanded): block mapping style
                        format_with(|f| {
                            for memo in &memo_entries {
                                write!(f, [hard_line_break(), memo])?;
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
                        node.l_curly_token()?.format(),
                        group(&format_args![
                            indent(&format_args![
                                soft_line_break_or_space(),
                                entries.format(),
                            ]),
                            soft_line_break_or_space(),
                        ]),
                        node.r_curly_token()?.format(),
                    ]
                )
            }
        }
    }
}
