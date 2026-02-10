use crate::prelude::*;
use biome_formatter::{CstFormatContext, write};
use biome_markdown_syntax::MdTable;
use biome_rowan::{AstNode, AstNodeList, Direction, SyntaxElement};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatMdTable;

fn split_cells(row_text: &str) -> Vec<String> {
    let trimmed = row_text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let s = if trimmed.starts_with('|') {
        &trimmed[1..]
    } else {
        trimmed
    };
    let s = if s.ends_with('|') {
        &s[..s.len() - 1]
    } else {
        s
    };
    s.split('|').map(|c| c.trim().to_string()).collect()
}

fn format_row(cells: &[String], widths: &[usize]) -> String {
    let mut parts = Vec::new();
    for (i, cell) in cells.iter().enumerate() {
        let w = widths.get(i).copied().unwrap_or(3);
        parts.push(std::format!(" {:<width$} ", cell, width = w));
    }
    for i in cells.len()..widths.len() {
        parts.push(std::format!(" {:<width$} ", "", width = widths[i]));
    }
    std::format!("|{}|", parts.join("|"))
}

fn format_separator(widths: &[usize]) -> String {
    let parts: Vec<String> = widths
        .iter()
        .map(|w| std::format!(" {} ", "-".repeat(*w)))
        .collect();
    std::format!("|{}|", parts.join("|"))
}

impl FormatNodeRule<MdTable> for FormatMdTable {
    fn fmt_fields(&self, node: &MdTable, f: &mut MarkdownFormatter) -> FormatResult<()> {
        for element in node.syntax().descendants_with_tokens(Direction::Next) {
            match element {
                SyntaxElement::Token(token) => f.state_mut().track_token(&token),
                SyntaxElement::Node(child) => {
                    f.context().comments().mark_suppression_checked(&child);
                }
            }
        }

        let header = match node.header() {
            Ok(h) => h,
            Err(_) => {
                let node_text = node.syntax().text_trimmed().to_string();
                return write!(
                    f,
                    [text(&node_text, node.syntax().text_trimmed_range().start())]
                );
            }
        };

        let header_text = header.syntax().text_trimmed().to_string();
        let header_cells = split_cells(&header_text);

        let sep_cells = node
            .separator()
            .ok()
            .map(|s| split_cells(&s.syntax().text_trimmed().to_string()))
            .unwrap_or_default();

        let data_rows: Vec<_> = node.rows().iter().collect();
        let data_cells: Vec<Vec<String>> = data_rows
            .iter()
            .map(|r| split_cells(&r.syntax().text_trimmed().to_string()))
            .collect();

        // Determine number of columns
        let num_cols = header_cells
            .len()
            .max(sep_cells.len())
            .max(data_cells.iter().map(|r| r.len()).max().unwrap_or(0));

        if num_cols == 0 {
            let node_text = node.syntax().text_trimmed().to_string();
            return write!(
                f,
                [text(&node_text, node.syntax().text_trimmed_range().start())]
            );
        }

        // Compute max column widths
        let mut col_widths = vec![3usize; num_cols];
        for (i, cell) in header_cells.iter().enumerate() {
            col_widths[i] = col_widths[i].max(cell.len());
        }
        for row_cells in &data_cells {
            for (i, cell) in row_cells.iter().enumerate() {
                if i < num_cols {
                    col_widths[i] = col_widths[i].max(cell.len());
                }
            }
        }

        // Build formatted table
        let mut result = format_row(&header_cells, &col_widths);
        result.push('\n');
        result.push_str(&format_separator(&col_widths));
        for row_cells in &data_cells {
            result.push('\n');
            result.push_str(&format_row(row_cells, &col_widths));
        }

        write!(
            f,
            [text(&result, node.syntax().text_trimmed_range().start())]
        )
    }
}
