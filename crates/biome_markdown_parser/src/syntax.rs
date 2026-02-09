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
/// MdQuote expects exactly 1 child: AnyMdBlock (we use MdParagraph).
fn parse_blockquote(p: &mut MarkdownParser) {
    let m = p.start();

    // Consume the `>` marker (becomes part of the paragraph content)
    // Build a paragraph containing the `>` and everything on this line
    let para = p.start();
    let list = p.start();

    let textual = p.start();
    p.bump_any();
    textual.complete(p, MD_TEXTUAL);

    while !p.at(T![EOF]) && !p.has_preceding_line_break() {
        let textual = p.start();
        p.bump_any();
        textual.complete(p, MD_TEXTUAL);
    }

    list.complete(p, MD_INLINE_ITEM_LIST);
    para.complete(p, MD_PARAGRAPH);

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
fn parse_unordered_list_item(p: &mut MarkdownParser) {
    let m = p.start();
    let list = p.start();

    // Parse consecutive bullet items
    parse_bullet(p);
    while !p.at(T![EOF]) && at_bullet_start(p) {
        parse_bullet(p);
    }

    list.complete(p, MD_BULLET_LIST);
    m.complete(p, MD_BULLET_LIST_ITEM);
}

/// Check if the current token is a bullet marker at the start of a new line.
fn at_bullet_start(p: &mut MarkdownParser) -> bool {
    if !p.has_preceding_line_break() {
        return false;
    }
    if p.cur() != MD_TEXTUAL_LITERAL {
        return false;
    }
    let text = p.cur_text();
    (text == "-" || text == "*" || text == "+") && p.before_whitespace_count() < 4
}

/// Parse a single bullet: marker + content on the same line.
/// MdBullet has 2 slots: bullet token (MINUS/STAR/PLUS) and content (MdInlineItemList).
fn parse_bullet(p: &mut MarkdownParser) {
    let m = p.start();

    // Slot 0: bullet marker — remap to the proper token kind
    let marker = p.cur_text().to_string();
    match marker.as_str() {
        "-" => p.bump_remap(MINUS),
        "*" => p.bump_remap(STAR),
        "+" => p.bump_remap(PLUS),
        _ => p.bump_any(),
    }

    // Slot 1: content — everything on the line after the marker
    parse_inline_list(p);

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
fn parse_ordered_list_item(p: &mut MarkdownParser) {
    let m = p.start();
    let list = p.start();

    // Parse consecutive ordered items
    parse_order_bullet(p);
    while !p.at(T![EOF]) && at_order_bullet_start(p) {
        parse_order_bullet(p);
    }

    list.complete(p, MD_ORDER_LIST);
    m.complete(p, MD_ORDER_LIST_ITEM);
}

/// Check if the current token is an ordered list marker at the start of a new line.
fn at_order_bullet_start(p: &mut MarkdownParser) -> bool {
    if !p.has_preceding_line_break() {
        return false;
    }
    if p.cur() != MD_TEXTUAL_LITERAL {
        return false;
    }
    is_ordered_marker(p.cur_text()) && p.before_whitespace_count() < 4
}

/// Parse a single ordered bullet: marker + content on the same line.
/// MdOrderBullet has 2 slots: marker (MD_TEXTUAL_LITERAL) and content (MdInlineItemList).
fn parse_order_bullet(p: &mut MarkdownParser) {
    let m = p.start();

    // Slot 0: marker (e.g. "1.", "2)") — keep as MD_TEXTUAL_LITERAL
    p.bump_any();

    // Slot 1: content — everything on the line after the marker
    parse_inline_list(p);

    m.complete(p, MD_ORDER_BULLET);
}

pub(crate) fn at_indent_code_block(p: &mut MarkdownParser) -> bool {
    p.before_whitespace_count() > 4
}

pub(crate) fn parse_indent_code_block(p: &mut MarkdownParser) {
    // Stub: treat as paragraph
    let _para = parse_paragraph(p);
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
        if at_inline_code(p) {
            parse_inline_code(p);
        } else if at_inline_image_start(p) {
            if !try_parse_inline_image(p) {
                let textual = p.start();
                p.bump_any();
                textual.complete(p, MD_TEXTUAL);
            }
        } else if at_inline_link_start(p) {
            if !try_parse_inline_link(p) {
                let textual = p.start();
                p.bump_any();
                textual.complete(p, MD_TEXTUAL);
            }
        } else if at_inline_emphasis_start(p) {
            if !try_parse_inline_emphasis_or_italic(p) {
                let textual = p.start();
                p.bump_any();
                textual.complete(p, MD_TEXTUAL);
            }
        } else {
            let textual = p.start();
            p.bump_any();
            textual.complete(p, MD_TEXTUAL);
        }
    }

    list.complete(p, MD_INLINE_ITEM_LIST);
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

        // Slot 1: text content (MdInlineItemList)
        let text = p.start();
        while !p.at(T![EOF]) && !p.has_preceding_line_break() {
            if p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == "]" {
                break;
            }
            let textual = p.start();
            p.bump_any();
            textual.complete(p, MD_TEXTUAL);
        }
        text.complete(p, MD_INLINE_ITEM_LIST);

        // Slot 2: ]
        if p.cur() != MD_TEXTUAL_LITERAL || p.cur_text() != "]" {
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
        while !p.at(T![EOF]) && !p.has_preceding_line_break() {
            if p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == "]" {
                break;
            }
            let textual = p.start();
            p.bump_any();
            textual.complete(p, MD_TEXTUAL);
        }
        alt_content.complete(p, MD_INLINE_ITEM_LIST);
        if p.cur() != MD_TEXTUAL_LITERAL || p.cur_text() != "]" {
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

        // Slot 1: content (MdInlineItemList)
        let content = p.start();
        let mut found_close = false;
        while !p.at(T![EOF]) && !p.has_preceding_line_break() {
            if p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == delimiter {
                found_close = true;
                break;
            }
            let textual = p.start();
            p.bump_any();
            textual.complete(p, MD_TEXTUAL);
        }
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

        // Slot 1: content (MdInlineItemList)
        let content = p.start();
        let mut found_close = false;
        while !p.at(T![EOF]) && !p.has_preceding_line_break() {
            if p.cur() == MD_TEXTUAL_LITERAL && p.cur_text() == delimiter {
                found_close = true;
                break;
            }
            let textual = p.start();
            p.bump_any();
            textual.complete(p, MD_TEXTUAL);
        }
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
