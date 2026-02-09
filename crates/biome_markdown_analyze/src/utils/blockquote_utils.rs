use crate::utils::fence_utils::FenceTracker;

/// A contiguous blockquote block in the document.
#[derive(Debug, Clone)]
pub struct BlockquoteBlock {
    /// 0-based line index of the first line.
    pub start_line: usize,
    /// 0-based line index of the last line.
    pub end_line: usize,
    /// The lines of the blockquote (with `> ` prefix stripped).
    pub lines: Vec<BlockquoteLine>,
    /// Byte offset of the block start.
    pub byte_offset: usize,
    /// Byte length of the entire block.
    pub byte_len: usize,
}

/// A single line within a blockquote.
#[derive(Debug, Clone)]
pub struct BlockquoteLine {
    /// 0-based line index.
    pub line_index: usize,
    /// Number of leading spaces before `>`.
    pub indent: usize,
    /// Whether the line starts with `>`.
    pub has_marker: bool,
    /// Number of spaces after `>` (before content). 0 if no marker.
    pub spaces_after_marker: usize,
    /// The content text after stripping the `> ` prefix.
    pub content: String,
    /// Byte offset of this line.
    pub byte_offset: usize,
    /// Byte length of this line.
    pub byte_len: usize,
}

/// Parse a single line as a potential blockquote line.
///
/// Returns `Some` if the line starts with optional spaces followed by `>`.
pub fn parse_blockquote_line(
    line: &str,
    line_index: usize,
    byte_offset: usize,
) -> Option<BlockquoteLine> {
    let bytes = line.as_bytes();
    let indent = line
        .bytes()
        .take_while(|&b| b == b' ' || b == b'\t')
        .count();

    if indent >= bytes.len() || bytes[indent] != b'>' {
        return None;
    }

    // Count spaces after >
    let after_marker = indent + 1;
    let spaces = if after_marker < bytes.len() {
        bytes[after_marker..]
            .iter()
            .take_while(|&&b| b == b' ' || b == b'\t')
            .count()
    } else {
        0
    };

    let content_start = after_marker + spaces;
    let content = if content_start < line.len() {
        line[content_start..].to_string()
    } else {
        String::new()
    };

    Some(BlockquoteLine {
        line_index,
        indent,
        has_marker: true,
        spaces_after_marker: spaces,
        content,
        byte_offset,
        byte_len: line.len(),
    })
}

/// Collect all blockquote blocks from a document's text, skipping fenced code blocks.
pub fn collect_blockquote_blocks(text: &str) -> Vec<BlockquoteBlock> {
    let mut blocks = Vec::new();
    let mut current_lines: Vec<BlockquoteLine> = Vec::new();
    let mut tracker = FenceTracker::new();
    let mut offset = 0usize;

    let lines: Vec<&str> = text.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        tracker.process_line(line_idx, line);

        if tracker.is_inside_fence() {
            if !current_lines.is_empty() {
                finalize_block(&mut blocks, &mut current_lines);
            }
            offset += line.len() + 1;
            continue;
        }

        if let Some(bq_line) = parse_blockquote_line(line, line_idx, offset) {
            current_lines.push(bq_line);
        } else if !current_lines.is_empty() {
            // Check for lazy continuation (non-blank line without `>` marker
            // immediately after a blockquote line)
            if !line.trim().is_empty() {
                let prev_line_idx = current_lines.last().unwrap().line_index;
                if line_idx == prev_line_idx + 1 {
                    // Lazy continuation line
                    current_lines.push(BlockquoteLine {
                        line_index: line_idx,
                        indent: 0,
                        has_marker: false,
                        spaces_after_marker: 0,
                        content: line.to_string(),
                        byte_offset: offset,
                        byte_len: line.len(),
                    });
                } else {
                    finalize_block(&mut blocks, &mut current_lines);
                }
            } else {
                finalize_block(&mut blocks, &mut current_lines);
            }
        }

        offset += line.len() + 1;
    }

    if !current_lines.is_empty() {
        finalize_block(&mut blocks, &mut current_lines);
    }

    blocks
}

fn finalize_block(blocks: &mut Vec<BlockquoteBlock>, lines: &mut Vec<BlockquoteLine>) {
    if lines.is_empty() {
        return;
    }
    let start_line = lines[0].line_index;
    let end_line = lines.last().unwrap().line_index;
    let byte_offset = lines[0].byte_offset;
    let last = lines.last().unwrap();
    let byte_len = (last.byte_offset + last.byte_len) - byte_offset;
    blocks.push(BlockquoteBlock {
        start_line,
        end_line,
        lines: lines.clone(),
        byte_offset,
        byte_len,
    });
    lines.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_blockquote() {
        let line = "> hello world";
        let bq = parse_blockquote_line(line, 0, 0).unwrap();
        assert!(bq.has_marker);
        assert_eq!(bq.indent, 0);
        assert_eq!(bq.spaces_after_marker, 1);
        assert_eq!(bq.content, "hello world");
    }

    #[test]
    fn parse_indented_blockquote() {
        let line = "  > indented";
        let bq = parse_blockquote_line(line, 0, 0).unwrap();
        assert_eq!(bq.indent, 2);
        assert_eq!(bq.content, "indented");
    }

    #[test]
    fn parse_no_space_after_marker() {
        let line = ">no space";
        let bq = parse_blockquote_line(line, 0, 0).unwrap();
        assert_eq!(bq.spaces_after_marker, 0);
        assert_eq!(bq.content, "no space");
    }

    #[test]
    fn not_a_blockquote() {
        assert!(parse_blockquote_line("hello", 0, 0).is_none());
        assert!(parse_blockquote_line("", 0, 0).is_none());
    }

    #[test]
    fn collect_single_block() {
        let text = "> line one\n> line two\n";
        let blocks = collect_blockquote_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].lines.len(), 2);
    }

    #[test]
    fn collect_separate_blocks() {
        let text = "> first\n\ntext\n\n> second\n";
        let blocks = collect_blockquote_blocks(text);
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn skip_fenced_content() {
        let text = "```\n> not a blockquote\n```\n";
        let blocks = collect_blockquote_blocks(text);
        assert!(blocks.is_empty());
    }
}
