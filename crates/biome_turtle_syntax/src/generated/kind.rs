//! Generated file, do not edit by hand, see `xtask/codegen`

#![allow(bad_style, missing_docs, unreachable_pub)]
#[doc = r" The kind of syntax node, e.g. `IDENT`, `FUNCTION_KW`, or `FOR_STMT`."]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u16)]
pub enum TurtleSyntaxKind {
    #[doc(hidden)]
    TOMBSTONE,
    #[doc = r" Marks the end of the file. May have trivia attached"]
    EOF,
    #[doc = r" Any Unicode BOM character that may be present at the start of"]
    #[doc = r" a file."]
    UNICODE_BOM,
    DOT,
    SEMICOLON,
    COMMA,
    L_BRACK,
    R_BRACK,
    L_PAREN,
    R_PAREN,
    CARET_CARET,
    PREFIX_KW,
    BASE_KW,
    A_KW,
    TRUE_KW,
    FALSE_KW,
    SPARQL_BASE_KW,
    SPARQL_PREFIX_KW,
    TURTLE_IRIREF_LITERAL,
    TURTLE_PNAME_NS_LITERAL,
    TURTLE_PNAME_LN_LITERAL,
    TURTLE_BLANK_NODE_LABEL_LITERAL,
    TURTLE_LANGTAG_LITERAL,
    TURTLE_INTEGER_LITERAL,
    TURTLE_DECIMAL_LITERAL,
    TURTLE_DOUBLE_LITERAL,
    TURTLE_STRING_LITERAL_QUOTE,
    TURTLE_STRING_LITERAL_SINGLE_QUOTE,
    TURTLE_STRING_LITERAL_LONG_QUOTE,
    TURTLE_STRING_LITERAL_LONG_SINGLE_QUOTE,
    ERROR_TOKEN,
    NEWLINE,
    WHITESPACE,
    COMMENT,
    TURTLE_ANON_TOKEN,
    TURTLE_ROOT,
    TURTLE_STATEMENT_LIST,
    TURTLE_PREFIX_DECLARATION,
    TURTLE_BASE_DECLARATION,
    TURTLE_SPARQL_PREFIX_DECLARATION,
    TURTLE_SPARQL_BASE_DECLARATION,
    TURTLE_TRIPLES,
    TURTLE_PREDICATE_OBJECT_LIST,
    TURTLE_PREDICATE_OBJECT_PAIR,
    TURTLE_PREDICATE_OBJECT_PAIR_LIST,
    TURTLE_OBJECT_LIST,
    TURTLE_SUBJECT,
    TURTLE_VERB,
    TURTLE_OBJECT,
    TURTLE_IRI,
    TURTLE_PREFIXED_NAME,
    TURTLE_BLANK_NODE,
    TURTLE_BLANK_NODE_PROPERTY_LIST,
    TURTLE_COLLECTION,
    TURTLE_COLLECTION_OBJECT_LIST,
    TURTLE_RDF_LITERAL,
    TURTLE_NUMERIC_LITERAL,
    TURTLE_BOOLEAN_LITERAL,
    TURTLE_DATATYPE_ANNOTATION,
    TURTLE_STRING,
    TURTLE_BOGUS,
    TURTLE_BOGUS_STATEMENT,
    #[doc(hidden)]
    __LAST,
}
use self::TurtleSyntaxKind::*;
impl TurtleSyntaxKind {
    pub const fn is_punct(self) -> bool {
        matches!(
            self,
            DOT | SEMICOLON | COMMA | L_BRACK | R_BRACK | L_PAREN | R_PAREN | CARET_CARET
        )
    }
    pub const fn is_literal(self) -> bool {
        matches!(
            self,
            TURTLE_IRIREF_LITERAL
                | TURTLE_PNAME_NS_LITERAL
                | TURTLE_PNAME_LN_LITERAL
                | TURTLE_BLANK_NODE_LABEL_LITERAL
                | TURTLE_LANGTAG_LITERAL
                | TURTLE_INTEGER_LITERAL
                | TURTLE_DECIMAL_LITERAL
                | TURTLE_DOUBLE_LITERAL
                | TURTLE_STRING_LITERAL_QUOTE
                | TURTLE_STRING_LITERAL_SINGLE_QUOTE
                | TURTLE_STRING_LITERAL_LONG_QUOTE
                | TURTLE_STRING_LITERAL_LONG_SINGLE_QUOTE
        )
    }
    pub const fn is_list(self) -> bool {
        matches!(
            self,
            TURTLE_STATEMENT_LIST
                | TURTLE_PREDICATE_OBJECT_LIST
                | TURTLE_PREDICATE_OBJECT_PAIR_LIST
                | TURTLE_OBJECT_LIST
                | TURTLE_BLANK_NODE_PROPERTY_LIST
                | TURTLE_COLLECTION_OBJECT_LIST
        )
    }
    pub fn from_keyword(ident: &str) -> Option<Self> {
        let kw = match ident {
            "prefix" => PREFIX_KW,
            "base" => BASE_KW,
            "a" => A_KW,
            "true" => TRUE_KW,
            "false" => FALSE_KW,
            "SPARQL_BASE" => SPARQL_BASE_KW,
            "SPARQL_PREFIX" => SPARQL_PREFIX_KW,
            _ => return None,
        };
        Some(kw)
    }
    pub const fn to_string(&self) -> Option<&'static str> {
        let tok = match self {
            DOT => ".",
            SEMICOLON => ";",
            COMMA => ",",
            L_BRACK => "[",
            R_BRACK => "]",
            L_PAREN => "(",
            R_PAREN => ")",
            CARET_CARET => "^^",
            PREFIX_KW => "prefix",
            BASE_KW => "base",
            A_KW => "a",
            TRUE_KW => "true",
            FALSE_KW => "false",
            SPARQL_BASE_KW => "SPARQL_BASE",
            SPARQL_PREFIX_KW => "SPARQL_PREFIX",
            EOF => "EOF",
            _ => return None,
        };
        Some(tok)
    }
}
#[doc = r" Utility macro for creating a SyntaxKind through simple macro syntax"]
#[macro_export]
macro_rules ! T { [.] => { $ crate :: TurtleSyntaxKind :: DOT } ; [;] => { $ crate :: TurtleSyntaxKind :: SEMICOLON } ; [,] => { $ crate :: TurtleSyntaxKind :: COMMA } ; ['['] => { $ crate :: TurtleSyntaxKind :: L_BRACK } ; [']'] => { $ crate :: TurtleSyntaxKind :: R_BRACK } ; ['('] => { $ crate :: TurtleSyntaxKind :: L_PAREN } ; [')'] => { $ crate :: TurtleSyntaxKind :: R_PAREN } ; ["^^"] => { $ crate :: TurtleSyntaxKind :: CARET_CARET } ; [prefix] => { $ crate :: TurtleSyntaxKind :: PREFIX_KW } ; [base] => { $ crate :: TurtleSyntaxKind :: BASE_KW } ; [a] => { $ crate :: TurtleSyntaxKind :: A_KW } ; [true] => { $ crate :: TurtleSyntaxKind :: TRUE_KW } ; [false] => { $ crate :: TurtleSyntaxKind :: FALSE_KW } ; [SPARQL_BASE] => { $ crate :: TurtleSyntaxKind :: SPARQL_BASE_KW } ; [SPARQL_PREFIX] => { $ crate :: TurtleSyntaxKind :: SPARQL_PREFIX_KW } ; [ident] => { $ crate :: TurtleSyntaxKind :: IDENT } ; [EOF] => { $ crate :: TurtleSyntaxKind :: EOF } ; [UNICODE_BOM] => { $ crate :: TurtleSyntaxKind :: UNICODE_BOM } ; [#] => { $ crate :: TurtleSyntaxKind :: HASH } ; }
