use crate::utils::fence_utils::FenceTracker;

/// A detected list item in the document.
#[derive(Debug, Clone)]
pub struct ListItem {
    /// 0-based line index.
    pub line_index: usize,
    /// The kind of list marker.
    pub marker_kind: ListMarkerKind,
    /// The raw marker string (e.g., "-", "*", "+", "1.", "2)").
    pub marker: String,
    /// Number of leading spaces/tabs before the marker.
    pub indent: usize,
    /// Number of spaces between marker and content.
    pub content_offset: usize,
    /// The content text after the marker.
    pub content: String,
    /// Byte offset of this line in the document.
    pub byte_offset: usize,
    /// Byte length of this line.
    pub byte_len: usize,
    /// If a checkbox item, the checkbox info.
    pub checkbox: Option<Checkbox>,
}

/// The kind of list marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListMarkerKind {
    /// `-` marker
    Dash,
    /// `*` marker
    Asterisk,
    /// `+` marker
    Plus,
    /// `1.` style ordered marker
    OrderedDot,
    /// `1)` style ordered marker
    OrderedParen,
}

impl ListMarkerKind {
    pub fn is_unordered(self) -> bool {
        matches!(self, Self::Dash | Self::Asterisk | Self::Plus)
    }

    pub fn is_ordered(self) -> bool {
        matches!(self, Self::OrderedDot | Self::OrderedParen)
    }

    pub fn marker_char(self) -> &'static str {
        match self {
            Self::Dash => "-",
            Self::Asterisk => "*",
            Self::Plus => "+",
            Self::OrderedDot => ".",
            Self::OrderedParen => ")",
        }
    }
}

/// Checkbox information for a list item.
#[derive(Debug, Clone)]
pub struct Checkbox {
    /// The character inside the brackets: ' ', 'x', 'X', etc.
    pub check_char: char,
    /// Number of spaces between `]` and content.
    pub content_spacing: usize,
}

/// A contiguous block of list items.
#[derive(Debug, Clone)]
pub struct ListBlock {
    /// The items in this list block.
    pub items: Vec<ListItem>,
    /// The first line index.
    pub start_line: usize,
    /// The last line index.
    pub end_line: usize,
    /// Byte offset of the block start.
    pub byte_offset: usize,
    /// Byte length of the entire block.
    pub byte_len: usize,
}

/// Try to parse a line as a list item.
///
/// Returns `None` if the line is not a list item.
pub fn parse_list_item(line: &str, line_index: usize, byte_offset: usize) -> Option<ListItem> {
    let bytes = line.as_bytes();

    // Count leading whitespace
    let indent = line
        .bytes()
        .take_while(|&b| b == b' ' || b == b'\t')
        .count();
    let mut i = indent;

    if i >= bytes.len() {
        return None;
    }

    // Check for unordered markers: -, *, +
    let (marker_kind, marker, marker_end) =
        if bytes[i] == b'-' || bytes[i] == b'*' || bytes[i] == b'+' {
            let kind = match bytes[i] {
                b'-' => ListMarkerKind::Dash,
                b'*' => ListMarkerKind::Asterisk,
                b'+' => ListMarkerKind::Plus,
                _ => unreachable!(),
            };
            let marker = (bytes[i] as char).to_string();
            (kind, marker, i + 1)
        } else if bytes[i].is_ascii_digit() {
            // Check for ordered markers: 1. or 1)
            let num_start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            if i >= bytes.len() {
                return None;
            }
            let (kind, delim_end) = if bytes[i] == b'.' {
                (ListMarkerKind::OrderedDot, i + 1)
            } else if bytes[i] == b')' {
                (ListMarkerKind::OrderedParen, i + 1)
            } else {
                return None;
            };
            let marker = line[num_start..delim_end].to_string();
            (kind, marker, delim_end)
        } else {
            return None;
        };

    // Must be followed by at least one space
    if marker_end >= bytes.len() || (bytes[marker_end] != b' ' && bytes[marker_end] != b'\t') {
        return None;
    }

    // Count spaces between marker and content
    let content_start_offset = bytes[marker_end..]
        .iter()
        .take_while(|&&b| b == b' ' || b == b'\t')
        .count();
    let content_start = marker_end + content_start_offset;
    let content = if content_start < line.len() {
        line[content_start..].to_string()
    } else {
        String::new()
    };

    // Check for checkbox: [x], [ ], [X]
    let checkbox =
        if content.len() >= 3 && content.as_bytes()[0] == b'[' && content.as_bytes()[2] == b']' {
            let check_char = content.as_bytes()[1] as char;
            if check_char == ' ' || check_char == 'x' || check_char == 'X' {
                let after_bracket = &content[3..];
                let checkbox_content_spacing = after_bracket
                    .bytes()
                    .take_while(|&b| b == b' ' || b == b'\t')
                    .count();
                Some(Checkbox {
                    check_char,
                    content_spacing: checkbox_content_spacing,
                })
            } else {
                None
            }
        } else {
            None
        };

    Some(ListItem {
        line_index,
        marker_kind,
        marker,
        indent,
        content_offset: content_start_offset,
        content,
        byte_offset,
        byte_len: line.len(),
        checkbox,
    })
}

/// Collect all list items from a document's text, skipping fenced code blocks.
pub fn collect_list_items(text: &str) -> Vec<ListItem> {
    let mut items = Vec::new();
    let mut tracker = FenceTracker::new();
    let mut offset = 0usize;

    for (line_idx, line) in text.lines().enumerate() {
        tracker.process_line(line_idx, line);

        if !tracker.is_inside_fence() {
            if let Some(item) = parse_list_item(line, line_idx, offset) {
                items.push(item);
            }
        }

        offset += line.len() + 1;
    }

    items
}

/// Group list items into contiguous list blocks.
///
/// Items are in the same block if there are no more than 1 blank line
/// between them and they are at compatible indentation levels.
pub fn collect_list_blocks(text: &str) -> Vec<ListBlock> {
    let items = collect_list_items(text);
    if items.is_empty() {
        return Vec::new();
    }

    let lines: Vec<&str> = text.lines().collect();
    let mut blocks = Vec::new();
    let mut current_items: Vec<ListItem> = vec![items[0].clone()];

    for item in items.iter().skip(1) {
        let prev = current_items.last().unwrap();
        let gap = item.line_index - prev.line_index;

        // Check if there are only blank lines between them
        let all_blank_between = (prev.line_index + 1..item.line_index)
            .all(|l| l >= lines.len() || lines[l].trim().is_empty());

        if gap <= 2 && all_blank_between {
            current_items.push(item.clone());
        } else {
            // Finalize current block
            let start_line = current_items[0].line_index;
            let end_line = current_items.last().unwrap().line_index;
            let byte_offset = current_items[0].byte_offset;
            let last = current_items.last().unwrap();
            let byte_len = (last.byte_offset + last.byte_len) - byte_offset;
            blocks.push(ListBlock {
                items: current_items,
                start_line,
                end_line,
                byte_offset,
                byte_len,
            });
            current_items = vec![item.clone()];
        }
    }

    // Finalize last block
    if !current_items.is_empty() {
        let start_line = current_items[0].line_index;
        let end_line = current_items.last().unwrap().line_index;
        let byte_offset = current_items[0].byte_offset;
        let last = current_items.last().unwrap();
        let byte_len = (last.byte_offset + last.byte_len) - byte_offset;
        blocks.push(ListBlock {
            items: current_items,
            start_line,
            end_line,
            byte_offset,
            byte_len,
        });
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_unordered_dash() {
        let item = parse_list_item("- hello", 0, 0).unwrap();
        assert_eq!(item.marker_kind, ListMarkerKind::Dash);
        assert_eq!(item.marker, "-");
        assert_eq!(item.content, "hello");
        assert_eq!(item.indent, 0);
    }

    #[test]
    fn parse_unordered_asterisk() {
        let item = parse_list_item("* world", 0, 0).unwrap();
        assert_eq!(item.marker_kind, ListMarkerKind::Asterisk);
    }

    #[test]
    fn parse_unordered_plus() {
        let item = parse_list_item("+ item", 0, 0).unwrap();
        assert_eq!(item.marker_kind, ListMarkerKind::Plus);
    }

    #[test]
    fn parse_ordered_dot() {
        let item = parse_list_item("1. first", 0, 0).unwrap();
        assert_eq!(item.marker_kind, ListMarkerKind::OrderedDot);
        assert_eq!(item.marker, "1.");
        assert_eq!(item.content, "first");
    }

    #[test]
    fn parse_ordered_paren() {
        let item = parse_list_item("2) second", 0, 0).unwrap();
        assert_eq!(item.marker_kind, ListMarkerKind::OrderedParen);
        assert_eq!(item.marker, "2)");
    }

    #[test]
    fn parse_indented() {
        let item = parse_list_item("  - nested", 0, 0).unwrap();
        assert_eq!(item.indent, 2);
        assert_eq!(item.content, "nested");
    }

    #[test]
    fn parse_checkbox() {
        let item = parse_list_item("- [x] done", 0, 0).unwrap();
        assert!(item.checkbox.is_some());
        assert_eq!(item.checkbox.unwrap().check_char, 'x');
    }

    #[test]
    fn parse_unchecked_checkbox() {
        let item = parse_list_item("- [ ] todo", 0, 0).unwrap();
        assert_eq!(item.checkbox.unwrap().check_char, ' ');
    }

    #[test]
    fn not_a_list_item() {
        assert!(parse_list_item("just text", 0, 0).is_none());
        assert!(parse_list_item("", 0, 0).is_none());
        assert!(parse_list_item("-no space", 0, 0).is_none());
    }

    #[test]
    fn collect_items() {
        let text = "- a\n- b\n\nText\n\n- c\n";
        let items = collect_list_items(text);
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn collect_blocks() {
        let text = "- a\n- b\n\nText\n\n- c\n";
        let blocks = collect_list_blocks(text);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].items.len(), 2);
        assert_eq!(blocks[1].items.len(), 1);
    }

    #[test]
    fn items_in_fence_ignored() {
        let text = "```\n- not a list\n```\n";
        let items = collect_list_items(text);
        assert!(items.is_empty());
    }
}
