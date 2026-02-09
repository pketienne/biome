/// Find the matching closing bracket/parenthesis for an opening one.
///
/// Starting at `start`, which must point to an `open` byte, walks forward
/// tracking nesting depth and returns the index of the matching `close` byte.
pub fn find_matching_bracket(bytes: &[u8], start: usize, open: u8, close: u8) -> Option<usize> {
    let mut depth = 0;
    for i in start..bytes.len() {
        if bytes[i] == open {
            depth += 1;
        } else if bytes[i] == close {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

/// Heuristic check for whether a string looks like a URL or path.
///
/// Returns `true` for strings starting with common URL schemes,
/// fragment identifiers, or relative/absolute paths.
pub fn looks_like_url(s: &str) -> bool {
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("ftp://")
        || s.starts_with("mailto:")
        || s.starts_with('#')
        || s.starts_with('/')
        || s.starts_with("./")
        || s.starts_with("../")
        || s.contains('.')
}

/// A span of inline code found in a line (byte offsets within the line).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeSpan {
    /// Byte offset of the opening backtick(s).
    pub open: usize,
    /// Byte offset past the closing backtick(s) (exclusive).
    pub close: usize,
    /// Number of backticks used for the delimiter.
    pub backtick_count: usize,
}

/// Find all inline code spans in a single line.
///
/// Handles multi-backtick delimiters (e.g. `` `code` `` or ``` ``code`` ```).
/// Backslash-escaped backticks are NOT special inside code spans per CommonMark.
pub fn find_code_spans(line: &str) -> Vec<CodeSpan> {
    let bytes = line.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'`' {
            let open_start = i;
            let mut count = 0;
            while i < bytes.len() && bytes[i] == b'`' {
                count += 1;
                i += 1;
            }
            // Look for matching closing delimiter with same backtick count
            let mut j = i;
            loop {
                // Find next run of backticks
                while j < bytes.len() && bytes[j] != b'`' {
                    j += 1;
                }
                if j >= bytes.len() {
                    break;
                }
                let _close_start = j;
                let mut close_count = 0;
                while j < bytes.len() && bytes[j] == b'`' {
                    close_count += 1;
                    j += 1;
                }
                if close_count == count {
                    spans.push(CodeSpan {
                        open: open_start,
                        close: j,
                        backtick_count: count,
                    });
                    i = j;
                    break;
                }
                // Not a match, keep looking
            }
            if i <= open_start + count {
                // No closing delimiter found, move past opening backticks
                i = open_start + count;
            }
        } else {
            i += 1;
        }
    }

    spans
}

/// Returns `true` if byte offset `pos` is inside any of the given code spans.
pub fn is_in_code_span(pos: usize, spans: &[CodeSpan]) -> bool {
    spans.iter().any(|s| pos >= s.open && pos < s.close)
}

/// An emphasis or strong marker found in a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmphasisMarker {
    /// Byte offset of the first marker character.
    pub start: usize,
    /// Number of marker characters (1 = emphasis, 2 = strong, 3 = both).
    pub count: usize,
    /// The marker character ('*' or '_').
    pub marker_char: char,
    /// Whether this is an opening or closing marker (heuristic).
    pub is_opening: bool,
}

/// Find emphasis/strong markers in a line, skipping code spans.
///
/// This is a heuristic approach — it finds runs of `*` or `_` that look like
/// emphasis delimiters. It skips markers inside code spans.
pub fn find_emphasis_markers(line: &str, code_spans: &[CodeSpan]) -> Vec<EmphasisMarker> {
    let bytes = line.as_bytes();
    let mut markers = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        if is_in_code_span(i, code_spans) {
            i += 1;
            continue;
        }

        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            // Skip escaped character
            i += 2;
            continue;
        }

        if bytes[i] == b'*' || bytes[i] == b'_' {
            let marker_byte = bytes[i];
            let start = i;
            let mut count = 0;
            while i < bytes.len() && bytes[i] == marker_byte {
                count += 1;
                i += 1;
            }

            // Heuristic: opening if followed by non-space, closing if preceded by non-space
            let preceded_by_space = start == 0 || bytes[start - 1] == b' ';
            let followed_by_space = i >= bytes.len() || bytes[i] == b' ';

            let is_opening = !followed_by_space;

            // Skip if both sides are spaces (not a valid delimiter)
            if preceded_by_space && followed_by_space {
                continue;
            }

            markers.push(EmphasisMarker {
                start,
                count,
                marker_char: marker_byte as char,
                is_opening,
            });
        } else {
            i += 1;
        }
    }

    markers
}

/// An HTML tag found in a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlTag {
    /// Byte offset of the '<'.
    pub start: usize,
    /// Byte offset past the '>'.
    pub end: usize,
    /// The tag name (lowercase).
    pub tag_name: String,
    /// Whether this is a closing tag (e.g. `</div>`).
    pub is_closing: bool,
    /// Whether this is a self-closing tag (e.g. `<br/>`).
    pub is_self_closing: bool,
}

/// Find HTML tags in a line, skipping code spans.
pub fn find_html_tags(line: &str, code_spans: &[CodeSpan]) -> Vec<HtmlTag> {
    let bytes = line.as_bytes();
    let mut tags = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        if is_in_code_span(i, code_spans) {
            i += 1;
            continue;
        }

        if bytes[i] == b'<' {
            let tag_start = i;
            i += 1;

            // Check for closing tag
            let is_closing = i < bytes.len() && bytes[i] == b'/';
            if is_closing {
                i += 1;
            }

            // Read tag name (must start with a letter)
            if i >= bytes.len() || !bytes[i].is_ascii_alphabetic() {
                continue;
            }

            let name_start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-') {
                i += 1;
            }
            let tag_name = line[name_start..i].to_ascii_lowercase();

            // Skip to closing '>'
            while i < bytes.len() && bytes[i] != b'>' {
                i += 1;
            }
            if i >= bytes.len() {
                continue;
            }

            let is_self_closing = i > 0 && bytes[i - 1] == b'/';
            i += 1; // skip '>'

            tags.push(HtmlTag {
                start: tag_start,
                end: i,
                tag_name,
                is_closing,
                is_self_closing,
            });
        } else {
            i += 1;
        }
    }

    tags
}

/// An inline link or image found in a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineLink {
    /// Byte offset of the start (the `[` or `![`).
    pub start: usize,
    /// Byte offset past the end (after the closing `)`).
    pub end: usize,
    /// Whether this is an image (`![`) vs a link (`[`).
    pub is_image: bool,
    /// The link text (between `[` and `]`).
    pub text: String,
    /// The URL/destination (between `(` and `)`, before any title).
    pub url: String,
    /// The optional title.
    pub title: Option<String>,
    /// The title delimiter character if present.
    pub title_delimiter: Option<char>,
}

/// A reference link or image found in a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceLink {
    /// Byte offset of the start.
    pub start: usize,
    /// Byte offset past the end.
    pub end: usize,
    /// Whether this is an image.
    pub is_image: bool,
    /// The link text.
    pub text: String,
    /// The reference label (empty for shortcut/collapsed).
    pub label: String,
    /// The kind of reference.
    pub kind: ReferenceLinkKind,
}

/// The kind of reference link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceLinkKind {
    /// `[text][label]` — full reference.
    Full,
    /// `[text][]` — collapsed reference.
    Collapsed,
    /// `[text]` — shortcut reference (no second bracket pair).
    Shortcut,
}

/// Find all inline links and images in a line, skipping code spans.
///
/// Detects patterns like `[text](url)`, `[text](url "title")`,
/// `![alt](url)`, etc.
pub fn find_inline_links(line: &str, code_spans: &[CodeSpan]) -> Vec<InlineLink> {
    let bytes = line.as_bytes();
    let mut links = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        if is_in_code_span(i, code_spans) {
            i += 1;
            continue;
        }

        // Check for image prefix
        let is_image = i + 1 < bytes.len() && bytes[i] == b'!' && bytes[i + 1] == b'[';
        let bracket_start = if is_image { i + 1 } else { i };

        if bytes[bracket_start] == b'[' {
            if let Some(close_bracket) = find_matching_bracket(bytes, bracket_start, b'[', b']') {
                // Check if followed by '('
                if close_bracket + 1 < bytes.len() && bytes[close_bracket + 1] == b'(' {
                    if let Some(close_paren) =
                        find_matching_bracket(bytes, close_bracket + 1, b'(', b')')
                    {
                        let text = line[bracket_start + 1..close_bracket].to_string();
                        let raw_dest = &line[close_bracket + 2..close_paren];

                        // Parse URL and optional title from destination
                        let (url, title, title_delim) = parse_link_destination(raw_dest);

                        links.push(InlineLink {
                            start: if is_image { i } else { bracket_start },
                            end: close_paren + 1,
                            is_image,
                            text,
                            url,
                            title,
                            title_delimiter: title_delim,
                        });

                        i = close_paren + 1;
                        continue;
                    }
                }
            }
        }

        i += 1;
    }

    links
}

/// Find all reference links and images in a line, skipping code spans.
///
/// Detects patterns like `[text][label]`, `[text][]`, `[text]`.
pub fn find_reference_links(line: &str, code_spans: &[CodeSpan]) -> Vec<ReferenceLink> {
    let bytes = line.as_bytes();
    let mut links = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        if is_in_code_span(i, code_spans) {
            i += 1;
            continue;
        }

        let is_image = i + 1 < bytes.len() && bytes[i] == b'!' && bytes[i + 1] == b'[';
        let bracket_start = if is_image { i + 1 } else { i };

        if bytes[bracket_start] == b'[' {
            if let Some(close_bracket) = find_matching_bracket(bytes, bracket_start, b'[', b']') {
                let text = line[bracket_start + 1..close_bracket].to_string();

                // Check what follows the first ]
                if close_bracket + 1 < bytes.len() && bytes[close_bracket + 1] == b'[' {
                    // Full or collapsed reference: [text][label] or [text][]
                    if let Some(close_label) =
                        find_matching_bracket(bytes, close_bracket + 1, b'[', b']')
                    {
                        let label = line[close_bracket + 2..close_label].to_string();
                        let kind = if label.is_empty() {
                            ReferenceLinkKind::Collapsed
                        } else {
                            ReferenceLinkKind::Full
                        };

                        links.push(ReferenceLink {
                            start: if is_image { i } else { bracket_start },
                            end: close_label + 1,
                            is_image,
                            text,
                            label,
                            kind,
                        });

                        i = close_label + 1;
                        continue;
                    }
                } else if close_bracket + 1 < bytes.len() && bytes[close_bracket + 1] == b'(' {
                    // This is an inline link, skip it (handled by find_inline_links)
                    i = close_bracket + 2;
                    continue;
                } else {
                    // Shortcut reference: [text]
                    // But only if text is not empty and doesn't look like other syntax
                    if !text.is_empty() {
                        links.push(ReferenceLink {
                            start: if is_image { i } else { bracket_start },
                            end: close_bracket + 1,
                            is_image,
                            text: text.clone(),
                            label: String::new(),
                            kind: ReferenceLinkKind::Shortcut,
                        });
                    }
                }
            }
        }

        i += 1;
    }

    links
}

/// Parse a link destination, extracting URL and optional title.
///
/// Input is the content between `(` and `)` of an inline link.
fn parse_link_destination(raw: &str) -> (String, Option<String>, Option<char>) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return (String::new(), None, None);
    }

    // Check for title at the end: url "title", url 'title', url (title)
    for &(open, close) in &[('"', '"'), ('\'', '\''), ('(', ')')] {
        if let Some(title_end_pos) = trimmed.rfind(close) {
            if title_end_pos > 0 {
                let before = &trimmed[..title_end_pos];
                if let Some(title_start_pos) = before.rfind(open) {
                    // Check there's a space before the title opening quote
                    if title_start_pos > 0 {
                        let url_part = trimmed[..title_start_pos].trim_end();
                        if !url_part.is_empty() {
                            let title = trimmed[title_start_pos + 1..title_end_pos].to_string();
                            return (url_part.to_string(), Some(title), Some(open));
                        }
                    }
                }
            }
        }
    }

    (trimmed.to_string(), None, None)
}

/// Check if a bare URL pattern exists at the given position in a line.
///
/// Returns `Some((start, end))` for the URL extent if found.
pub fn find_bare_url(line: &str, start: usize) -> Option<(usize, usize)> {
    let rest = &line[start..];
    // Check for common URL schemes
    for scheme in &["https://", "http://", "ftp://", "mailto:"] {
        if rest.starts_with(scheme) {
            // Find end of URL (next whitespace or end of line)
            let url_end = rest
                .find(|c: char| c.is_whitespace() || c == '>' || c == ')' || c == ']')
                .unwrap_or(rest.len());
            if url_end > scheme.len() {
                return Some((start, start + url_end));
            }
        }
    }
    None
}

/// Find all bare URLs in a line (not wrapped in `<>` or `[]()`).
pub fn find_bare_urls(line: &str, code_spans: &[CodeSpan]) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut urls = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        if is_in_code_span(i, code_spans) {
            i += 1;
            continue;
        }

        // Check if preceded by '<' (autolink) or '[', '(' (markdown link syntax)
        let preceded_by_special =
            i > 0 && (bytes[i - 1] == b'<' || bytes[i - 1] == b'(' || bytes[i - 1] == b'[');

        if !preceded_by_special {
            if let Some((start, end)) = find_bare_url(line, i) {
                urls.push((start, end));
                i = end;
                continue;
            }
        }

        i += 1;
    }

    urls
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_matching_basic() {
        let bytes = b"[hello]";
        assert_eq!(find_matching_bracket(bytes, 0, b'[', b']'), Some(6));
    }

    #[test]
    fn find_matching_nested() {
        let bytes = b"([inner])";
        assert_eq!(find_matching_bracket(bytes, 0, b'(', b')'), Some(8));
        assert_eq!(find_matching_bracket(bytes, 1, b'[', b']'), Some(7));
    }

    #[test]
    fn find_matching_none() {
        let bytes = b"[no close";
        assert_eq!(find_matching_bracket(bytes, 0, b'[', b']'), None);
    }

    #[test]
    fn url_heuristics() {
        assert!(looks_like_url("https://example.com"));
        assert!(looks_like_url("http://example.com"));
        assert!(looks_like_url("ftp://files.example.com"));
        assert!(looks_like_url("mailto:user@example.com"));
        assert!(looks_like_url("#fragment"));
        assert!(looks_like_url("/absolute/path"));
        assert!(looks_like_url("./relative"));
        assert!(looks_like_url("../parent"));
        assert!(looks_like_url("file.txt"));
        assert!(!looks_like_url("just text"));
    }

    #[test]
    fn code_spans_basic() {
        let spans = find_code_spans("hello `code` world");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].open, 6);
        assert_eq!(spans[0].close, 12);
        assert_eq!(spans[0].backtick_count, 1);
    }

    #[test]
    fn code_spans_double_backtick() {
        let spans = find_code_spans("use ``code with ` inside`` here");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].backtick_count, 2);
    }

    #[test]
    fn code_spans_multiple() {
        let spans = find_code_spans("`a` and `b`");
        assert_eq!(spans.len(), 2);
    }

    #[test]
    fn code_spans_unclosed() {
        let spans = find_code_spans("hello `unclosed");
        assert_eq!(spans.len(), 0);
    }

    #[test]
    fn emphasis_markers_basic() {
        let code_spans = find_code_spans("hello *world* end");
        let markers = find_emphasis_markers("hello *world* end", &code_spans);
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].marker_char, '*');
        assert_eq!(markers[0].count, 1);
        assert!(markers[0].is_opening);
    }

    #[test]
    fn emphasis_markers_strong() {
        let code_spans = find_code_spans("hello **world** end");
        let markers = find_emphasis_markers("hello **world** end", &code_spans);
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].count, 2);
    }

    #[test]
    fn emphasis_markers_skip_code() {
        let line = "hello `*not emphasis*` world";
        let code_spans = find_code_spans(line);
        let markers = find_emphasis_markers(line, &code_spans);
        assert_eq!(markers.len(), 0);
    }

    #[test]
    fn html_tags_basic() {
        let line = "hello <em>world</em> end";
        let code_spans = find_code_spans(line);
        let tags = find_html_tags(line, &code_spans);
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].tag_name, "em");
        assert!(!tags[0].is_closing);
        assert_eq!(tags[1].tag_name, "em");
        assert!(tags[1].is_closing);
    }

    #[test]
    fn html_tags_self_closing() {
        let line = "line <br/> break";
        let code_spans = find_code_spans(line);
        let tags = find_html_tags(line, &code_spans);
        assert_eq!(tags.len(), 1);
        assert!(tags[0].is_self_closing);
        assert_eq!(tags[0].tag_name, "br");
    }

    #[test]
    fn html_tags_skip_code() {
        let line = "hello `<em>not html</em>` world";
        let code_spans = find_code_spans(line);
        let tags = find_html_tags(line, &code_spans);
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn inline_link_basic() {
        let line = "click [here](https://example.com) now";
        let code_spans = find_code_spans(line);
        let links = find_inline_links(line, &code_spans);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].text, "here");
        assert_eq!(links[0].url, "https://example.com");
        assert!(!links[0].is_image);
    }

    #[test]
    fn inline_link_with_title() {
        let line = "[text](url \"title\")";
        let code_spans = find_code_spans(line);
        let links = find_inline_links(line, &code_spans);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "url");
        assert_eq!(links[0].title.as_deref(), Some("title"));
    }

    #[test]
    fn inline_image() {
        let line = "![alt](image.png)";
        let code_spans = find_code_spans(line);
        let links = find_inline_links(line, &code_spans);
        assert_eq!(links.len(), 1);
        assert!(links[0].is_image);
        assert_eq!(links[0].text, "alt");
        assert_eq!(links[0].url, "image.png");
    }

    #[test]
    fn reference_link_full() {
        let line = "[text][label]";
        let code_spans = find_code_spans(line);
        let refs = find_reference_links(line, &code_spans);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].text, "text");
        assert_eq!(refs[0].label, "label");
        assert_eq!(refs[0].kind, ReferenceLinkKind::Full);
    }

    #[test]
    fn reference_link_collapsed() {
        let line = "[text][]";
        let code_spans = find_code_spans(line);
        let refs = find_reference_links(line, &code_spans);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].kind, ReferenceLinkKind::Collapsed);
    }

    #[test]
    fn reference_link_shortcut() {
        let line = "see [text] for details";
        let code_spans = find_code_spans(line);
        let refs = find_reference_links(line, &code_spans);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].kind, ReferenceLinkKind::Shortcut);
    }

    #[test]
    fn bare_url_detection() {
        let line = "visit https://example.com for details";
        let code_spans = find_code_spans(line);
        let urls = find_bare_urls(line, &code_spans);
        assert_eq!(urls.len(), 1);
        assert_eq!(&line[urls[0].0..urls[0].1], "https://example.com");
    }

    #[test]
    fn bare_url_in_angle_brackets() {
        let line = "visit <https://example.com> for details";
        let code_spans = find_code_spans(line);
        let urls = find_bare_urls(line, &code_spans);
        assert_eq!(urls.len(), 0); // preceded by '<', not bare
    }
}
