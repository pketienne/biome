use crate::utils::fence_utils::FenceTracker;

/// A GFM table detected in the document.
#[derive(Debug, Clone)]
pub struct Table {
    /// 0-based line index of the header row.
    pub header_line: usize,
    /// 0-based line index of the separator row.
    pub separator_line: usize,
    /// 0-based line indices of data rows.
    pub data_lines: Vec<usize>,
    /// Number of columns (from separator row).
    pub column_count: usize,
    /// Byte offset of the header row start.
    pub byte_offset: usize,
    /// Byte length from start of header to end of last data row.
    pub byte_len: usize,
}

/// A single table row parsed into cells.
#[derive(Debug, Clone)]
pub struct TableRow {
    /// The raw cell contents (trimmed).
    pub cells: Vec<String>,
    /// Whether the row starts with a pipe.
    pub has_leading_pipe: bool,
    /// Whether the row ends with a pipe.
    pub has_trailing_pipe: bool,
    /// The raw line content.
    pub raw: String,
}

/// Check if a line is a GFM table separator row.
///
/// A separator row contains only pipes, hyphens, colons, and whitespace.
/// Must have at least one cell with hyphens.
pub fn is_separator_row(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Must contain at least one hyphen
    if !trimmed.contains('-') {
        return false;
    }

    // All characters must be pipe, hyphen, colon, or whitespace
    for ch in trimmed.chars() {
        if ch != '|' && ch != '-' && ch != ':' && ch != ' ' && ch != '\t' {
            return false;
        }
    }

    // Must have at least one cell (hyphen sequence)
    let cells = split_table_cells(trimmed);
    if cells.is_empty() {
        return false;
    }

    // Each cell must match pattern: optional colon, hyphens, optional colon
    for cell in &cells {
        let cell = cell.trim();
        if cell.is_empty() {
            continue;
        }
        let without_colons = cell.trim_start_matches(':').trim_end_matches(':');
        if without_colons.is_empty() || !without_colons.chars().all(|c| c == '-') {
            return false;
        }
    }

    true
}

/// Check if a line looks like a table row (contains pipes).
pub fn is_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.contains('|')
}

/// Split a table row into cells by pipe delimiter.
///
/// Handles leading/trailing pipes by stripping them.
pub fn split_table_cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    // Strip leading pipe
    let s = if trimmed.starts_with('|') {
        &trimmed[1..]
    } else {
        trimmed
    };

    // Strip trailing pipe
    let s = if s.ends_with('|') {
        &s[..s.len() - 1]
    } else {
        s
    };

    s.split('|').map(|c| c.trim().to_string()).collect()
}

/// Parse a table row into a structured `TableRow`.
pub fn parse_table_row(line: &str) -> TableRow {
    let trimmed = line.trim();
    let has_leading_pipe = trimmed.starts_with('|');
    let has_trailing_pipe = trimmed.ends_with('|');
    let cells = split_table_cells(trimmed);

    TableRow {
        cells,
        has_leading_pipe,
        has_trailing_pipe,
        raw: line.to_string(),
    }
}

/// Collect all GFM tables from a document's text.
///
/// A GFM table consists of:
/// 1. A header row (with pipes)
/// 2. A separator row (pipes, hyphens, colons)
/// 3. Zero or more data rows (with pipes)
pub fn collect_tables(text: &str) -> Vec<Table> {
    let mut tables = Vec::new();
    let mut tracker = FenceTracker::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut offsets = Vec::with_capacity(lines.len());
    let mut offset = 0usize;
    for line in &lines {
        offsets.push(offset);
        offset += line.len() + 1;
    }

    let mut i = 0;
    while i + 1 < lines.len() {
        tracker.process_line(i, lines[i]);

        if tracker.is_inside_fence() {
            i += 1;
            continue;
        }

        // Look for header + separator pattern
        if is_table_row(lines[i]) && i + 1 < lines.len() {
            // Check next line without advancing tracker (peek)
            let peek_tracker = FenceTracker::new();
            // Replay tracker state... actually we just check if separator
            if is_separator_row(lines[i + 1]) {
                let header_line = i;
                let separator_line = i + 1;
                let header_cells = split_table_cells(lines[header_line]);
                let sep_cells = split_table_cells(lines[separator_line]);
                let column_count = sep_cells.len();

                // Collect data rows
                let mut data_lines = Vec::new();
                let mut j = separator_line + 1;
                while j < lines.len() {
                    // Simple check: if the line looks like a table row
                    if is_table_row(lines[j]) && !lines[j].trim().is_empty() {
                        data_lines.push(j);
                        j += 1;
                    } else {
                        break;
                    }
                }

                let last_line = if data_lines.is_empty() {
                    separator_line
                } else {
                    *data_lines.last().unwrap()
                };

                let byte_offset = offsets[header_line];
                let byte_end = offsets[last_line] + lines[last_line].len();

                tables.push(Table {
                    header_line,
                    separator_line,
                    data_lines,
                    column_count,
                    byte_offset,
                    byte_len: byte_end - byte_offset,
                });

                let _ = (header_cells, peek_tracker);
                i = last_line + 1;
                continue;
            }
        }

        i += 1;
    }

    // Process last line for fence tracking
    if i < lines.len() {
        tracker.process_line(i, lines[i]);
    }

    tables
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separator_detection() {
        assert!(is_separator_row("| --- | --- |"));
        assert!(is_separator_row("|---|---|"));
        assert!(is_separator_row("| :--- | :---: | ---: |"));
        assert!(is_separator_row("--- | ---"));
        assert!(!is_separator_row("| text | text |"));
        assert!(!is_separator_row(""));
        assert!(!is_separator_row("just text"));
    }

    #[test]
    fn cell_splitting() {
        let cells = split_table_cells("| a | b | c |");
        assert_eq!(cells, vec!["a", "b", "c"]);

        let cells = split_table_cells("a | b | c");
        assert_eq!(cells, vec!["a", "b", "c"]);
    }

    #[test]
    fn table_row_parsing() {
        let row = parse_table_row("| a | b |");
        assert!(row.has_leading_pipe);
        assert!(row.has_trailing_pipe);
        assert_eq!(row.cells, vec!["a", "b"]);
    }

    #[test]
    fn table_collection() {
        let text = "# Heading\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n| 3 | 4 |\n\nText after.\n";
        let tables = collect_tables(text);
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].column_count, 2);
        assert_eq!(tables[0].header_line, 2);
        assert_eq!(tables[0].separator_line, 3);
        assert_eq!(tables[0].data_lines, vec![4, 5]);
    }

    #[test]
    fn no_table() {
        let text = "# Heading\n\nJust some text.\n";
        let tables = collect_tables(text);
        assert!(tables.is_empty());
    }

    #[test]
    fn table_in_fence_ignored() {
        let text = "```\n| A | B |\n| --- | --- |\n| 1 | 2 |\n```\n";
        let tables = collect_tables(text);
        assert!(tables.is_empty());
    }
}
