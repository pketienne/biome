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
