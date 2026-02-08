//! A crate for generated Syntax node definitions and utility macros.
//! Both biome_turtle_lexer and biome_turtle_parser rely on these definitions, therefore
//! they are wrapped in this crate to prevent cyclic dependencies

#![deny(clippy::use_self)]

#[macro_use]
mod generated;
mod file_source;
mod syntax_node;

use biome_rowan::{AstNode, RawSyntaxKind, SyntaxKind, TokenText};

pub use biome_rowan::{TextLen, TextRange, TextSize, TokenAtOffset, TriviaPieceKind, WalkEvent};
pub use file_source::TurtleFileSource;
pub use generated::*;
pub use syntax_node::*;

use TurtleSyntaxKind::*;

impl From<u16> for TurtleSyntaxKind {
    fn from(d: u16) -> Self {
        assert!(d <= (Self::__LAST as u16));
        unsafe { std::mem::transmute::<u16, Self>(d) }
    }
}

impl From<TurtleSyntaxKind> for u16 {
    fn from(k: TurtleSyntaxKind) -> Self {
        k as Self
    }
}

impl TurtleSyntaxKind {
    /// Returns `true` for any contextual or non-contextual keyword
    #[inline]
    pub const fn is_keyword(self) -> bool {
        (self as u16) <= (Self::SPARQL_PREFIX_KW as u16)
            && (self as u16) >= (Self::PREFIX_KW as u16)
    }
}

impl biome_rowan::SyntaxKind for TurtleSyntaxKind {
    const TOMBSTONE: Self = TOMBSTONE;
    const EOF: Self = EOF;

    fn is_bogus(&self) -> bool {
        matches!(self, TURTLE_BOGUS | TURTLE_BOGUS_STATEMENT)
    }

    fn to_bogus(&self) -> Self {
        match self {
            kind if AnyTurtleStatement::can_cast(*kind) => TURTLE_BOGUS_STATEMENT,
            _ => TURTLE_BOGUS,
        }
    }

    #[inline]
    fn to_raw(&self) -> RawSyntaxKind {
        RawSyntaxKind(*self as u16)
    }

    #[inline]
    fn from_raw(raw: RawSyntaxKind) -> Self {
        Self::from(raw.0)
    }

    fn is_root(&self) -> bool {
        TurtleRoot::can_cast(*self)
    }

    fn is_list(&self) -> bool {
        Self::is_list(*self)
    }

    fn is_trivia(self) -> bool {
        matches!(self, Self::NEWLINE | Self::WHITESPACE | Self::COMMENT)
    }

    fn to_string(&self) -> Option<&'static str> {
        Self::to_string(self)
    }
}

impl TryFrom<TurtleSyntaxKind> for TriviaPieceKind {
    type Error = ();

    fn try_from(value: TurtleSyntaxKind) -> Result<Self, Self::Error> {
        if value.is_trivia() {
            match value {
                TurtleSyntaxKind::NEWLINE => Ok(Self::Newline),
                TurtleSyntaxKind::WHITESPACE => Ok(Self::Whitespace),
                TurtleSyntaxKind::COMMENT => Ok(Self::SingleLineComment),
                _ => unreachable!("Not Trivia"),
            }
        } else {
            Err(())
        }
    }
}

/// Text of `token`, excluding all trivia and removing quotes if `token` is a string literal.
pub fn inner_string_text(token: &TurtleSyntaxToken) -> TokenText {
    let text = token.token_text_trimmed();
    let kind = token.kind();

    match kind {
        TurtleSyntaxKind::TURTLE_STRING_LITERAL_LONG_QUOTE
        | TurtleSyntaxKind::TURTLE_STRING_LITERAL_LONG_SINGLE_QUOTE => {
            // Remove triple quotes """...""" or '''...'''
            let slice = TextRange::new(3.into(), text.len() - TextSize::from(3));
            text.slice(slice)
        }
        TurtleSyntaxKind::TURTLE_STRING_LITERAL_QUOTE
        | TurtleSyntaxKind::TURTLE_STRING_LITERAL_SINGLE_QUOTE => {
            // Remove single quotes "..." or '...'
            let slice = TextRange::new(1.into(), text.len() - TextSize::from(1));
            text.slice(slice)
        }
        TurtleSyntaxKind::TURTLE_IRIREF_LITERAL => {
            // Remove angle brackets <...>
            let slice = TextRange::new(1.into(), text.len() - TextSize::from(1));
            text.slice(slice)
        }
        _ => text,
    }
}
