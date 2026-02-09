/// The kind of a directive (text, leaf, or container).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectiveKind {
    /// Inline directive: `:name`
    Text,
    /// Leaf directive: `::name`
    Leaf,
    /// Container directive: `:::name`
    Container,
}

/// A parsed directive attribute.
#[derive(Debug, Clone)]
pub struct DirAttribute {
    /// The attribute name.
    pub name: String,
    /// The attribute value (if any), without quotes.
    pub value: Option<String>,
    /// Quote character used, if any.
    pub quote_char: Option<char>,
    /// Whether this is a shorthand class (`.foo`).
    pub is_class_shorthand: bool,
    /// Whether this is a shorthand id (`#foo`).
    pub is_id_shorthand: bool,
    /// Byte offset of this attribute in the original text.
    pub byte_offset: usize,
    /// Byte length of this attribute in the original text.
    pub byte_len: usize,
}

/// A parsed directive found in a line.
#[derive(Debug, Clone)]
pub struct Directive {
    /// The directive name.
    pub name: String,
    /// The kind of directive.
    pub kind: DirectiveKind,
    /// Attributes parsed from the `{...}` block.
    pub attributes: Vec<DirAttribute>,
    /// Byte offset of the directive start in the original text.
    pub start: usize,
    /// Byte offset past the end of the directive.
    pub end: usize,
}

/// Find directives in a single line of text.
///
/// Directives follow the pattern: `:name[label]{attrs}` (text),
/// `::name[label]{attrs}` (leaf), `:::name[label]{attrs}` (container).
pub fn find_directives(line: &str, line_byte_offset: usize) -> Vec<Directive> {
    let mut directives = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Look for directive pattern: colon(s) followed by name
        if bytes[i] == b':' {
            let colon_start = i;
            let mut colon_count = 0;
            while i < len && bytes[i] == b':' {
                colon_count += 1;
                i += 1;
            }

            if colon_count >= 1 && colon_count <= 3 && i < len && is_name_start(bytes[i]) {
                let kind = match colon_count {
                    1 => DirectiveKind::Text,
                    2 => DirectiveKind::Leaf,
                    _ => DirectiveKind::Container,
                };

                // Parse name
                let name_start = i;
                while i < len && is_name_char(bytes[i]) {
                    i += 1;
                }
                let name = &line[name_start..i];

                // Skip optional [label]
                if i < len && bytes[i] == b'[' {
                    let mut depth = 1;
                    i += 1;
                    while i < len && depth > 0 {
                        if bytes[i] == b'[' {
                            depth += 1;
                        } else if bytes[i] == b']' {
                            depth -= 1;
                        }
                        i += 1;
                    }
                }

                // Parse {attrs}
                let mut attributes = Vec::new();
                if i < len && bytes[i] == b'{' {
                    let brace_start = i;
                    i += 1;
                    attributes = parse_directive_attributes(line, &mut i, line_byte_offset);
                    // Skip to closing brace
                    while i < len && bytes[i] != b'}' {
                        i += 1;
                    }
                    if i < len {
                        i += 1; // skip }
                    }
                    let _ = brace_start;
                }

                directives.push(Directive {
                    name: name.to_string(),
                    kind,
                    attributes,
                    start: line_byte_offset + colon_start,
                    end: line_byte_offset + i,
                });
                continue;
            }
        } else {
            i += 1;
        }
    }

    directives
}

fn is_name_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

fn parse_directive_attributes(
    line: &str,
    pos: &mut usize,
    line_byte_offset: usize,
) -> Vec<DirAttribute> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut attrs = Vec::new();

    while *pos < len && bytes[*pos] != b'}' {
        let ch = bytes[*pos];

        if ch.is_ascii_whitespace() {
            *pos += 1;
            continue;
        }

        // Class shorthand: .foo
        if ch == b'.' {
            let attr_start = *pos;
            *pos += 1;
            let val_start = *pos;
            while *pos < len && is_name_char(bytes[*pos]) {
                *pos += 1;
            }
            let val = &line[val_start..*pos];
            attrs.push(DirAttribute {
                name: "class".to_string(),
                value: Some(val.to_string()),
                quote_char: None,
                is_class_shorthand: true,
                is_id_shorthand: false,
                byte_offset: line_byte_offset + attr_start,
                byte_len: *pos - attr_start,
            });
            continue;
        }

        // Id shorthand: #foo
        if ch == b'#' {
            let attr_start = *pos;
            *pos += 1;
            let val_start = *pos;
            while *pos < len && is_name_char(bytes[*pos]) {
                *pos += 1;
            }
            let val = &line[val_start..*pos];
            attrs.push(DirAttribute {
                name: "id".to_string(),
                value: Some(val.to_string()),
                quote_char: None,
                is_class_shorthand: false,
                is_id_shorthand: true,
                byte_offset: line_byte_offset + attr_start,
                byte_len: *pos - attr_start,
            });
            continue;
        }

        // Regular attribute: name or name=value or name="value"
        if ch.is_ascii_alphabetic() || ch == b'_' {
            let attr_start = *pos;
            while *pos < len && is_name_char(bytes[*pos]) {
                *pos += 1;
            }
            let attr_name = &line[attr_start..*pos];

            if *pos < len && bytes[*pos] == b'=' {
                *pos += 1;
                if *pos < len && (bytes[*pos] == b'"' || bytes[*pos] == b'\'') {
                    let quote = bytes[*pos] as char;
                    *pos += 1;
                    let val_start = *pos;
                    while *pos < len && bytes[*pos] != quote as u8 {
                        *pos += 1;
                    }
                    let val = &line[val_start..*pos];
                    if *pos < len {
                        *pos += 1;
                    }
                    attrs.push(DirAttribute {
                        name: attr_name.to_string(),
                        value: Some(val.to_string()),
                        quote_char: Some(quote),
                        is_class_shorthand: false,
                        is_id_shorthand: false,
                        byte_offset: line_byte_offset + attr_start,
                        byte_len: *pos - attr_start,
                    });
                } else {
                    // Unquoted value
                    let val_start = *pos;
                    while *pos < len && !bytes[*pos].is_ascii_whitespace() && bytes[*pos] != b'}' {
                        *pos += 1;
                    }
                    let val = &line[val_start..*pos];
                    attrs.push(DirAttribute {
                        name: attr_name.to_string(),
                        value: Some(val.to_string()),
                        quote_char: None,
                        is_class_shorthand: false,
                        is_id_shorthand: false,
                        byte_offset: line_byte_offset + attr_start,
                        byte_len: *pos - attr_start,
                    });
                }
            } else {
                // Boolean attribute (no value)
                attrs.push(DirAttribute {
                    name: attr_name.to_string(),
                    value: None,
                    quote_char: None,
                    is_class_shorthand: false,
                    is_id_shorthand: false,
                    byte_offset: line_byte_offset + attr_start,
                    byte_len: *pos - attr_start,
                });
            }
            continue;
        }

        *pos += 1;
    }

    attrs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_directive() {
        let directives = find_directives("text :name[label]{.class}", 0);
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].name, "name");
        assert_eq!(directives[0].kind, DirectiveKind::Text);
        assert_eq!(directives[0].attributes.len(), 1);
        assert!(directives[0].attributes[0].is_class_shorthand);
    }

    #[test]
    fn leaf_directive() {
        let directives = find_directives("::video{src=\"video.mp4\"}", 0);
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].name, "video");
        assert_eq!(directives[0].kind, DirectiveKind::Leaf);
        assert_eq!(directives[0].attributes.len(), 1);
        assert_eq!(directives[0].attributes[0].name, "src");
    }

    #[test]
    fn container_directive() {
        let directives = find_directives(":::note{#main .important}", 0);
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].name, "note");
        assert_eq!(directives[0].kind, DirectiveKind::Container);
        assert_eq!(directives[0].attributes.len(), 2);
        assert!(directives[0].attributes[0].is_id_shorthand);
        assert!(directives[0].attributes[1].is_class_shorthand);
    }

    #[test]
    fn multiple_attributes() {
        let directives = find_directives("::elem{a=\"1\" b=\"2\" c=\"3\"}", 0);
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].attributes.len(), 3);
        assert_eq!(directives[0].attributes[0].name, "a");
        assert_eq!(directives[0].attributes[1].name, "b");
        assert_eq!(directives[0].attributes[2].name, "c");
    }
}
