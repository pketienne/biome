pub mod thematic_break_block;

use biome_markdown_syntax::{T, kind::MarkdownSyntaxKind::*};
use biome_parser::{
    Parser,
    prelude::ParsedSyntax::{self, *},
};
use thematic_break_block::{at_thematic_break_block, parse_thematic_break_block};

use crate::MarkdownParser;

pub(crate) fn parse_document(p: &mut MarkdownParser) {
    let m = p.start();
    let _ = parse_block_list(p);
    m.complete(p, MD_DOCUMENT);
}

pub(crate) fn parse_block_list(p: &mut MarkdownParser) -> ParsedSyntax {
    let m = p.start();

    while !p.at(T![EOF]) {
        parse_any_block(p);
    }
    Present(m.complete(p, MD_BLOCK_LIST))
}

pub(crate) fn parse_any_block(p: &mut MarkdownParser) {
    if at_indent_code_block(p) {
        parse_indent_code_block(p);
    } else if at_header(p) {
        parse_header(p);
    } else if at_fenced_code_block(p) {
        parse_fenced_code_block(p);
    } else if at_thematic_break_block(p) {
        let break_block = try_parse(p, |p| {
            let break_block = parse_thematic_break_block(p);
            if break_block.is_absent() {
                return Err(());
            }
            Ok(break_block)
        });
        if break_block.is_err() {
            parse_paragraph(p);
        }
    } else {
        parse_paragraph(p);
    }
}

// === Headers (ATX-style) ===

/// Check if the current position starts an ATX heading (# chars at line start)
pub(crate) fn at_header(p: &mut MarkdownParser) -> bool {
    p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == "#"
}

/// Parse an ATX heading: `# Heading`, `## Heading`, etc.
pub(crate) fn parse_header(p: &mut MarkdownParser) {
    let m = p.start();

    // Parse the leading hash list (before)
    let hash_list = p.start();
    let mut first_hash = true;
    while p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == "#" {
        // After the first hash, stop if we hit a line break
        if !first_hash && p.has_preceding_line_break() {
            break;
        }
        let hash_node = p.start();
        p.bump_remap(HASH);
        hash_node.complete(p, MD_HASH);
        first_hash = false;
    }
    hash_list.complete(p, MD_HASH_LIST);

    // Parse the heading content as an optional paragraph
    // Content is everything until the next line break
    if !p.at(T![EOF]) && !p.has_preceding_line_break() {
        parse_header_content(p);
    }

    // Parse trailing hash list (after) — empty for now
    let after_list = p.start();
    after_list.complete(p, MD_HASH_LIST);

    m.complete(p, MD_HEADER);
}

/// Parse heading content as a paragraph node (what MdHeader.content() returns)
fn parse_header_content(p: &mut MarkdownParser) {
    let m = p.start();
    let list = p.start();
    while !p.at(T![EOF]) && !p.has_preceding_line_break() {
        let textual = p.start();
        p.bump_any();
        textual.complete(p, MD_TEXTUAL);
    }
    list.complete(p, MD_INLINE_ITEM_LIST);
    m.complete(p, MD_PARAGRAPH);
}

// === Fenced Code Blocks ===

/// Check if the current position starts a fenced code block (3+ backticks or tildes).
pub(crate) fn at_fenced_code_block(p: &mut MarkdownParser) -> bool {
    if p.cur() != MD_TEXTUAL_LITERAL {
        return false;
    }
    let text = p.cur_text();
    text.len() >= 3 && (text.chars().all(|c| c == '`') || text.chars().all(|c| c == '~'))
}

/// Parse a fenced code block into an MdFencedCodeBlock node.
pub(crate) fn parse_fenced_code_block(p: &mut MarkdownParser) {
    let fence_text = p.cur_text().to_string();
    let fence_char = fence_text.chars().next().unwrap();
    let fence_count = fence_text.len();

    let m = p.start();

    // Slot 0: l_fence — remap to TRIPLE_BACKTICK
    p.bump_remap(TRIPLE_BACKTICK);

    // Slot 1: code_list — language identifier tokens on same line
    let code_list = p.start();
    while !p.at(T![EOF]) && !p.has_preceding_line_break() {
        let textual = p.start();
        p.bump_any();
        textual.complete(p, MD_TEXTUAL);
    }
    code_list.complete(p, MD_CODE_NAME_LIST);

    // Slot 2: content — all tokens until closing fence
    let content = p.start();
    while !p.at(T![EOF]) {
        if is_closing_fence(p, fence_char, fence_count) {
            break;
        }
        let textual = p.start();
        p.bump_any();
        textual.complete(p, MD_TEXTUAL);
    }
    content.complete(p, MD_INLINE_ITEM_LIST);

    // Slot 3: r_fence — closing fence (if present)
    if !p.at(T![EOF]) {
        p.bump_remap(TRIPLE_BACKTICK);
        // Consume trailing tokens on closing fence line
        while !p.at(T![EOF]) && !p.has_preceding_line_break() {
            p.bump_any();
        }
    }

    m.complete(p, MD_FENCED_CODE_BLOCK);
}

/// Check if the current token is a closing fence matching the opening fence.
fn is_closing_fence(p: &mut MarkdownParser, fence_char: char, min_count: usize) -> bool {
    if !p.has_preceding_line_break() {
        return false;
    }
    if p.cur() != MD_TEXTUAL_LITERAL {
        return false;
    }
    let text = p.cur_text();
    text.len() >= min_count && text.chars().all(|c| c == fence_char)
}

pub(crate) fn at_indent_code_block(p: &mut MarkdownParser) -> bool {
    p.before_whitespace_count() > 4
}

pub(crate) fn parse_indent_code_block(p: &mut MarkdownParser) {
    // Stub: treat as paragraph
    parse_paragraph(p);
}

pub(crate) fn parse_paragraph(p: &mut MarkdownParser) {
    let m = p.start();
    let list = p.start();

    // Consume the first token (we know we're at a non-EOF position)
    if !p.at(T![EOF]) {
        let textual = p.start();
        p.bump_any();
        textual.complete(p, MD_TEXTUAL);
    }

    // Continue consuming tokens until the next line break or EOF
    while !p.at(T![EOF]) && !p.has_preceding_line_break() {
        let textual = p.start();
        p.bump_any();
        textual.complete(p, MD_TEXTUAL);
    }

    list.complete(p, MD_INLINE_ITEM_LIST);
    m.complete(p, MD_PARAGRAPH);
}

/// Attempt to parse some input with the given parsing function. If parsing
/// succeeds, `Ok` is returned with the result of the parse and the state is
/// preserved. If parsing fails, this function rewinds the parser back to
/// where it was before attempting the parse and the `Err` value is returned.
#[must_use = "The result of try_parse contains information about whether the parse succeeded and should not be ignored"]
pub(crate) fn try_parse<T, E>(
    p: &mut MarkdownParser,
    func: impl FnOnce(&mut MarkdownParser) -> Result<T, E>,
) -> Result<T, E> {
    let checkpoint = p.checkpoint();

    let res = func(p);

    if res.is_err() {
        p.rewind(checkpoint);
    }

    res
}
