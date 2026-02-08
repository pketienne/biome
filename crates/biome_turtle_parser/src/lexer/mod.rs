//! An extremely fast, lookup table based, Turtle lexer which yields SyntaxKind tokens used by the biome Turtle parser.
#[rustfmt::skip]
mod tests;

use biome_turtle_syntax::{TurtleSyntaxKind, TurtleSyntaxKind::*, T, TextLen, TextSize};
use biome_parser::diagnostic::ParseDiagnostic;
use biome_parser::lexer::{Lexer, LexerCheckpoint, LexerWithCheckpoint, TokenFlags};
use biome_rowan::SyntaxKind;

#[derive(Debug)]
pub struct TurtleLexer<'src> {
    /// Source text
    source: &'src str,

    /// The start byte position in the source text of the next token.
    position: usize,

    /// If the source starts with a Unicode BOM, this is the number of bytes for that token.
    unicode_bom_length: usize,

    /// Byte offset of the current token from the start of the source
    current_start: TextSize,

    /// The kind of the current token
    current_kind: TurtleSyntaxKind,

    /// Flags for the current token
    current_flags: TokenFlags,

    diagnostics: Vec<ParseDiagnostic>,
}

impl<'src> Lexer<'src> for TurtleLexer<'src> {
    const NEWLINE: Self::Kind = NEWLINE;
    const WHITESPACE: Self::Kind = WHITESPACE;

    type Kind = TurtleSyntaxKind;
    type LexContext = ();
    type ReLexContext = ();

    fn source(&self) -> &'src str {
        self.source
    }

    fn current(&self) -> Self::Kind {
        self.current_kind
    }

    fn position(&self) -> usize {
        self.position
    }

    fn current_start(&self) -> TextSize {
        self.current_start
    }

    fn push_diagnostic(&mut self, diagnostic: ParseDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    fn has_unicode_escape(&self) -> bool {
        self.current_flags().has_unicode_escape()
    }

    fn has_preceding_line_break(&self) -> bool {
        self.current_flags().has_preceding_line_break()
    }

    fn consume_newline_or_whitespaces(&mut self) -> Self::Kind {
        if self.consume_newline() {
            self.current_flags
                .set(TokenFlags::PRECEDING_LINE_BREAK, true);
            NEWLINE
        } else {
            self.consume_whitespaces();
            WHITESPACE
        }
    }

    fn next_token(&mut self, _context: Self::LexContext) -> Self::Kind {
        self.current_start = self.text_position();
        self.current_flags = TokenFlags::empty();

        let kind = match self.current_byte() {
            Some(current) => self.consume_token(current),
            None => EOF,
        };

        self.current_kind = kind;

        if !kind.is_trivia() {
            self.current_flags
                .set(TokenFlags::PRECEDING_LINE_BREAK, false);
        }

        kind
    }

    #[inline]
    fn advance_char_unchecked(&mut self) {
        let c = self.current_char_unchecked();
        self.position += c.len_utf8();
    }

    #[inline]
    fn advance(&mut self, n: usize) {
        self.position += n;
    }

    fn finish(self) -> Vec<ParseDiagnostic> {
        self.diagnostics
    }

    fn rewind(&mut self, _checkpoint: LexerCheckpoint<Self::Kind>) {
        unimplemented!("Turtle lexer doesn't support rewinding");
    }
}

impl<'src> LexerWithCheckpoint<'src> for TurtleLexer<'src> {
    fn checkpoint(&self) -> LexerCheckpoint<Self::Kind> {
        LexerCheckpoint {
            position: TextSize::from(self.position as u32),
            current_start: self.current_start,
            current_flags: self.current_flags,
            current_kind: self.current_kind,
            after_line_break: self.has_preceding_line_break(),
            unicode_bom_length: self.unicode_bom_length,
            diagnostics_pos: self.diagnostics.len() as u32,
        }
    }
}

impl<'src> TurtleLexer<'src> {
    /// Make a new lexer from a str, this is safe because strs are valid utf8
    pub fn from_str(source: &'src str) -> Self {
        Self {
            source,
            current_kind: TOMBSTONE,
            current_start: TextSize::from(0),
            current_flags: TokenFlags::empty(),
            position: 0,
            diagnostics: vec![],
            unicode_bom_length: 0,
        }
    }

    /// Bumps the current byte and creates a lexed token of the passed in kind
    fn consume_byte(&mut self, tok: TurtleSyntaxKind) -> TurtleSyntaxKind {
        self.advance(1);
        tok
    }

    /// Lexes the next token
    ///
    /// Guaranteed to not be at the end of the file
    fn consume_token(&mut self, current: u8) -> TurtleSyntaxKind {
        match current {
            b'.' => self.consume_dot(),
            b';' => self.consume_byte(T![;]),
            b',' => self.consume_byte(T![,]),
            b'[' => self.consume_bracket_or_anon(),
            b']' => self.consume_byte(T![']']),
            b'(' => self.consume_byte(T!['(']),
            b')' => self.consume_byte(T![')']),
            b'^' => self.consume_caret(),
            b'<' => self.consume_iriref(),
            b'"' => self.consume_string(b'"'),
            b'\'' => self.consume_string(b'\''),
            b'#' => self.consume_comment(),
            b'@' => self.consume_at(),
            b'_' => self.consume_blank_node_label(),
            b':' => self.consume_pname_ns_bare(),
            b'\n' | b'\r' | b'\t' | b' ' => self.consume_newline_or_whitespaces(),
            b'+' | b'-' => self.consume_signed_number(),
            b'0'..=b'9' => self.consume_number(),
            _ if current.is_ascii_alphabetic() => self.consume_name(current),
            _ if self.position == 0 => {
                if let Some((bom, bom_size)) = self.consume_potential_bom(UNICODE_BOM) {
                    self.unicode_bom_length = bom_size;
                    return bom;
                }
                self.consume_unexpected_character()
            }
            _ => self.consume_unexpected_character(),
        }
    }

    /// Consume a dot: either statement terminator or start of decimal number
    fn consume_dot(&mut self) -> TurtleSyntaxKind {
        self.assert_byte(b'.');
        self.advance(1);

        // Check if followed by a digit — part of a decimal/double
        if let Some(b'0'..=b'9') = self.current_byte() {
            // .123 form — consume as decimal
            self.consume_digits();
            // Check for exponent
            if let Some(b'e' | b'E') = self.current_byte() {
                self.advance(1);
                if let Some(b'+' | b'-') = self.current_byte() {
                    self.advance(1);
                }
                self.consume_digits();
                return TURTLE_DOUBLE_LITERAL;
            }
            return TURTLE_DECIMAL_LITERAL;
        }

        T![.]
    }

    /// Consume `[` — either L_BRACK or ANON token `[]`
    fn consume_bracket_or_anon(&mut self) -> TurtleSyntaxKind {
        self.assert_byte(b'[');
        self.advance(1);

        // Save position and skip whitespace to check for `]`
        let saved_pos = self.position;
        self.skip_whitespace_for_anon();

        if self.current_byte() == Some(b']') {
            self.advance(1);
            return TURTLE_ANON_TOKEN;
        }

        // Not ANON, restore position to after `[`
        self.position = saved_pos;
        T!['[']
    }

    /// Skip whitespace (including newlines) for ANON detection
    fn skip_whitespace_for_anon(&mut self) {
        while let Some(b) = self.current_byte() {
            match b {
                b' ' | b'\t' | b'\n' | b'\r' => self.advance(1),
                _ => break,
            }
        }
    }

    /// Consume `^^` — datatype annotation
    fn consume_caret(&mut self) -> TurtleSyntaxKind {
        self.assert_byte(b'^');
        self.advance(1);
        if self.current_byte() == Some(b'^') {
            self.advance(1);
            T!["^^"]
        } else {
            let err = ParseDiagnostic::new(
                "unexpected character `^`, expected `^^`",
                self.current_start..self.text_position(),
            );
            self.diagnostics.push(err);
            ERROR_TOKEN
        }
    }

    /// Consume IRIREF: `<` ... `>`
    fn consume_iriref(&mut self) -> TurtleSyntaxKind {
        self.assert_byte(b'<');
        let start = self.text_position();
        self.advance(1);

        loop {
            match self.current_byte() {
                Some(b'>') => {
                    self.advance(1);
                    return TURTLE_IRIREF_LITERAL;
                }
                Some(b'\\') => {
                    // Unicode escape in IRI: \uXXXX or \UXXXXXXXX
                    self.advance(1);
                    match self.current_byte() {
                        Some(b'u') => {
                            self.advance(1);
                            for _ in 0..4 {
                                match self.current_byte() {
                                    Some(b) if b.is_ascii_hexdigit() => self.advance(1),
                                    _ => {
                                        self.diagnostics.push(ParseDiagnostic::new(
                                            "Invalid unicode escape in IRI",
                                            start..self.text_position(),
                                        ));
                                        return ERROR_TOKEN;
                                    }
                                }
                            }
                        }
                        Some(b'U') => {
                            self.advance(1);
                            for _ in 0..8 {
                                match self.current_byte() {
                                    Some(b) if b.is_ascii_hexdigit() => self.advance(1),
                                    _ => {
                                        self.diagnostics.push(ParseDiagnostic::new(
                                            "Invalid unicode escape in IRI",
                                            start..self.text_position(),
                                        ));
                                        return ERROR_TOKEN;
                                    }
                                }
                            }
                        }
                        _ => {
                            self.diagnostics.push(ParseDiagnostic::new(
                                "Invalid escape in IRI, expected \\uXXXX or \\UXXXXXXXX",
                                start..self.text_position(),
                            ));
                            return ERROR_TOKEN;
                        }
                    }
                }
                Some(b) if b <= 0x20 || matches!(b, b'<' | b'"' | b'{' | b'}' | b'|' | b'`') => {
                    self.diagnostics.push(ParseDiagnostic::new(
                        "Invalid character in IRI",
                        start..self.text_position(),
                    ));
                    return ERROR_TOKEN;
                }
                Some(_) => {
                    self.advance_char_unchecked();
                }
                None => {
                    self.diagnostics.push(
                        ParseDiagnostic::new("Unterminated IRI", start..self.text_position())
                            .with_detail(
                                self.source.text_len()..self.source.text_len(),
                                "file ends here",
                            ),
                    );
                    return ERROR_TOKEN;
                }
            }
        }
    }

    /// Consume string literal (single or double, short or long)
    fn consume_string(&mut self, quote: u8) -> TurtleSyntaxKind {
        let start = self.text_position();
        self.advance(1);

        // Check for long string (triple quotes)
        if self.current_byte() == Some(quote) {
            self.advance(1);
            if self.current_byte() == Some(quote) {
                self.advance(1);
                // Long string
                return self.consume_long_string(quote, start);
            }
            // Empty short string
            return if quote == b'"' {
                TURTLE_STRING_LITERAL_QUOTE
            } else {
                TURTLE_STRING_LITERAL_SINGLE_QUOTE
            };
        }

        // Short string
        self.consume_short_string(quote, start)
    }

    fn consume_short_string(&mut self, quote: u8, start: TextSize) -> TurtleSyntaxKind {
        loop {
            match self.current_byte() {
                Some(b) if b == quote => {
                    self.advance(1);
                    return if quote == b'"' {
                        TURTLE_STRING_LITERAL_QUOTE
                    } else {
                        TURTLE_STRING_LITERAL_SINGLE_QUOTE
                    };
                }
                Some(b'\\') => {
                    self.advance(1);
                    self.consume_string_escape();
                }
                Some(b'\n') | Some(b'\r') | None => {
                    self.diagnostics.push(ParseDiagnostic::new(
                        "Unterminated string literal",
                        start..self.text_position(),
                    ));
                    return ERROR_TOKEN;
                }
                Some(_) => {
                    self.advance_char_unchecked();
                }
            }
        }
    }

    fn consume_long_string(&mut self, quote: u8, start: TextSize) -> TurtleSyntaxKind {
        loop {
            match self.current_byte() {
                Some(b) if b == quote => {
                    self.advance(1);
                    if self.current_byte() == Some(quote) {
                        self.advance(1);
                        if self.current_byte() == Some(quote) {
                            self.advance(1);
                            return if quote == b'"' {
                                TURTLE_STRING_LITERAL_LONG_QUOTE
                            } else {
                                TURTLE_STRING_LITERAL_LONG_SINGLE_QUOTE
                            };
                        }
                    }
                }
                Some(b'\\') => {
                    self.advance(1);
                    self.consume_string_escape();
                }
                Some(_) => {
                    self.advance_char_unchecked();
                }
                None => {
                    self.diagnostics.push(
                        ParseDiagnostic::new(
                            "Unterminated long string literal",
                            start..self.text_position(),
                        )
                        .with_detail(
                            self.source.text_len()..self.source.text_len(),
                            "file ends here",
                        ),
                    );
                    return ERROR_TOKEN;
                }
            }
        }
    }

    fn consume_string_escape(&mut self) {
        match self.current_byte() {
            Some(b't' | b'b' | b'n' | b'r' | b'f' | b'\\' | b'\'' | b'"') => {
                self.advance(1);
            }
            Some(b'u') => {
                self.advance(1);
                for _ in 0..4 {
                    if let Some(b) = self.current_byte() {
                        if b.is_ascii_hexdigit() {
                            self.advance(1);
                        }
                    }
                }
            }
            Some(b'U') => {
                self.advance(1);
                for _ in 0..8 {
                    if let Some(b) = self.current_byte() {
                        if b.is_ascii_hexdigit() {
                            self.advance(1);
                        }
                    }
                }
            }
            _ => {
                // Invalid escape — the parser will catch this
            }
        }
    }

    /// Consume `#` comment to end of line
    fn consume_comment(&mut self) -> TurtleSyntaxKind {
        self.assert_byte(b'#');
        self.advance(1);
        while let Some(chr) = self.current_byte() {
            match chr {
                b'\n' | b'\r' => return COMMENT,
                chr => self.advance_byte_or_char(chr),
            }
        }
        COMMENT
    }

    /// Consume `@` — either @prefix, @base, or a language tag
    fn consume_at(&mut self) -> TurtleSyntaxKind {
        self.assert_byte(b'@');
        self.advance(1);

        // Try to match @prefix or @base keywords
        let remaining = &self.source[self.position..];
        if remaining.starts_with("prefix") && self.is_keyword_boundary(self.position + 6) {
            self.advance(6);
            return PREFIX_KW;
        }
        if remaining.starts_with("base") && self.is_keyword_boundary(self.position + 4) {
            self.advance(4);
            return BASE_KW;
        }

        // Otherwise it's a language tag: @[a-zA-Z]+ ('-' [a-zA-Z0-9]+)*
        if !self
            .current_byte()
            .is_some_and(|b| b.is_ascii_alphabetic())
        {
            let err = ParseDiagnostic::new(
                "Expected language tag after `@`",
                self.current_start..self.text_position(),
            );
            self.diagnostics.push(err);
            return ERROR_TOKEN;
        }

        while self
            .current_byte()
            .is_some_and(|b| b.is_ascii_alphabetic())
        {
            self.advance(1);
        }

        while self.current_byte() == Some(b'-') {
            self.advance(1);
            if !self
                .current_byte()
                .is_some_and(|b| b.is_ascii_alphanumeric())
            {
                break;
            }
            while self
                .current_byte()
                .is_some_and(|b| b.is_ascii_alphanumeric())
            {
                self.advance(1);
            }
        }

        TURTLE_LANGTAG_LITERAL
    }

    /// Check if position is at a keyword boundary (not followed by name char)
    fn is_keyword_boundary(&self, pos: usize) -> bool {
        if pos >= self.source.len() {
            return true;
        }
        let b = self.source.as_bytes()[pos];
        !b.is_ascii_alphanumeric() && b != b'_'
    }

    /// Consume `_:` prefix for blank node labels
    fn consume_blank_node_label(&mut self) -> TurtleSyntaxKind {
        self.assert_byte(b'_');
        let start = self.text_position();

        if self.byte_at(1) == Some(b':') {
            self.advance(2); // skip _:

            // First char must be PN_CHARS_U or digit
            match self.current_byte() {
                Some(b) if b.is_ascii_alphanumeric() || b == b'_' => {
                    self.advance(1);
                }
                Some(b) if b > 0x7F => {
                    // Non-ASCII char — likely valid unicode PN_CHARS_BASE
                    self.advance_char_unchecked();
                }
                _ => {
                    self.diagnostics.push(ParseDiagnostic::new(
                        "Expected character after `_:`",
                        start..self.text_position(),
                    ));
                    return ERROR_TOKEN;
                }
            }

            // Continue with PN_CHARS | '.'
            // But the last char must not be '.'
            loop {
                match self.current_byte() {
                    Some(b)
                        if b.is_ascii_alphanumeric()
                            || matches!(b, b'_' | b'-' | b'\xB7') =>
                    {
                        self.advance(1);
                    }
                    Some(b'.') => {
                        // Dot is allowed inside but not at end
                        // Peek ahead: if next char is a valid PN_CHARS, include the dot
                        if self.byte_at(1).is_some_and(|b| {
                            b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-')
                        }) {
                            self.advance(1);
                        } else {
                            break;
                        }
                    }
                    Some(b) if b > 0x7F => {
                        self.advance_char_unchecked();
                    }
                    _ => break,
                }
            }

            return TURTLE_BLANK_NODE_LABEL_LITERAL;
        }

        // Just `_` followed by something else — treat as unexpected
        self.advance(1);
        self.diagnostics.push(ParseDiagnostic::new(
            "Expected `:` after `_` for blank node label",
            start..self.text_position(),
        ));
        ERROR_TOKEN
    }

    /// Consume a bare `:` as PNAME_NS (empty prefix)
    fn consume_pname_ns_bare(&mut self) -> TurtleSyntaxKind {
        self.assert_byte(b':');
        self.advance(1);

        // Check if followed by local name chars
        if self
            .current_byte()
            .is_some_and(|b| is_pn_local_start(b) || b == b'%' || b == b'\\')
        {
            self.consume_pn_local();
            return TURTLE_PNAME_LN_LITERAL;
        }

        // Just `:` — empty PNAME_NS
        TURTLE_PNAME_NS_LITERAL
    }

    /// Consume a name starting with an alphabetic char
    /// Could be: keyword (a, true, false, BASE, PREFIX), PNAME_NS, PNAME_LN
    fn consume_name(&mut self, _first: u8) -> TurtleSyntaxKind {
        self.assert_current_char_boundary();

        // Collect prefix part
        let name_start = self.position;
        self.advance(1);

        while let Some(b) = self.current_byte() {
            if is_pn_chars(b) || b == b'.' {
                // Dot is allowed in PN_PREFIX but not at end
                if b == b'.' {
                    if self
                        .byte_at(1)
                        .is_some_and(|next| is_pn_chars(next) || next == b'.')
                    {
                        self.advance(1);
                    } else {
                        break;
                    }
                } else {
                    self.advance(1);
                }
            } else if b > 0x7F {
                self.advance_char_unchecked();
            } else {
                break;
            }
        }

        // Check what follows
        if self.current_byte() == Some(b':') {
            self.advance(1); // consume the ':'

            // Check if it's a known keyword followed by a colon — that's a PNAME
            // Check if followed by local name
            if self
                .current_byte()
                .is_some_and(|b| is_pn_local_start(b) || b == b'%' || b == b'\\')
            {
                self.consume_pn_local();
                return TURTLE_PNAME_LN_LITERAL;
            }

            // Just prefix: — PNAME_NS
            return TURTLE_PNAME_NS_LITERAL;
        }

        // It's a bare name — check for keywords
        let name = &self.source[name_start..self.position];
        match name {
            "a" => A_KW,
            "true" => TRUE_KW,
            "false" => FALSE_KW,
            _ => {
                // Case-insensitive check for SPARQL keywords
                let upper = name.to_ascii_uppercase();
                match upper.as_str() {
                    "BASE" if name.chars().any(|c| c.is_uppercase()) => SPARQL_BASE_KW,
                    "PREFIX" if name.chars().any(|c| c.is_uppercase()) => SPARQL_PREFIX_KW,
                    _ => {
                        // Not a keyword and no colon — error
                        let err = ParseDiagnostic::new(
                            format!("Unexpected name `{name}`"),
                            self.current_start..self.text_position(),
                        );
                        self.diagnostics.push(err);
                        ERROR_TOKEN
                    }
                }
            }
        }
    }

    /// Consume a prefixed name local part (after the colon)
    fn consume_pn_local(&mut self) {
        // First char: PN_CHARS_U | ':' | digit | PLX
        match self.current_byte() {
            Some(b'%') => {
                self.consume_percent_encode();
            }
            Some(b'\\') => {
                self.advance(1);
                if self.current_byte().is_some() {
                    self.advance(1);
                }
            }
            Some(b) if is_pn_local_start(b) => {
                self.advance(1);
            }
            Some(b) if b > 0x7F => {
                self.advance_char_unchecked();
            }
            _ => return,
        }

        // Rest: PN_CHARS | '.' | ':' | PLX
        loop {
            match self.current_byte() {
                Some(b)
                    if b.is_ascii_alphanumeric()
                        || matches!(b, b'_' | b'-' | b':' | b'\xB7') =>
                {
                    self.advance(1);
                }
                Some(b'.') => {
                    // Dot allowed inside but not at end
                    if self.byte_at(1).is_some_and(|next| {
                        is_pn_chars(next) || matches!(next, b':' | b'.' | b'%' | b'\\')
                    }) {
                        self.advance(1);
                    } else {
                        break;
                    }
                }
                Some(b'%') => {
                    self.consume_percent_encode();
                }
                Some(b'\\') => {
                    self.advance(1);
                    if self.current_byte().is_some() {
                        self.advance(1);
                    }
                }
                Some(b) if b > 0x7F => {
                    self.advance_char_unchecked();
                }
                _ => break,
            }
        }
    }

    /// Consume a percent-encoded char (%XX)
    fn consume_percent_encode(&mut self) {
        self.advance(1); // skip %
        for _ in 0..2 {
            if let Some(b) = self.current_byte() {
                if b.is_ascii_hexdigit() {
                    self.advance(1);
                }
            }
        }
    }

    /// Consume a number starting with a sign
    fn consume_signed_number(&mut self) -> TurtleSyntaxKind {
        let start = self.text_position();
        self.advance(1); // skip + or -

        match self.current_byte() {
            Some(b'.') => {
                self.advance(1);
                if self
                    .current_byte()
                    .is_some_and(|b| b.is_ascii_digit())
                {
                    self.consume_digits();
                    if let Some(b'e' | b'E') = self.current_byte() {
                        self.advance(1);
                        if let Some(b'+' | b'-') = self.current_byte() {
                            self.advance(1);
                        }
                        self.consume_digits();
                        return TURTLE_DOUBLE_LITERAL;
                    }
                    return TURTLE_DECIMAL_LITERAL;
                }
                self.diagnostics.push(ParseDiagnostic::new(
                    "Expected digit after sign and decimal point",
                    start..self.text_position(),
                ));
                ERROR_TOKEN
            }
            Some(b) if b.is_ascii_digit() => self.consume_number(),
            _ => {
                self.diagnostics.push(ParseDiagnostic::new(
                    "Expected digit after sign",
                    start..self.text_position(),
                ));
                ERROR_TOKEN
            }
        }
    }

    /// Consume a number (integer, decimal, or double)
    fn consume_number(&mut self) -> TurtleSyntaxKind {
        self.consume_digits();

        match self.current_byte() {
            Some(b'.') => {
                // Could be decimal or double
                if self
                    .byte_at(1)
                    .is_some_and(|b| b.is_ascii_digit())
                {
                    self.advance(1); // skip .
                    self.consume_digits();
                    if let Some(b'e' | b'E') = self.current_byte() {
                        self.advance(1);
                        if let Some(b'+' | b'-') = self.current_byte() {
                            self.advance(1);
                        }
                        self.consume_digits();
                        return TURTLE_DOUBLE_LITERAL;
                    }
                    TURTLE_DECIMAL_LITERAL
                } else {
                    // Dot is statement terminator
                    TURTLE_INTEGER_LITERAL
                }
            }
            Some(b'e' | b'E') => {
                self.advance(1);
                if let Some(b'+' | b'-') = self.current_byte() {
                    self.advance(1);
                }
                self.consume_digits();
                TURTLE_DOUBLE_LITERAL
            }
            _ => TURTLE_INTEGER_LITERAL,
        }
    }

    fn consume_digits(&mut self) {
        while self
            .current_byte()
            .is_some_and(|b| b.is_ascii_digit())
        {
            self.advance(1);
        }
    }

    #[inline]
    fn consume_unexpected_character(&mut self) -> TurtleSyntaxKind {
        self.assert_current_char_boundary();
        let char = self.current_char_unchecked();
        let err = ParseDiagnostic::new(
            format!("unexpected character `{char}`"),
            self.text_position()..self.text_position() + char.text_len(),
        );
        self.diagnostics.push(err);
        self.advance(char.len_utf8());
        ERROR_TOKEN
    }
}

/// Check if byte is a valid PN_CHARS character (simplified to ASCII subset)
fn is_pn_chars(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'\xB7')
}

/// Check if byte can start a PN_LOCAL
fn is_pn_local_start(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'_' | b':')
}
