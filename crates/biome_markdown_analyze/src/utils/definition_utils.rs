/// A parsed link reference definition: `[label]: url "title"`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkDefinition {
    /// The normalized label (trimmed, collapsed whitespace, lowercased).
    pub label: String,
    /// The raw label text as written.
    pub raw_label: String,
    /// The URL/destination.
    pub url: String,
    /// The optional title (without surrounding quotes).
    pub title: Option<String>,
    /// The title delimiter character ('"', '\'', or '(').
    pub title_delimiter: Option<char>,
    /// The 0-based line index where this definition appears.
    pub line_index: usize,
    /// Byte offset of the start of this definition in the document text.
    pub byte_offset: usize,
    /// Byte length of the entire definition line.
    pub byte_len: usize,
}

/// Normalize a link reference label: trim, collapse internal whitespace, lowercase.
pub fn normalize_label(raw: &str) -> String {
    raw.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Try to parse a link reference definition from a single line.
///
/// Format: `[label]: destination "title"` or `[label]: destination 'title'`
/// or `[label]: destination (title)`.
///
/// Returns `None` if the line is not a valid definition.
pub fn parse_definition(
    line: &str,
    line_index: usize,
    byte_offset: usize,
) -> Option<LinkDefinition> {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();

    // Definitions can have at most 3 spaces of indentation
    if indent > 3 {
        return None;
    }

    let bytes = trimmed.as_bytes();
    if bytes.is_empty() || bytes[0] != b'[' {
        return None;
    }

    // Find closing bracket for label
    let mut i = 1;
    let mut bracket_depth = 1;
    while i < bytes.len() && bracket_depth > 0 {
        match bytes[i] {
            b'\\' => {
                i += 1; // skip escaped char
            }
            b'[' => bracket_depth += 1,
            b']' => bracket_depth -= 1,
            _ => {}
        }
        i += 1;
    }

    if bracket_depth != 0 {
        return None;
    }

    let label_end = i - 1; // position of the ']'
    let raw_label = trimmed[1..label_end].to_string();

    // Label must not be empty
    if raw_label.trim().is_empty() {
        return None;
    }

    // Must be followed by ':'
    if i >= bytes.len() || bytes[i] != b':' {
        return None;
    }
    i += 1;

    // Skip optional whitespace after ':'
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }

    // Skip optional angle brackets around URL
    let url_in_angles = i < bytes.len() && bytes[i] == b'<';
    if url_in_angles {
        i += 1;
    }

    // Parse URL
    let url_start = i;
    if url_in_angles {
        while i < bytes.len() && bytes[i] != b'>' {
            if bytes[i] == b'\\' {
                i += 1; // skip escaped char
            }
            i += 1;
        }
        let url = trimmed[url_start..i].to_string();
        if i < bytes.len() {
            i += 1; // skip '>'
        }
        parse_definition_title(
            trimmed,
            i,
            raw_label,
            url,
            line_index,
            byte_offset,
            line.len(),
        )
    } else {
        while i < bytes.len() && bytes[i] != b' ' && bytes[i] != b'\t' {
            i += 1;
        }
        let url = trimmed[url_start..i].to_string();
        parse_definition_title(
            trimmed,
            i,
            raw_label,
            url,
            line_index,
            byte_offset,
            line.len(),
        )
    }
}

fn parse_definition_title(
    trimmed: &str,
    mut i: usize,
    raw_label: String,
    url: String,
    line_index: usize,
    byte_offset: usize,
    byte_len: usize,
) -> Option<LinkDefinition> {
    let bytes = trimmed.as_bytes();

    // Skip whitespace before optional title
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }

    let (title, title_delimiter) = if i < bytes.len() {
        let delim = bytes[i];
        let close_delim = match delim {
            b'"' => b'"',
            b'\'' => b'\'',
            b'(' => b')',
            _ => {
                // If there's non-whitespace remaining that's not a title, it's still valid
                // if the rest is empty — otherwise not a definition
                let rest = trimmed[i..].trim();
                if rest.is_empty() {
                    return Some(LinkDefinition {
                        label: normalize_label(&raw_label),
                        raw_label,
                        url,
                        title: None,
                        title_delimiter: None,
                        line_index,
                        byte_offset,
                        byte_len,
                    });
                }
                return None;
            }
        };

        i += 1;
        let title_start = i;
        while i < bytes.len() && bytes[i] != close_delim {
            if bytes[i] == b'\\' {
                i += 1;
            }
            i += 1;
        }

        if i >= bytes.len() {
            return None; // Unclosed title
        }

        let title_text = trimmed[title_start..i].to_string();
        (Some(title_text), Some(delim as char))
    } else {
        (None, None)
    };

    Some(LinkDefinition {
        label: normalize_label(&raw_label),
        raw_label,
        url,
        title,
        title_delimiter,
        line_index,
        byte_offset,
        byte_len,
    })
}

/// Collect all link reference definitions from a document's text.
///
/// Skips definitions inside fenced code blocks.
pub fn collect_definitions(text: &str) -> Vec<LinkDefinition> {
    use crate::utils::fence_utils::FenceTracker;

    let mut defs = Vec::new();
    let mut tracker = FenceTracker::new();
    let mut offset = 0usize;

    for (line_idx, line) in text.lines().enumerate() {
        tracker.process_line(line_idx, line);

        if !tracker.is_inside_fence() {
            if let Some(def) = parse_definition(line, line_idx, offset) {
                defs.push(def);
            }
        }

        offset += line.len() + 1;
    }

    defs
}

/// Collect all link reference definitions from the AST by walking `MdLinkBlock` descendants.
///
/// This is the preferred method over `collect_definitions()` — it uses the parser's AST
/// rather than re-parsing the text.
pub fn collect_definitions_from_ast(
    document: &biome_markdown_syntax::MdDocument,
) -> Vec<LinkDefinition> {
    use biome_markdown_syntax::MdLinkBlock;
    use biome_rowan::AstNode;

    let full_text = document.syntax().text_with_trivia().to_string();
    let doc_start = u32::from(document.syntax().text_range_with_trivia().start()) as usize;
    let mut defs = Vec::new();

    for node in document.syntax().descendants() {
        let Some(link_block) = MdLinkBlock::cast(node) else {
            continue;
        };

        let raw_label = link_block.label().syntax().text_trimmed().to_string();
        if raw_label.trim().is_empty() {
            continue;
        }

        let url = link_block.url().syntax().text_trimmed().to_string();

        let (title, title_delimiter) = if let Some(title_node) = link_block.title() {
            let delim = title_node
                .delimiter_token()
                .ok()
                .and_then(|t| t.text_trimmed().chars().next());
            let content = title_node.content().syntax().text_trimmed().to_string();
            (Some(content), delim)
        } else {
            (None, None)
        };

        // Compute line-based fields matching the text-based collect_definitions() semantics
        let node_start = u32::from(link_block.syntax().text_trimmed_range().start()) as usize;
        let relative_start = node_start - doc_start;
        let text_before = &full_text[..relative_start];
        let line_index = text_before.bytes().filter(|&b| b == b'\n').count();
        let line_start = text_before.rfind('\n').map_or(0, |p| p + 1);
        let line_end = full_text[line_start..]
            .find('\n')
            .map_or(full_text.len(), |p| line_start + p);

        defs.push(LinkDefinition {
            label: normalize_label(&raw_label),
            raw_label,
            url,
            title,
            title_delimiter,
            line_index,
            byte_offset: doc_start + line_start,
            byte_len: line_end - line_start,
        });
    }

    defs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_definition() {
        let def = parse_definition("[foo]: https://example.com", 0, 0).unwrap();
        assert_eq!(def.label, "foo");
        assert_eq!(def.url, "https://example.com");
        assert_eq!(def.title, None);
    }

    #[test]
    fn definition_with_title() {
        let def = parse_definition("[foo]: https://example.com \"Example\"", 0, 0).unwrap();
        assert_eq!(def.label, "foo");
        assert_eq!(def.url, "https://example.com");
        assert_eq!(def.title.as_deref(), Some("Example"));
        assert_eq!(def.title_delimiter, Some('"'));
    }

    #[test]
    fn definition_with_single_quote_title() {
        let def = parse_definition("[foo]: https://example.com 'Example'", 0, 0).unwrap();
        assert_eq!(def.title_delimiter, Some('\''));
    }

    #[test]
    fn definition_with_paren_title() {
        let def = parse_definition("[foo]: https://example.com (Example)", 0, 0).unwrap();
        assert_eq!(def.title_delimiter, Some('('));
        assert_eq!(def.title.as_deref(), Some("Example"));
    }

    #[test]
    fn definition_with_angle_brackets() {
        let def = parse_definition("[foo]: <https://example.com>", 0, 0).unwrap();
        assert_eq!(def.url, "https://example.com");
    }

    #[test]
    fn label_normalization() {
        assert_eq!(normalize_label("  Foo  Bar  "), "foo bar");
        assert_eq!(normalize_label("FOO"), "foo");
    }

    #[test]
    fn indented_definition() {
        let def = parse_definition("   [foo]: https://example.com", 0, 0).unwrap();
        assert_eq!(def.label, "foo");
    }

    #[test]
    fn too_much_indent() {
        assert!(parse_definition("    [foo]: https://example.com", 0, 0).is_none());
    }

    #[test]
    fn not_a_definition() {
        assert!(parse_definition("just a line", 0, 0).is_none());
        assert!(parse_definition("[]: https://example.com", 0, 0).is_none());
    }

    #[test]
    fn collect_skips_fences() {
        let text = "before\n[a]: url1\n```\n[b]: url2\n```\n[c]: url3\n";
        let defs = collect_definitions(text);
        let labels: Vec<&str> = defs.iter().map(|d| d.label.as_str()).collect();
        assert_eq!(labels, vec!["a", "c"]);
    }

    #[test]
    fn multiple_spaces_in_definition() {
        let def = parse_definition("[foo]:   https://example.com   \"title\"", 0, 0).unwrap();
        assert_eq!(def.url, "https://example.com");
        assert_eq!(def.title.as_deref(), Some("title"));
    }
}
