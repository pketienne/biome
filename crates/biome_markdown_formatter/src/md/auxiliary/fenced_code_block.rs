use crate::prelude::*;
use crate::trivia::format_replaced;
use biome_formatter::write;
use biome_markdown_syntax::MdFencedCodeBlock;

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdFencedCodeBlock;

impl FormatNodeRule<MdFencedCodeBlock> for FormatMdFencedCodeBlock {
    fn fmt_fields(
        &self,
        node: &MdFencedCodeBlock,
        f: &mut MarkdownFormatter,
    ) -> FormatResult<()> {
        let l_fence = node.l_fence_token()?;
        let fence_len = l_fence.text_trimmed().len();

        // Opening fence: normalize tildes to backticks
        if l_fence.text_trimmed().contains('~') {
            let normalized = "`".repeat(fence_len);
            write!(
                f,
                [format_replaced(
                    &l_fence,
                    &text(&normalized, l_fence.text_range().start())
                )]
            )?;
        } else {
            write!(f, [l_fence.format()])?;
        }

        // Language identifier: verbatim
        write!(f, [format_verbatim_node(node.code_list().syntax())])?;

        // Content: verbatim
        write!(f, [format_verbatim_node(node.content().syntax())])?;

        // Closing fence: normalize to match opening style
        if let Ok(r_fence) = node.r_fence_token() {
            let r_len = r_fence.text_trimmed().len();
            let norm_len = r_len.max(fence_len);
            if r_fence.text_trimmed().contains('~') {
                let normalized = "`".repeat(norm_len);
                write!(
                    f,
                    [format_replaced(
                        &r_fence,
                        &text(&normalized, r_fence.text_range().start())
                    )]
                )?;
            } else {
                write!(f, [r_fence.format()])?;
            }
        }

        Ok(())
    }
}
