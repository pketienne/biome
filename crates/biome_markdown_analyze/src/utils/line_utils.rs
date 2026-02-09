use biome_rowan::{TextRange, TextSize};

/// Pre-computed line information for a document's text.
///
/// Stores line boundaries so that line-based rules can efficiently
/// map between line numbers and `TextRange`s.
pub struct DocumentLines {
    /// Byte offset of the start of each line (relative to the text slice).
    line_starts: Vec<usize>,
    text_len: usize,
}

impl DocumentLines {
    /// Build line metadata from the raw text of a syntax node.
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Self {
            line_starts,
            text_len: text.len(),
        }
    }

    /// Number of lines in the text.
    pub fn len(&self) -> usize {
        self.line_starts.len()
    }

    /// Returns `true` if the text is empty (zero lines).
    pub fn is_empty(&self) -> bool {
        self.line_starts.is_empty()
    }

    /// Return the byte offset where line `n` (0-based) starts.
    pub fn line_start(&self, n: usize) -> usize {
        self.line_starts[n]
    }

    /// Return the byte offset where line `n` (0-based) ends (exclusive, before newline).
    pub fn line_end(&self, n: usize) -> usize {
        if n + 1 < self.line_starts.len() {
            // The line ends just before the `\n` that starts the next line.
            // Subtract 1 for the `\n` itself.
            self.line_starts[n + 1] - 1
        } else {
            self.text_len
        }
    }

    /// Get the content of line `n` from `text`.
    pub fn line_content<'a>(&self, text: &'a str, n: usize) -> &'a str {
        &text[self.line_start(n)..self.line_end(n)]
    }

    /// Convert a 0-based line index to a `TextRange`, offset by `base`.
    pub fn line_range(&self, n: usize, base: TextSize) -> TextRange {
        let start = base + TextSize::from(self.line_start(n) as u32);
        let end = base + TextSize::from(self.line_end(n) as u32);
        TextRange::new(start, end)
    }
}

/// Returns `true` if `line` is blank (empty or whitespace-only).
pub fn is_blank_line(line: &str) -> bool {
    line.trim().is_empty()
}

/// Returns the number of leading space/tab characters in `line`.
pub fn leading_indent(line: &str) -> usize {
    line.bytes()
        .take_while(|&b| b == b' ' || b == b'\t')
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_lines_basic() {
        let text = "line1\nline2\nline3";
        let lines = DocumentLines::new(text);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines.line_content(text, 0), "line1");
        assert_eq!(lines.line_content(text, 1), "line2");
        assert_eq!(lines.line_content(text, 2), "line3");
    }

    #[test]
    fn document_lines_trailing_newline() {
        let text = "a\nb\n";
        let lines = DocumentLines::new(text);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines.line_content(text, 0), "a");
        assert_eq!(lines.line_content(text, 1), "b");
        assert_eq!(lines.line_content(text, 2), "");
    }

    #[test]
    fn blank_and_indent() {
        assert!(is_blank_line(""));
        assert!(is_blank_line("   "));
        assert!(is_blank_line("\t"));
        assert!(!is_blank_line("hello"));
        assert_eq!(leading_indent("   hello"), 3);
        assert_eq!(leading_indent("\thello"), 1);
        assert_eq!(leading_indent("hello"), 0);
    }
}
