/// A detected JSX element in a line of text.
#[derive(Debug, Clone)]
pub struct JsxElement {
    /// The tag name (e.g. `Component`).
    pub tag: String,
    /// Attributes found on the opening tag.
    pub attributes: Vec<JsxAttribute>,
    /// Byte offset of the `<` in the original text.
    pub start: usize,
    /// Byte offset past the closing `>` of the opening/self-closing tag.
    pub end: usize,
    /// Whether this is a self-closing tag (`<Foo />`).
    pub self_closing: bool,
    /// Whether this has a separate closing tag.
    pub has_closing_tag: bool,
}

/// A single attribute on a JSX element.
#[derive(Debug, Clone)]
pub struct JsxAttribute {
    /// Attribute name.
    pub name: String,
    /// Raw value including quotes, if any (e.g. `"hello"`, `{true}`).
    pub value: Option<String>,
    /// Quote character used for the value, if it is a string literal.
    pub quote_char: Option<char>,
    /// Byte offset of the attribute name in the original text.
    pub byte_offset: usize,
    /// Byte length of the entire attribute (name=value).
    pub byte_len: usize,
}

/// Void HTML elements that cannot have children.
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// Check if a tag name is a void HTML element.
pub fn is_void_element(tag: &str) -> bool {
    VOID_ELEMENTS.contains(&tag.to_lowercase().as_str())
}

/// Find JSX elements in a single line of text.
///
/// JSX elements are identified by having a tag name starting with an uppercase
/// letter (to distinguish from plain HTML), or being known HTML void elements.
pub fn find_jsx_elements(line: &str, line_byte_offset: usize) -> Vec<JsxElement> {
    let mut elements = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'<' && i + 1 < len && bytes[i + 1] != b'/' && bytes[i + 1] != b'!' {
            if let Some(elem) = parse_jsx_element(line, i, line_byte_offset) {
                let end = elem.end - line_byte_offset;
                elements.push(elem);
                i = end;
                continue;
            }
        }
        i += 1;
    }

    elements
}

fn parse_jsx_element(line: &str, start: usize, line_byte_offset: usize) -> Option<JsxElement> {
    let rest = &line[start + 1..];

    // Parse tag name
    let tag_end =
        rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '-' && c != '_')?;
    if tag_end == 0 {
        return None;
    }
    let tag = &rest[..tag_end];

    // Must start with uppercase (JSX component) or be a known void element
    let first_char = tag.chars().next()?;
    if !first_char.is_uppercase() && !is_void_element(tag) {
        return None;
    }

    // Parse attributes
    let mut attrs = Vec::new();
    let pos = tag_end;
    let attr_region = &rest[pos..];

    let mut p = 0;
    while p < attr_region.len() {
        let ch = attr_region.as_bytes()[p];

        if ch == b'>' {
            // End of opening tag
            let end = start + 1 + pos + p + 1;
            let has_closing = find_closing_tag(line, end, tag);
            return Some(JsxElement {
                tag: tag.to_string(),
                attributes: attrs,
                start: line_byte_offset + start,
                end: line_byte_offset + end,
                self_closing: false,
                has_closing_tag: has_closing,
            });
        }

        if ch == b'/' && p + 1 < attr_region.len() && attr_region.as_bytes()[p + 1] == b'>' {
            let end = start + 1 + pos + p + 2;
            return Some(JsxElement {
                tag: tag.to_string(),
                attributes: attrs,
                start: line_byte_offset + start,
                end: line_byte_offset + end,
                self_closing: true,
                has_closing_tag: false,
            });
        }

        if ch.is_ascii_whitespace() {
            p += 1;
            continue;
        }

        // Parse attribute name
        if ch.is_ascii_alphabetic() || ch == b'_' {
            let attr_start = p;
            let abs_attr_start = line_byte_offset + start + 1 + pos + p;
            while p < attr_region.len()
                && (attr_region.as_bytes()[p].is_ascii_alphanumeric()
                    || attr_region.as_bytes()[p] == b'-'
                    || attr_region.as_bytes()[p] == b'_'
                    || attr_region.as_bytes()[p] == b':')
            {
                p += 1;
            }
            let attr_name = &attr_region[attr_start..p];

            // Check for `=`
            if p < attr_region.len() && attr_region.as_bytes()[p] == b'=' {
                p += 1;
                if p < attr_region.len() {
                    let val_ch = attr_region.as_bytes()[p];
                    if val_ch == b'"' || val_ch == b'\'' {
                        // Quoted value
                        let quote = val_ch as char;
                        p += 1;
                        let val_start = p;
                        while p < attr_region.len() && attr_region.as_bytes()[p] != val_ch {
                            p += 1;
                        }
                        if p < attr_region.len() {
                            p += 1; // skip closing quote
                        }
                        let val_str = &attr_region[val_start - 1..p];
                        let total_len = p - attr_start;
                        attrs.push(JsxAttribute {
                            name: attr_name.to_string(),
                            value: Some(val_str.to_string()),
                            quote_char: Some(quote),
                            byte_offset: abs_attr_start,
                            byte_len: total_len,
                        });
                    } else if val_ch == b'{' {
                        // Expression value
                        let brace_start = p;
                        let mut depth = 1;
                        p += 1;
                        while p < attr_region.len() && depth > 0 {
                            if attr_region.as_bytes()[p] == b'{' {
                                depth += 1;
                            } else if attr_region.as_bytes()[p] == b'}' {
                                depth -= 1;
                            }
                            p += 1;
                        }
                        let val_str = &attr_region[brace_start..p];
                        let total_len = p - attr_start;
                        attrs.push(JsxAttribute {
                            name: attr_name.to_string(),
                            value: Some(val_str.to_string()),
                            quote_char: None,
                            byte_offset: abs_attr_start,
                            byte_len: total_len,
                        });
                    } else {
                        // Unquoted value (skip until whitespace or >)
                        let val_start = p;
                        while p < attr_region.len()
                            && !attr_region.as_bytes()[p].is_ascii_whitespace()
                            && attr_region.as_bytes()[p] != b'>'
                            && attr_region.as_bytes()[p] != b'/'
                        {
                            p += 1;
                        }
                        let val_str = &attr_region[val_start..p];
                        let total_len = p - attr_start;
                        attrs.push(JsxAttribute {
                            name: attr_name.to_string(),
                            value: Some(val_str.to_string()),
                            quote_char: None,
                            byte_offset: abs_attr_start,
                            byte_len: total_len,
                        });
                    }
                }
            } else {
                // Boolean attribute (no value)
                let total_len = p - attr_start;
                attrs.push(JsxAttribute {
                    name: attr_name.to_string(),
                    value: None,
                    quote_char: None,
                    byte_offset: abs_attr_start,
                    byte_len: total_len,
                });
            }
            continue;
        }

        p += 1;
    }

    None
}

fn find_closing_tag(line: &str, from: usize, tag: &str) -> bool {
    let rest = &line[from..];
    let closing = format!("</{}>", tag);
    rest.contains(&closing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_jsx_element() {
        let elements = find_jsx_elements("<Component />", 0);
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].tag, "Component");
        assert!(elements[0].self_closing);
    }

    #[test]
    fn jsx_with_attributes() {
        let elements = find_jsx_elements("<Button onClick={handler} disabled>click</Button>", 0);
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].tag, "Button");
        assert_eq!(elements[0].attributes.len(), 2);
        assert_eq!(elements[0].attributes[0].name, "onClick");
        assert_eq!(elements[0].attributes[1].name, "disabled");
    }

    #[test]
    fn void_element() {
        let elements = find_jsx_elements("<br />", 0);
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].tag, "br");
        assert!(elements[0].self_closing);
    }

    #[test]
    fn quoted_attributes() {
        let elements = find_jsx_elements("<Comp name=\"hello\" title='world' />", 0);
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].attributes.len(), 2);
        assert_eq!(elements[0].attributes[0].quote_char, Some('"'));
        assert_eq!(elements[0].attributes[1].quote_char, Some('\''));
    }

    #[test]
    fn no_lowercase_components() {
        let elements = find_jsx_elements("<div className=\"foo\">text</div>", 0);
        // div is not a void element and not uppercase, should be ignored
        assert_eq!(elements.len(), 0);
    }
}
