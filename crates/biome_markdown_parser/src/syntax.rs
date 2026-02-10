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
    } else if at_blockquote(p) {
        parse_blockquote(p);
    } else if at_unordered_list(p) {
        parse_unordered_list_item(p);
    } else if at_ordered_list(p) {
        parse_ordered_list_item(p);
    } else if at_thematic_break_block(p) {
        let break_block = try_parse(p, |p| {
            let break_block = parse_thematic_break_block(p);
            if break_block.is_absent() {
                return Err(());
            }
            Ok(break_block)
        });
        if break_block.is_err() {
            let para = parse_paragraph(p);
            maybe_wrap_setext_header(p, para);
        }
    } else if at_table_start(p) {
        if !try_parse_table(p) {
            let para = parse_paragraph(p);
            maybe_wrap_setext_header(p, para);
        }
    } else if at_link_definition(p) {
        if !try_parse_link_definition(p) {
            let para = parse_paragraph(p);
            maybe_wrap_setext_header(p, para);
        }
    } else if at_html_block(p) {
        parse_html_block(p);
    } else {
        let para = parse_paragraph(p);
        // Check if this paragraph is followed by a setext underline (=== or ---)
        maybe_wrap_setext_header(p, para);
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
    parse_inline_list(p);
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

// === Blockquotes ===

/// Check if the current position starts a blockquote (`>` at line start).
fn at_blockquote(p: &mut MarkdownParser) -> bool {
    if p.cur() != MD_TEXTUAL_LITERAL {
        return false;
    }
    p.cur_text() == ">"
}

/// Parse a blockquote into an MdQuote node.
/// MdQuote has 2 slots: `>` token (R_ANGLE) and content (MdBlockList).
/// Supports nested blockquotes (`> > text`) and lazy continuation (lines without `>`).
fn parse_blockquote(p: &mut MarkdownParser) {
    let m = p.start();

    // Slot 0: `>` marker — remap to R_ANGLE
    p.bump_remap(R_ANGLE);

    // Slot 1: content as MdBlockList
    let block_list = p.start();

    // Check for nested blockquote (another `>` on the same line)
    if !p.at(T![EOF])
        && !p.has_preceding_line_break()
        && p.cur() == MD_TEXTUAL_LITERAL
        && p.cur_text() == ">"
    {
        // Nested blockquote: `> > content` — recursively parse inner blockquote
        parse_blockquote(p);
    } else {
        // Regular content — parse as paragraph with continuation
        let para = p.start();
        let list = p.start();

        // First line content (after `>`)
        while !p.at(T![EOF]) && !p.has_preceding_line_break() {
            let textual = p.start();
            p.bump_any();
            textual.complete(p, MD_TEXTUAL);
        }

        // Continuation lines: `>` followed by content, or lazy continuation
        while !p.at(T![EOF])
            && p.has_preceding_line_break()
            && !p.has_preceding_blank_line()
        {
            if p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == ">" {
                // Standard continuation with `>` marker
                let textual = p.start();
                p.bump_any();
                textual.complete(p, MD_TEXTUAL);
                while !p.at(T![EOF]) && !p.has_preceding_line_break() {
                    let textual = p.start();
                    p.bump_any();
                    textual.complete(p, MD_TEXTUAL);
                }
            } else if !at_continuation_stop(p) {
                // Lazy continuation — line without `>` but not a new block start.
                // The first token has a preceding line break, so consume it unconditionally.
                let textual = p.start();
                p.bump_any();
                textual.complete(p, MD_TEXTUAL);
                while !p.at(T![EOF]) && !p.has_preceding_line_break() {
                    let textual = p.start();
                    p.bump_any();
                    textual.complete(p, MD_TEXTUAL);
                }
            } else {
                break;
            }
        }

        list.complete(p, MD_INLINE_ITEM_LIST);
        para.complete(p, MD_PARAGRAPH);
    }

    block_list.complete(p, MD_BLOCK_LIST);

    m.complete(p, MD_QUOTE);
}

// === Unordered Lists ===

/// Check if the current position starts an unordered list item (`-`, `*`, or `+`
/// followed by content on the same line, with less than 4 spaces of indentation).
fn at_unordered_list(p: &mut MarkdownParser) -> bool {
    if p.cur() != MD_TEXTUAL_LITERAL {
        return false;
    }
    let text = p.cur_text();
    (text == "-" || text == "*" || text == "+") && p.before_whitespace_count() < 4
}

/// Parse an unordered list item into MdBulletListItem > MdBulletList > MdBullet* nodes.
/// Only collects sibling bullets at the same indentation level.
fn parse_unordered_list_item(p: &mut MarkdownParser) {
    let list_indent = p.before_whitespace_count();
    let m = p.start();
    let list = p.start();

    // Parse consecutive bullet items at the same indent level
    parse_bullet(p, list_indent);
    while !p.at(T![EOF]) && at_bullet_start_at_indent(p, list_indent) {
        parse_bullet(p, list_indent);
    }

    list.complete(p, MD_BULLET_LIST);
    m.complete(p, MD_BULLET_LIST_ITEM);
}

/// Check if the current token is a bullet marker at the start of a new line,
/// at the expected indentation level.
fn at_bullet_start_at_indent(p: &mut MarkdownParser, expected_indent: usize) -> bool {
    if !p.has_preceding_line_break() {
        return false;
    }
    if p.cur() != MD_TEXTUAL_LITERAL {
        return false;
    }
    let text = p.cur_text();
    (text == "-" || text == "*" || text == "+")
        && p.before_whitespace_count() == expected_indent
}

/// Parse a single bullet: marker + optional checkbox + content block list.
/// MdBullet has 3 slots: bullet token (MINUS/STAR/PLUS), optional MdCheckbox,
/// and content (MdBlockList containing a MdParagraph and optional nested blocks).
fn parse_bullet(p: &mut MarkdownParser, list_indent: usize) {
    let m = p.start();

    // Slot 0: bullet marker — remap to the proper token kind
    let marker = p.cur_text().to_string();
    match marker.as_str() {
        "-" => p.bump_remap(MINUS),
        "*" => p.bump_remap(STAR),
        "+" => p.bump_remap(PLUS),
        _ => p.bump_any(),
    }

    // Slot 1: optional checkbox — [ ] or [x] or [X]
    try_parse_checkbox(p);

    // Slot 2: content — wrapped in MdBlockList > MdParagraph + optional nested blocks
    let content_indent = list_indent + 2;
    let block_list = p.start();
    let para = p.start();
    parse_multiline_inline_list(p, content_indent);
    para.complete(p, MD_PARAGRAPH);

    // Check for nested list items at deeper indent
    while !p.at(T![EOF]) && p.has_preceding_line_break() && !p.has_preceding_blank_line() {
        let indent = p.before_whitespace_count();
        if indent < content_indent {
            break;
        }
        if at_unordered_list(p) {
            parse_unordered_list_item(p);
        } else if at_ordered_list(p) {
            parse_ordered_list_item(p);
        } else {
            break;
        }
    }

    block_list.complete(p, MD_BLOCK_LIST);

    m.complete(p, MD_BULLET);
}

// === Ordered Lists ===

/// Check if the current position starts an ordered list item (digits followed by `.` or `)`).
fn at_ordered_list(p: &mut MarkdownParser) -> bool {
    if p.cur() != MD_TEXTUAL_LITERAL {
        return false;
    }
    if p.before_whitespace_count() >= 4 {
        return false;
    }
    let text = p.cur_text();
    is_ordered_marker(text)
}

/// Check if a text token looks like an ordered list marker (e.g. "1.", "2)", "10.").
fn is_ordered_marker(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    // Must have at least one digit followed by `.` or `)` and nothing else
    i > 0 && i + 1 == bytes.len() && (bytes[i] == b'.' || bytes[i] == b')')
}

/// Parse an ordered list item into MdOrderListItem > MdOrderList > MdOrderBullet* nodes.
/// Only collects sibling bullets at the same indentation level.
fn parse_ordered_list_item(p: &mut MarkdownParser) {
    let list_indent = p.before_whitespace_count();
    let m = p.start();
    let list = p.start();

    // Parse consecutive ordered items at the same indent level
    parse_order_bullet(p, list_indent);
    while !p.at(T![EOF]) && at_order_bullet_start_at_indent(p, list_indent) {
        parse_order_bullet(p, list_indent);
    }

    list.complete(p, MD_ORDER_LIST);
    m.complete(p, MD_ORDER_LIST_ITEM);
}

/// Check if the current token is an ordered list marker at the start of a new line,
/// at the expected indentation level.
fn at_order_bullet_start_at_indent(p: &mut MarkdownParser, expected_indent: usize) -> bool {
    if !p.has_preceding_line_break() {
        return false;
    }
    if p.cur() != MD_TEXTUAL_LITERAL {
        return false;
    }
    is_ordered_marker(p.cur_text()) && p.before_whitespace_count() == expected_indent
}

/// Parse a single ordered bullet: marker + optional checkbox + content block list.
/// MdOrderBullet has 3 slots: marker (MD_TEXTUAL_LITERAL), optional MdCheckbox,
/// and content (MdBlockList containing a MdParagraph and optional nested blocks).
fn parse_order_bullet(p: &mut MarkdownParser, list_indent: usize) {
    let m = p.start();

    // Slot 0: marker (e.g. "1.", "2)") — keep as MD_TEXTUAL_LITERAL
    let marker_width = p.cur_text().len();
    p.bump_any();

    // Slot 1: optional checkbox — [ ] or [x] or [X]
    try_parse_checkbox(p);

    // Slot 2: content — wrapped in MdBlockList > MdParagraph + optional nested blocks
    // Continuation indent is list indent + marker width + 1 space (e.g. "1. " = 3)
    let content_indent = list_indent + marker_width + 1;
    let block_list = p.start();
    let para = p.start();
    parse_multiline_inline_list(p, content_indent);
    para.complete(p, MD_PARAGRAPH);

    // Check for nested list items at deeper indent
    while !p.at(T![EOF]) && p.has_preceding_line_break() && !p.has_preceding_blank_line() {
        let indent = p.before_whitespace_count();
        if indent < content_indent {
            break;
        }
        if at_unordered_list(p) {
            parse_unordered_list_item(p);
        } else if at_ordered_list(p) {
            parse_ordered_list_item(p);
        } else {
            break;
        }
    }

    block_list.complete(p, MD_BLOCK_LIST);

    m.complete(p, MD_ORDER_BULLET);
}

// === Task List Checkbox ===

/// Try to parse a task list checkbox: `[ ]`, `[x]`, or `[X]`.
/// MdCheckbox has 3 slots: `[` (L_BRACK), optional value (MD_TEXTUAL_LITERAL), `]` (R_BRACK).
/// For `[ ]`, value is absent (space is trivia). For `[x]`/`[X]`, value is present.
/// If the current position doesn't look like a checkbox, does nothing (slot remains absent).
fn try_parse_checkbox(p: &mut MarkdownParser) {
    // Must be `[` on the same line as the bullet marker
    if p.at(T![EOF]) || p.has_preceding_line_break() {
        return;
    }
    if p.cur() != MD_TEXTUAL_LITERAL || p.cur_text() != "[" {
        return;
    }

    // Use try_parse to rewind if this isn't a valid checkbox
    let _ = try_parse(p, |p| {
        let m = p.start();

        // Slot 0: [
        p.bump_remap(L_BRACK);

        // Slot 1: optional value — "x" or "X" (for `[ ]`, space is trivia so value is absent)
        if !p.at(T![EOF]) && !p.has_preceding_line_break() && p.cur() == MD_TEXTUAL_LITERAL {
            let value = p.cur_text();
            if value == "x" || value == "X" {
                p.bump_any(); // keep as MD_TEXTUAL_LITERAL
            }
        }

        // Slot 2: ]
        if p.at(T![EOF])
            || p.has_preceding_line_break()
            || p.cur() != MD_TEXTUAL_LITERAL
            || p.cur_text() != "]"
        {
            m.abandon(p);
            return Err(());
        }
        p.bump_remap(R_BRACK);

        m.complete(p, MD_CHECKBOX);
        Ok(())
    });
}

pub(crate) fn at_indent_code_block(p: &mut MarkdownParser) -> bool {
    p.before_whitespace_count() >= 4
}

pub(crate) fn parse_indent_code_block(p: &mut MarkdownParser) {
    let m = p.start();
    let content = p.start();
    let mut first = true;

    while !p.at(T![EOF]) {
        if !first && p.has_preceding_line_break() {
            // Blank lines are allowed inside indented code blocks,
            // but stop if next non-blank line has < 4 indent
            if p.has_preceding_blank_line() {
                if p.before_whitespace_count() < 4 {
                    break;
                }
            } else if p.before_whitespace_count() < 4 {
                break;
            }
        }
        first = false;
        let textual = p.start();
        p.bump_any();
        textual.complete(p, MD_TEXTUAL);
    }

    content.complete(p, MD_INLINE_ITEM_LIST);
    m.complete(p, MD_INDENT_CODE_BLOCK);
}

pub(crate) fn parse_paragraph(p: &mut MarkdownParser) -> biome_parser::CompletedMarker {
    let m = p.start();
    parse_inline_list(p);
    m.complete(p, MD_PARAGRAPH)
}

// === Inline Content Parsing ===

/// Parse inline content into an MdInlineItemList.
/// Detects inline elements: code spans, links, emphasis, and italic.
/// All other tokens become MdTextual nodes.
///
/// The first token is always consumed regardless of preceding line breaks,
/// because callers (e.g. `parse_paragraph`) invoke this at the start of a
/// new block where the token legitimately has a preceding line break from
/// the previous block. Subsequent tokens stop at line boundaries.
fn parse_inline_list(p: &mut MarkdownParser) {
    let list = p.start();
    let mut first = true;

    while !p.at(T![EOF]) && (first || !p.has_preceding_line_break()) {
        first = false;
        if !try_parse_one_inline(p) {
            let textual = p.start();
            p.bump_any();
            textual.complete(p, MD_TEXTUAL);
        }
    }

    list.complete(p, MD_INLINE_ITEM_LIST);
}

/// Try to parse a single inline element at the current position.
/// Returns `true` if an inline element was parsed, `false` if the current
/// token should be treated as textual content.
fn try_parse_one_inline(p: &mut MarkdownParser) -> bool {
    if at_inline_code(p) {
        parse_inline_code(p);
        return true;
    }
    if at_inline_image_start(p) {
        return try_parse_inline_image(p);
    }
    if at_inline_link_start(p) {
        return try_parse_inline_link(p);
    }
    if at_inline_strikethrough_start(p) {
        return try_parse_inline_strikethrough(p);
    }
    if at_inline_emphasis_start(p) {
        return try_parse_inline_emphasis_or_italic(p);
    }
    if at_inline_directive_start(p) {
        return try_parse_inline_directive(p);
    }
    if at_inline_mdx_jsx_start(p) {
        return try_parse_inline_mdx_jsx(p);
    }
    false
}

/// Parse inline content with nesting until the given delimiter text is found.
/// Returns `true` if the delimiter was found, `false` if EOF or line break
/// was reached first. The delimiter token is NOT consumed.
fn parse_inline_content_until_delimiter(
    p: &mut MarkdownParser,
    delimiter_text: &str,
) -> bool {
    let mut found = false;
    while !p.at(T![EOF]) && !p.has_preceding_line_break() {
        // Check stop condition BEFORE inline dispatch — this prevents
        // same-type nesting (e.g. `**` inside `**...**` stops the loop).
        if p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == delimiter_text {
            found = true;
            break;
        }
        if !try_parse_one_inline(p) {
            let textual = p.start();
            p.bump_any();
            textual.complete(p, MD_TEXTUAL);
        }
    }
    found
}

/// Parse inline content that may span multiple lines.
/// The first line is always consumed (like `parse_inline_list`).
/// Subsequent lines are consumed as continuation if:
///   - No blank line precedes them
///   - They have at least `min_indent` spaces of indentation
///   - They don't start a new list marker
fn parse_multiline_inline_list(p: &mut MarkdownParser, min_indent: usize) {
    let list = p.start();
    let mut first = true;

    while !p.at(T![EOF]) {
        if !first && p.has_preceding_line_break() {
            // At a line boundary — check if we should continue
            if p.has_preceding_blank_line() {
                break;
            }
            if p.before_whitespace_count() < min_indent {
                break;
            }
            // Don't continue into new list markers or block-level starts
            if at_continuation_stop(p) {
                break;
            }
        }
        first = false;
        if !try_parse_one_inline(p) {
            let textual = p.start();
            p.bump_any();
            textual.complete(p, MD_TEXTUAL);
        }
    }

    list.complete(p, MD_INLINE_ITEM_LIST);
}

/// Check if the current position is a block-level element that should stop
/// list item continuation. Checks for list markers, headers, fenced code,
/// blockquotes, and HTML blocks.
fn at_continuation_stop(p: &mut MarkdownParser) -> bool {
    if p.cur() != MD_TEXTUAL_LITERAL {
        return false;
    }
    let text = p.cur_text();
    // Unordered list marker
    if (text == "-" || text == "*" || text == "+") && p.before_whitespace_count() < 4 {
        return true;
    }
    // Ordered list marker
    if is_ordered_marker(text) && p.before_whitespace_count() < 4 {
        return true;
    }
    // ATX heading
    if text == "#" {
        return true;
    }
    // Blockquote
    if text == ">" {
        return true;
    }
    false
}

/// Check if the current token starts an inline code span (backtick delimiter).
fn at_inline_code(p: &mut MarkdownParser) -> bool {
    if p.cur() != MD_TEXTUAL_LITERAL {
        return false;
    }
    let text = p.cur_text();
    !text.is_empty() && text.chars().all(|c| c == '`')
}

/// Parse an inline code span: `code` or ``code with `backtick` inside``
/// MdInlineCode has 3 slots: l_tick (BACKTICK), content (MdInlineItemList), r_tick (BACKTICK)
fn parse_inline_code(p: &mut MarkdownParser) {
    let opening_len = p.cur_text().len();
    let m = p.start();

    // Slot 0: opening backtick(s)
    p.bump_remap(BACKTICK);

    // Slot 1: content — everything until matching closing backtick(s)
    let content = p.start();
    let mut found_close = false;

    while !p.at(T![EOF]) && !p.has_preceding_line_break() {
        // Check if this token is a matching closing backtick run
        if p.cur() == MD_TEXTUAL_LITERAL {
            let text = p.cur_text();
            if text.len() == opening_len && text.chars().all(|c| c == '`') {
                found_close = true;
                break;
            }
        }
        let textual = p.start();
        p.bump_any();
        textual.complete(p, MD_TEXTUAL);
    }

    content.complete(p, MD_INLINE_ITEM_LIST);

    // Slot 2: closing backtick(s)
    if found_close {
        p.bump_remap(BACKTICK);
    }

    m.complete(p, MD_INLINE_CODE);
}

// === Inline Links ===

/// Check if the current token starts an inline link (`[`).
fn at_inline_link_start(p: &mut MarkdownParser) -> bool {
    p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == "["
}

/// Try to parse an inline link: `[text](url)`.
/// MdInlineLink has 6 slots: `[`, text, `]`, `(`, source, `)`.
/// Returns true if a link was successfully parsed.
fn try_parse_inline_link(p: &mut MarkdownParser) -> bool {
    try_parse(p, |p| {
        let m = p.start();

        // Slot 0: [
        p.bump_remap(L_BRACK);

        // Slot 1: text content (MdInlineItemList) — with nested inline parsing
        let text = p.start();
        let found_bracket = parse_inline_content_until_delimiter(p, "]");
        text.complete(p, MD_INLINE_ITEM_LIST);

        // Slot 2: ]
        if !found_bracket {
            m.abandon(p);
            return Err(());
        }
        p.bump_remap(R_BRACK);

        // Slot 3: ( — must follow ] without line break
        if p.at(T![EOF])
            || p.has_preceding_line_break()
            || p.cur() != MD_TEXTUAL_LITERAL
            || p.cur_text() != "("
        {
            m.abandon(p);
            return Err(());
        }
        p.bump_remap(L_PAREN);

        // Slot 4: source content (MdInlineItemList)
        let source = p.start();
        while !p.at(T![EOF]) && !p.has_preceding_line_break() {
            if p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == ")" {
                break;
            }
            let textual = p.start();
            p.bump_any();
            textual.complete(p, MD_TEXTUAL);
        }
        source.complete(p, MD_INLINE_ITEM_LIST);

        // Slot 5: )
        if p.cur() != MD_TEXTUAL_LITERAL || p.cur_text() != ")" {
            m.abandon(p);
            return Err(());
        }
        p.bump_remap(R_PAREN);

        m.complete(p, MD_INLINE_LINK);
        Ok(())
    })
    .is_ok()
}

// === Inline Images ===

/// Check if the current token starts an inline image (`!`).
fn at_inline_image_start(p: &mut MarkdownParser) -> bool {
    p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == "!"
}

/// Try to parse an inline image: `![alt](url)`.
/// MdInlineImage has 4 slots: `!`, alt (MdInlineImageAlt), source (MdInlineImageSource), link? (absent).
/// Returns true if an image was successfully parsed.
fn try_parse_inline_image(p: &mut MarkdownParser) -> bool {
    try_parse(p, |p| {
        let m = p.start();

        // Slot 0: ! (BANG)
        p.bump_remap(BANG);

        // Slot 1: MdInlineImageAlt = [ content ]
        if p.cur() != MD_TEXTUAL_LITERAL || p.cur_text() != "[" {
            m.abandon(p);
            return Err(());
        }
        let alt = p.start();
        p.bump_remap(L_BRACK);
        let alt_content = p.start();
        let found_bracket = parse_inline_content_until_delimiter(p, "]");
        alt_content.complete(p, MD_INLINE_ITEM_LIST);
        if !found_bracket {
            alt.abandon(p);
            m.abandon(p);
            return Err(());
        }
        p.bump_remap(R_BRACK);
        alt.complete(p, MD_INLINE_IMAGE_ALT);

        // Slot 2: MdInlineImageSource = ( content )
        if p.at(T![EOF])
            || p.has_preceding_line_break()
            || p.cur() != MD_TEXTUAL_LITERAL
            || p.cur_text() != "("
        {
            m.abandon(p);
            return Err(());
        }
        let source = p.start();
        p.bump_remap(L_PAREN);
        let src_content = p.start();
        while !p.at(T![EOF]) && !p.has_preceding_line_break() {
            if p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == ")" {
                break;
            }
            let textual = p.start();
            p.bump_any();
            textual.complete(p, MD_TEXTUAL);
        }
        src_content.complete(p, MD_INLINE_ITEM_LIST);
        if p.cur() != MD_TEXTUAL_LITERAL || p.cur_text() != ")" {
            source.abandon(p);
            m.abandon(p);
            return Err(());
        }
        p.bump_remap(R_PAREN);
        source.complete(p, MD_INLINE_IMAGE_SOURCE);

        // Slot 3: MdInlineImageLink — absent for basic images

        m.complete(p, MD_INLINE_IMAGE);
        Ok(())
    })
    .is_ok()
}

// === Inline Emphasis & Italic ===

/// Check if the current token starts emphasis or italic (`*`, `**`, `_`, `__`).
fn at_inline_emphasis_start(p: &mut MarkdownParser) -> bool {
    if p.cur() != MD_TEXTUAL_LITERAL {
        return false;
    }
    let text = p.cur_text();
    text == "*" || text == "**" || text == "_" || text == "__"
}

/// Try to parse emphasis (bold) or italic.
/// Tries bold (`**`/`__`) first, then italic (`*`/`_`).
fn try_parse_inline_emphasis_or_italic(p: &mut MarkdownParser) -> bool {
    let text = p.cur_text().to_string();
    match text.as_str() {
        "**" | "__" => try_parse_inline_emphasis(p),
        "*" | "_" => try_parse_inline_italic(p),
        _ => false,
    }
}

/// Try to parse bold emphasis: `**text**` or `__text__`.
/// MdInlineEmphasis has 3 slots: l_fence (DOUBLE_STAR/DOUBLE_UNDERSCORE), content, r_fence.
fn try_parse_inline_emphasis(p: &mut MarkdownParser) -> bool {
    let delimiter = p.cur_text().to_string();
    let remap_kind = if delimiter == "**" {
        DOUBLE_STAR
    } else {
        DOUBLE_UNDERSCORE
    };

    try_parse(p, |p| {
        let m = p.start();

        // Slot 0: opening delimiter (** or __)
        p.bump_remap(remap_kind);

        // Slot 1: content (MdInlineItemList) — with nested inline parsing
        let content = p.start();
        let found_close = parse_inline_content_until_delimiter(p, &delimiter);
        content.complete(p, MD_INLINE_ITEM_LIST);

        if !found_close {
            m.abandon(p);
            return Err(());
        }

        // Slot 2: closing delimiter (** or __)
        p.bump_remap(remap_kind);

        m.complete(p, MD_INLINE_EMPHASIS);
        Ok(())
    })
    .is_ok()
}

/// Try to parse italic: `*text*` or `_text_`.
/// MdInlineItalic has 3 slots: l_fence (STAR/UNDERSCORE), content, r_fence.
fn try_parse_inline_italic(p: &mut MarkdownParser) -> bool {
    let delimiter = p.cur_text().to_string();
    let remap_kind = if delimiter == "*" {
        STAR
    } else {
        UNDERSCORE
    };

    try_parse(p, |p| {
        let m = p.start();

        // Slot 0: opening delimiter (* or _)
        p.bump_remap(remap_kind);

        // Slot 1: content (MdInlineItemList) — with nested inline parsing
        let content = p.start();
        let found_close = parse_inline_content_until_delimiter(p, &delimiter);
        content.complete(p, MD_INLINE_ITEM_LIST);

        if !found_close {
            m.abandon(p);
            return Err(());
        }

        // Slot 2: closing delimiter (* or _)
        p.bump_remap(remap_kind);

        m.complete(p, MD_INLINE_ITALIC);
        Ok(())
    })
    .is_ok()
}

// === Inline Strikethrough ===

/// Check if the current token starts a strikethrough (`~~`).
fn at_inline_strikethrough_start(p: &mut MarkdownParser) -> bool {
    p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == "~~"
}

/// Try to parse strikethrough: `~~text~~`.
/// MdInlineStrikethrough has 3 slots: l_fence (DOUBLE_TILDE), content (MdInlineItemList), r_fence (DOUBLE_TILDE).
fn try_parse_inline_strikethrough(p: &mut MarkdownParser) -> bool {
    try_parse(p, |p| {
        let m = p.start();

        // Slot 0: opening ~~
        p.bump_remap(DOUBLE_TILDE);

        // Slot 1: content (MdInlineItemList) — with nested inline parsing
        let content = p.start();
        let found_close = parse_inline_content_until_delimiter(p, "~~");
        content.complete(p, MD_INLINE_ITEM_LIST);

        if !found_close {
            m.abandon(p);
            return Err(());
        }

        // Slot 2: closing ~~
        p.bump_remap(DOUBLE_TILDE);

        m.complete(p, MD_INLINE_STRIKETHROUGH);
        Ok(())
    })
    .is_ok()
}

// === Directives ===

/// Check if the current position starts a directive (`:name`, `::name`, `:::name`).
/// The lexer combines consecutive `:` into a single token (up to 3).
fn at_inline_directive_start(p: &mut MarkdownParser) -> bool {
    if p.cur() != MD_TEXTUAL_LITERAL {
        return false;
    }
    let text = p.cur_text();
    text == ":" || text == "::" || text == ":::"
}

/// Try to parse a directive: `:name{attrs}`, `::name{attrs}`, `:::name{attrs}`.
/// MdDirective has 5 slots: marker(0), name(1), l_curly(2)?, attributes(3), r_curly(4)?
fn try_parse_inline_directive(p: &mut MarkdownParser) -> bool {
    try_parse(p, |p| {
        let m = p.start();

        // Slot 0: marker (`:`, `::`, or `:::`)
        p.bump_any(); // MD_TEXTUAL_LITERAL stays as-is

        // The next token must be a name character (letter or underscore).
        if p.at(T![EOF])
            || p.has_preceding_line_break()
            || p.cur() != MD_TEXTUAL_LITERAL
        {
            m.abandon(p);
            return Err(());
        }
        let first_char = p.cur_text().as_bytes()[0];
        if !(first_char.is_ascii_alphabetic() || first_char == b'_') {
            m.abandon(p);
            return Err(());
        }

        // Slot 1: name (MdInlineItemList) — consume name characters (alphanumeric, -, _)
        let name = p.start();
        while !p.at(T![EOF]) && !p.has_preceding_line_break() && p.cur() == MD_TEXTUAL_LITERAL {
            let ch = p.cur_text().as_bytes()[0];
            if ch.is_ascii_alphanumeric() || ch == b'-' || ch == b'_' {
                let textual = p.start();
                p.bump_any();
                textual.complete(p, MD_TEXTUAL);
            } else {
                break;
            }
        }
        name.complete(p, MD_INLINE_ITEM_LIST);

        // Slots 2-4: optional `{attrs}` block
        if !p.at(T![EOF])
            && !p.has_preceding_line_break()
            && p.cur() == MD_TEXTUAL_LITERAL
            && p.cur_text() == "{"
        {
            // Slot 2: `{` (L_CURLY)
            p.bump_remap(L_CURLY);

            // Slot 3: attributes (MdDirectiveAttributeList)
            let attr_list = p.start();
            parse_directive_attributes(p);
            attr_list.complete(p, MD_DIRECTIVE_ATTRIBUTE_LIST);

            // Slot 4: `}` (R_CURLY)
            if !p.at(T![EOF])
                && !p.has_preceding_line_break()
                && p.cur() == MD_TEXTUAL_LITERAL
                && p.cur_text() == "}"
            {
                p.bump_remap(R_CURLY);
            }
        } else {
            // No braces: emit empty attribute list (slot 3)
            let attr_list = p.start();
            attr_list.complete(p, MD_DIRECTIVE_ATTRIBUTE_LIST);
        }

        m.complete(p, MD_DIRECTIVE);
        Ok(())
    })
    .is_ok()
}

/// Parse the attributes inside a directive's `{...}` block.
/// Produces MdDirectiveAttribute nodes in the current list context.
fn parse_directive_attributes(p: &mut MarkdownParser) {
    while !p.at(T![EOF]) && !p.has_preceding_line_break() {
        if p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == "}" {
            break;
        }

        // Skip whitespace tokens between attributes
        if p.cur() == WHITESPACE || p.cur() == TAB {
            p.bump_any();
            continue;
        }

        let ch = if p.cur() == MD_TEXTUAL_LITERAL {
            p.cur_text().as_bytes()[0]
        } else {
            p.bump_any();
            continue;
        };

        if ch == b'.' || ch == b'#' {
            // Shorthand: .class or #id
            let attr = p.start();
            let attr_name = p.start();
            let t = p.start();
            p.bump_any(); // `.` or `#`
            t.complete(p, MD_TEXTUAL);
            // Consume the value chars
            while !p.at(T![EOF])
                && !p.has_preceding_line_break()
                && p.cur() == MD_TEXTUAL_LITERAL
            {
                let b = p.cur_text().as_bytes()[0];
                if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' {
                    let t = p.start();
                    p.bump_any();
                    t.complete(p, MD_TEXTUAL);
                } else {
                    break;
                }
            }
            attr_name.complete(p, MD_INLINE_ITEM_LIST);
            // No `=` or value for shorthands
            attr.complete(p, MD_DIRECTIVE_ATTRIBUTE);
        } else if ch.is_ascii_alphabetic() || ch == b'_' {
            // Regular attribute: name, name=value, name="value"
            let attr = p.start();
            let attr_name = p.start();
            while !p.at(T![EOF])
                && !p.has_preceding_line_break()
                && p.cur() == MD_TEXTUAL_LITERAL
            {
                let b = p.cur_text().as_bytes()[0];
                if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' {
                    let t = p.start();
                    p.bump_any();
                    t.complete(p, MD_TEXTUAL);
                } else {
                    break;
                }
            }
            attr_name.complete(p, MD_INLINE_ITEM_LIST);

            // Check for `=`
            if !p.at(T![EOF])
                && !p.has_preceding_line_break()
                && p.cur() == MD_TEXTUAL_LITERAL
                && p.cur_text() == "="
            {
                p.bump_remap(EQ); // consume `=` as EQ token

                // Check for quoted value
                if !p.at(T![EOF])
                    && !p.has_preceding_line_break()
                    && p.cur() == MD_TEXTUAL_LITERAL
                    && (p.cur_text() == "\"" || p.cur_text() == "'")
                {
                    let quote = p.cur_text().to_string();
                    let val = p.start();
                    p.bump_any(); // opening quote (delimiter)
                    let val_content = p.start();
                    while !p.at(T![EOF])
                        && !p.has_preceding_line_break()
                        && !(p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == quote)
                    {
                        let t = p.start();
                        p.bump_any();
                        t.complete(p, MD_TEXTUAL);
                    }
                    val_content.complete(p, MD_INLINE_ITEM_LIST);
                    if !p.at(T![EOF])
                        && !p.has_preceding_line_break()
                        && p.cur() == MD_TEXTUAL_LITERAL
                        && p.cur_text() == quote
                    {
                        p.bump_any(); // closing quote (closing_delimiter)
                    }
                    val.complete(p, MD_DIRECTIVE_ATTRIBUTE_VALUE);
                } else {
                    // Unquoted value — no delimiters, just content
                    let val = p.start();
                    let val_content = p.start();
                    while !p.at(T![EOF])
                        && !p.has_preceding_line_break()
                        && p.cur() == MD_TEXTUAL_LITERAL
                    {
                        let b = p.cur_text().as_bytes()[0];
                        if b.is_ascii_whitespace() || b == b'}' {
                            break;
                        }
                        let t = p.start();
                        p.bump_any();
                        t.complete(p, MD_TEXTUAL);
                    }
                    val_content.complete(p, MD_INLINE_ITEM_LIST);
                    val.complete(p, MD_DIRECTIVE_ATTRIBUTE_VALUE);
                }
            }

            attr.complete(p, MD_DIRECTIVE_ATTRIBUTE);
        } else {
            // Unknown char inside braces, skip it
            p.bump_any();
        }
    }
}

// === MDX JSX Elements ===

/// Check if the current position might start an MDX JSX element: `<`.
/// Full validation happens in `try_parse_inline_mdx_jsx` with backtracking.
fn at_inline_mdx_jsx_start(p: &mut MarkdownParser) -> bool {
    p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == "<"
}

/// Try to parse an MDX JSX element: `<Component prop="value" />`
/// Uses try_parse for backtracking.
fn try_parse_inline_mdx_jsx(p: &mut MarkdownParser) -> bool {
    try_parse(p, |p| {
        let m = p.start();

        // Slot 0: `<`
        p.bump_remap(L_ANGLE);

        // Slot 1: name (MdInlineItemList)
        // Name stops at whitespace, >, /, or non-name characters.
        let name = p.start();
        let mut has_name = false;
        while !p.at(T![EOF]) && !p.has_preceding_line_break() && p.cur() == MD_TEXTUAL_LITERAL {
            // Stop at whitespace boundary (space between name and attributes)
            if has_name && p.has_preceding_whitespace() {
                break;
            }
            let b = p.cur_text().as_bytes()[0];
            if b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b'_' {
                let t = p.start();
                p.bump_any();
                t.complete(p, MD_TEXTUAL);
                has_name = true;
            } else {
                break;
            }
        }
        name.complete(p, MD_INLINE_ITEM_LIST);

        if !has_name {
            m.abandon(p);
            return Err(());
        }

        // Slot 2: attributes (MdMdxJsxAttributeList)
        let attr_list = p.start();
        parse_mdx_jsx_attributes(p);
        attr_list.complete(p, MD_MDX_JSX_ATTRIBUTE_LIST);

        // Slot 3: optional `/` (self-closing)
        if !p.at(T![EOF])
            && !p.has_preceding_line_break()
            && p.cur() == MD_TEXTUAL_LITERAL
            && p.cur_text() == "/"
        {
            p.bump_remap(SLASH);
        }

        // Slot 4: `>`
        if p.at(T![EOF])
            || p.has_preceding_line_break()
            || p.cur() != MD_TEXTUAL_LITERAL
            || p.cur_text() != ">"
        {
            m.abandon(p);
            return Err(());
        }
        p.bump_remap(R_ANGLE);

        m.complete(p, MD_MDX_JSX_ELEMENT);
        Ok(())
    })
    .is_ok()
}

/// Parse attributes inside an MDX JSX element (between the tag name and `/>` or `>`).
fn parse_mdx_jsx_attributes(p: &mut MarkdownParser) {
    while !p.at(T![EOF]) && !p.has_preceding_line_break() && p.cur() == MD_TEXTUAL_LITERAL {
        let text = p.cur_text();
        if text == ">" || text == "/" {
            break;
        }

        let ch = text.as_bytes()[0];

        if ch.is_ascii_alphabetic() || ch == b'_' {
            // Attribute: name, name=value, name="value", name={expr}
            let attr = p.start();
            let attr_name = p.start();
            let mut has_attr_name = false;
            while !p.at(T![EOF])
                && !p.has_preceding_line_break()
                && p.cur() == MD_TEXTUAL_LITERAL
            {
                // Stop at whitespace boundary within attribute name
                if has_attr_name && p.has_preceding_whitespace() {
                    break;
                }
                let b = p.cur_text().as_bytes()[0];
                if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b':' {
                    let t = p.start();
                    p.bump_any();
                    t.complete(p, MD_TEXTUAL);
                    has_attr_name = true;
                } else {
                    break;
                }
            }
            attr_name.complete(p, MD_INLINE_ITEM_LIST);

            // Check for `=`
            if !p.at(T![EOF])
                && !p.has_preceding_line_break()
                && p.cur() == MD_TEXTUAL_LITERAL
                && p.cur_text() == "="
            {
                p.bump_remap(EQ);

                // Quoted value or expression value
                if !p.at(T![EOF])
                    && !p.has_preceding_line_break()
                    && p.cur() == MD_TEXTUAL_LITERAL
                {
                    let val_text = p.cur_text().to_string();
                    if val_text == "\"" || val_text == "'" {
                        // Quoted value
                        let val = p.start();
                        p.bump_any(); // opening quote (delimiter)
                        let val_content = p.start();
                        while !p.at(T![EOF])
                            && !p.has_preceding_line_break()
                            && p.cur() == MD_TEXTUAL_LITERAL
                            && p.cur_text() != val_text
                        {
                            let t = p.start();
                            p.bump_any();
                            t.complete(p, MD_TEXTUAL);
                        }
                        val_content.complete(p, MD_INLINE_ITEM_LIST);
                        if !p.at(T![EOF])
                            && !p.has_preceding_line_break()
                            && p.cur() == MD_TEXTUAL_LITERAL
                            && p.cur_text() == val_text
                        {
                            p.bump_any(); // closing quote
                        }
                        val.complete(p, MD_MDX_JSX_ATTRIBUTE_VALUE);
                    } else if val_text == "{" {
                        // Expression value: {expr}
                        let val = p.start();
                        p.bump_any(); // opening {
                        let val_content = p.start();
                        let mut depth = 1;
                        while !p.at(T![EOF]) && !p.has_preceding_line_break() && depth > 0 {
                            if p.cur() == MD_TEXTUAL_LITERAL {
                                if p.cur_text() == "{" {
                                    depth += 1;
                                } else if p.cur_text() == "}" {
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                }
                            }
                            let t = p.start();
                            p.bump_any();
                            t.complete(p, MD_TEXTUAL);
                        }
                        val_content.complete(p, MD_INLINE_ITEM_LIST);
                        if !p.at(T![EOF])
                            && !p.has_preceding_line_break()
                            && p.cur() == MD_TEXTUAL_LITERAL
                            && p.cur_text() == "}"
                        {
                            p.bump_any(); // closing }
                        }
                        val.complete(p, MD_MDX_JSX_ATTRIBUTE_VALUE);
                    }
                }
            }

            attr.complete(p, MD_MDX_JSX_ATTRIBUTE);
        } else {
            // Unknown char, skip
            p.bump_any();
        }
    }
}

// === GFM Tables ===

/// Quick check: the first token on the current line is a pipe `|`.
/// This is a cheap pre-check before attempting full table parsing.
fn at_table_start(p: &mut MarkdownParser) -> bool {
    p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == "|"
}

/// Try to parse a GFM table. Returns true if a table was successfully parsed.
/// A table requires: header row, separator row (pipes + hyphens), then zero or more data rows.
/// Uses try_parse to rewind if the second line isn't a valid separator.
fn try_parse_table(p: &mut MarkdownParser) -> bool {
    try_parse(p, |p| {
        let m = p.start();

        // Slot 0: header row (MdTableRow)
        parse_table_row(p);

        // After header row, we must be at a new line
        if p.at(T![EOF]) || !p.has_preceding_line_break() {
            m.abandon(p);
            return Err(());
        }

        // Check if the next line is a separator row (pipes, hyphens, colons).
        // is_separator_line always rewinds so the separator line is still available for parsing.
        if !is_separator_line(p) {
            m.abandon(p);
            return Err(());
        }

        // Slot 1: separator row (MdTableRow) — parse it as a regular row
        parse_table_row(p);

        // Slot 2: data rows (MdTableRowList)
        let rows = p.start();
        while !p.at(T![EOF]) && p.has_preceding_line_break() && !p.has_preceding_blank_line() {
            // Stop if this line doesn't look like a table row (must contain a pipe)
            if !line_has_pipe(p) {
                break;
            }
            parse_table_row(p);
        }
        rows.complete(p, MD_TABLE_ROW_LIST);

        m.complete(p, MD_TABLE);
        Ok(())
    })
    .is_ok()
}

/// Check if the current line is a table separator row (non-destructive, always rewinds).
/// A separator line consists only of pipes `|`, hyphens `-`, colons `:`, and whitespace (trivia).
/// Must have at least one pipe and at least one hyphen.
fn is_separator_line(p: &mut MarkdownParser) -> bool {
    let mut is_valid = false;
    let _ = try_parse(p, |p| {
        let mut has_pipe = false;
        let mut has_hyphen = false;
        let mut first = true;

        while !p.at(T![EOF]) && (first || !p.has_preceding_line_break()) {
            first = false;
            if p.cur() == MD_TEXTUAL_LITERAL {
                match p.cur_text() {
                    "|" => has_pipe = true,
                    "-" => has_hyphen = true,
                    ":" => {}
                    _ => return Err::<(), ()>(()),
                }
            }
            p.bump_any();
        }

        is_valid = has_pipe && has_hyphen;
        Err::<(), ()>(()) // Always rewind — this is a lookahead check
    });
    is_valid
}

/// Check (non-destructively, always rewinds) if the current line contains a pipe character.
fn line_has_pipe(p: &mut MarkdownParser) -> bool {
    let mut found = false;
    let _ = try_parse(p, |p| {
        let mut first = true;
        while !p.at(T![EOF]) && (first || !p.has_preceding_line_break()) {
            first = false;
            if p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == "|" {
                found = true;
                return Err::<(), ()>(());
            }
            p.bump_any();
        }
        Err::<(), ()>(())
    });
    found
}

/// Parse a single table row as a flat inline content list.
/// MdTableRow has 1 slot: content (MdInlineItemList).
/// All tokens (pipes, text, etc.) become MdTextual items in the list.
fn parse_table_row(p: &mut MarkdownParser) {
    let row = p.start();
    let content = p.start();
    let mut first = true;

    while !p.at(T![EOF]) && (first || !p.has_preceding_line_break()) {
        first = false;
        let textual = p.start();
        p.bump_any();
        textual.complete(p, MD_TEXTUAL);
    }

    content.complete(p, MD_INLINE_ITEM_LIST);
    row.complete(p, MD_TABLE_ROW);
}

// === Link Reference Definitions ===

/// Quick check: the current token is `[` with at most 3 spaces of indentation.
fn at_link_definition(p: &mut MarkdownParser) -> bool {
    p.cur() == MD_TEXTUAL_LITERAL
        && p.cur_text() == "["
        && p.before_whitespace_count() <= 3
}

/// Non-destructive lookahead: check if the current line matches `[label]:`.
/// Always rewinds regardless of result.
fn is_link_definition_line(p: &mut MarkdownParser) -> bool {
    let mut found = false;
    let _ = try_parse(p, |p| {
        // Skip past [
        p.bump_any();
        let mut first = true;
        // Find ] on same line
        while !p.at(T![EOF]) && (first || !p.has_preceding_line_break()) {
            first = false;
            if p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == "]" {
                p.bump_any();
                // Must be followed by :
                if !p.at(T![EOF])
                    && !p.has_preceding_line_break()
                    && p.cur() == MD_TEXTUAL_LITERAL
                    && p.cur_text() == ":"
                {
                    found = true;
                }
                break;
            }
            p.bump_any();
        }
        Err::<(), ()>(()) // Always rewind
    });
    found
}

/// Try to parse a link reference definition: `[label]: url "title"`.
/// Returns true if successfully parsed into an MdLinkBlock node.
/// MdLinkBlock has 5 slots: `[` label `]` `:` url
fn try_parse_link_definition(p: &mut MarkdownParser) -> bool {
    try_parse(p, |p| {
        if !is_link_definition_line(p) {
            return Err(());
        }
        let m = p.start();

        // Slot 0: [
        p.bump_remap(L_BRACK);

        // Slot 1: label (MdInlineItemList) — tokens until ]
        let label = p.start();
        while !p.at(T![EOF]) && !p.has_preceding_line_break() {
            if p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == "]" {
                break;
            }
            let textual = p.start();
            p.bump_any();
            textual.complete(p, MD_TEXTUAL);
        }
        label.complete(p, MD_INLINE_ITEM_LIST);

        // Slot 2: ]
        if p.at(T![EOF]) || p.has_preceding_line_break() || p.cur_text() != "]" {
            m.abandon(p);
            return Err(());
        }
        p.bump_remap(R_BRACK);

        // Slot 3: :
        if p.at(T![EOF]) || p.has_preceding_line_break() || p.cur_text() != ":" {
            m.abandon(p);
            return Err(());
        }
        p.bump_remap(COLON);

        // Slot 4: url (MdInlineItemList) — rest of line
        let url = p.start();
        while !p.at(T![EOF]) && !p.has_preceding_line_break() {
            let textual = p.start();
            p.bump_any();
            textual.complete(p, MD_TEXTUAL);
        }
        url.complete(p, MD_INLINE_ITEM_LIST);

        m.complete(p, MD_LINK_BLOCK);
        Ok(())
    })
    .is_ok()
}

// === Setext Headers ===

/// Check if the current token is a setext-style underline (a line of `=` or `-` characters).
/// Per CommonMark, a setext underline must immediately follow the paragraph with no blank line.
fn at_setext_underline(p: &mut MarkdownParser) -> bool {
    if !p.has_preceding_line_break() || p.at(T![EOF]) {
        return false;
    }
    // Reject if there's a blank line between paragraph and underline
    if p.has_preceding_blank_line() {
        return false;
    }
    // `===` is tokenized as MD_TEXTUAL_LITERAL (combined by lexer)
    if p.cur() == MD_TEXTUAL_LITERAL {
        let text = p.cur_text();
        return text.len() >= 1 && text.chars().all(|c| c == '=');
    }
    // `---` is tokenized as MD_THEMATIC_BREAK_LITERAL
    if p.cur() == MD_THEMATIC_BREAK_LITERAL {
        let text = p.cur_text();
        return text.chars().all(|c| c == '-' || c == ' ');
    }
    false
}

/// If the next line is a setext underline, wrap the paragraph in MdSetextHeader.
fn maybe_wrap_setext_header(p: &mut MarkdownParser, para: biome_parser::CompletedMarker) {
    if !at_setext_underline(p) {
        return;
    }
    // Wrap the already-completed paragraph in a setext header
    let m = para.precede(p);

    // Slot 1: underline wrapped in MdTextual
    let textual = p.start();
    // Remap both MD_TEXTUAL_LITERAL and MD_THEMATIC_BREAK_LITERAL to MD_TEXTUAL_LITERAL
    p.bump_remap(MD_TEXTUAL_LITERAL);
    textual.complete(p, MD_TEXTUAL);

    // Consume any remaining tokens on the underline line (they become trailing trivia-like)
    while !p.at(T![EOF]) && !p.has_preceding_line_break() {
        p.bump_any();
    }
    m.complete(p, MD_SETEXT_HEADER);
}

// === HTML Blocks ===

/// Check if the current position starts an HTML block.
/// Detects `<` at line start (with ≤3 spaces indent) as the beginning of an HTML block.
/// Excludes JSX-style tags (uppercase first letter after `<`) so they fall through to
/// paragraph parsing where the inline MDX JSX parser produces proper AST nodes.
fn at_html_block(p: &mut MarkdownParser) -> bool {
    if p.cur() != MD_TEXTUAL_LITERAL || p.cur_text() != "<" || p.before_whitespace_count() > 3 {
        return false;
    }
    // Check the character immediately after `<` in the source text.
    // If it's uppercase, this is likely a JSX component tag — let paragraph parser handle it.
    if let Some(next_byte) = p.peek_next_byte() {
        if next_byte.is_ascii_uppercase() {
            return false;
        }
    }
    true
}

/// Parse an HTML block: content starting with `<` until a blank line or EOF.
/// HTML blocks are preserved verbatim — no inline parsing of content.
fn parse_html_block(p: &mut MarkdownParser) {
    let m = p.start();
    let content = p.start();
    let mut first = true;

    while !p.at(T![EOF]) {
        if !first && p.has_preceding_line_break() {
            // Stop at blank lines (two consecutive line breaks)
            if p.has_preceding_blank_line() {
                break;
            }
        }
        first = false;
        let textual = p.start();
        p.bump_any();
        textual.complete(p, MD_TEXTUAL);
    }

    content.complete(p, MD_INLINE_ITEM_LIST);
    m.complete(p, MD_HTML_BLOCK);
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
