mod parse_error;

use crate::token_source::TurtleTokenSource;
use biome_turtle_syntax::TurtleSyntaxKind::{self, *};
use biome_turtle_syntax::T;
use biome_parser::ParserContext;
use biome_parser::diagnostic::merge_diagnostics;
use biome_parser::event::Event;
use biome_parser::parse_lists::{ParseNodeList, ParseSeparatedList};
use biome_parser::parse_recovery::ParseRecoveryTokenSet;
use biome_parser::prelude::{ParsedSyntax::*, *};
use biome_parser::token_source::Trivia;

use self::parse_error::*;

pub(crate) struct TurtleParser<'source> {
    context: ParserContext<TurtleSyntaxKind>,
    source: TurtleTokenSource<'source>,
}

impl<'source> TurtleParser<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            context: ParserContext::default(),
            source: TurtleTokenSource::from_str(source),
        }
    }

    pub fn finish(
        self,
    ) -> (
        Vec<Event<TurtleSyntaxKind>>,
        Vec<ParseDiagnostic>,
        Vec<Trivia>,
    ) {
        let (trivia, lexer_diagnostics) = self.source.finish();
        let (events, parse_diagnostics) = self.context.finish();

        let diagnostics = merge_diagnostics(lexer_diagnostics, parse_diagnostics);

        (events, diagnostics, trivia)
    }
}

impl<'source> Parser for TurtleParser<'source> {
    type Kind = TurtleSyntaxKind;
    type Source = TurtleTokenSource<'source>;

    fn context(&self) -> &ParserContext<Self::Kind> {
        &self.context
    }

    fn context_mut(&mut self) -> &mut ParserContext<Self::Kind> {
        &mut self.context
    }

    fn source(&self) -> &Self::Source {
        &self.source
    }

    fn source_mut(&mut self) -> &mut Self::Source {
        &mut self.source
    }
}

pub(crate) fn parse_root(p: &mut TurtleParser) -> CompletedMarker {
    let m = p.start();

    p.eat(UNICODE_BOM);

    StatementList.parse_list(p);

    p.expect(EOF);

    m.complete(p, TURTLE_ROOT)
}

/// Token set for tokens that can start a statement
const STATEMENT_START: TokenSet<TurtleSyntaxKind> = token_set![
    T![prefix],
    T![base],
    SPARQL_BASE_KW,
    SPARQL_PREFIX_KW,
    TURTLE_IRIREF_LITERAL,
    TURTLE_PNAME_LN_LITERAL,
    TURTLE_PNAME_NS_LITERAL,
    TURTLE_BLANK_NODE_LABEL_LITERAL,
    TURTLE_ANON_TOKEN,
    T!['['],
    T!['('],
];

/// Token set for tokens that can be a subject
const SUBJECT_START: TokenSet<TurtleSyntaxKind> = token_set![
    TURTLE_IRIREF_LITERAL,
    TURTLE_PNAME_LN_LITERAL,
    TURTLE_PNAME_NS_LITERAL,
    TURTLE_BLANK_NODE_LABEL_LITERAL,
    TURTLE_ANON_TOKEN,
    T!['['],
    T!['('],
];

/// Token set for tokens that can be a verb
const VERB_START: TokenSet<TurtleSyntaxKind> = token_set![
    T![a],
    TURTLE_IRIREF_LITERAL,
    TURTLE_PNAME_LN_LITERAL,
    TURTLE_PNAME_NS_LITERAL,
];

/// Token set for tokens that can be an object
const OBJECT_START: TokenSet<TurtleSyntaxKind> = token_set![
    TURTLE_IRIREF_LITERAL,
    TURTLE_PNAME_LN_LITERAL,
    TURTLE_PNAME_NS_LITERAL,
    TURTLE_BLANK_NODE_LABEL_LITERAL,
    TURTLE_ANON_TOKEN,
    T!['['],
    T!['('],
    TURTLE_STRING_LITERAL_QUOTE,
    TURTLE_STRING_LITERAL_SINGLE_QUOTE,
    TURTLE_STRING_LITERAL_LONG_QUOTE,
    TURTLE_STRING_LITERAL_LONG_SINGLE_QUOTE,
    TURTLE_INTEGER_LITERAL,
    TURTLE_DECIMAL_LITERAL,
    TURTLE_DOUBLE_LITERAL,
    T![true],
    T![false],
];

// Statement list
struct StatementList;

impl ParseNodeList for StatementList {
    type Kind = TurtleSyntaxKind;
    type Parser<'source> = TurtleParser<'source>;

    const LIST_KIND: Self::Kind = TURTLE_STATEMENT_LIST;

    fn parse_element(&mut self, p: &mut TurtleParser) -> ParsedSyntax {
        parse_statement(p)
    }

    fn is_at_list_end(&self, p: &mut TurtleParser) -> bool {
        p.at(EOF)
    }

    fn recover(
        &mut self,
        p: &mut TurtleParser,
        parsed_element: ParsedSyntax,
    ) -> biome_parser::parse_recovery::RecoveryResult {
        parsed_element.or_recover_with_token_set(
            p,
            &ParseRecoveryTokenSet::new(TURTLE_BOGUS_STATEMENT, STATEMENT_START),
            expected_statement,
        )
    }
}

fn parse_statement(p: &mut TurtleParser) -> ParsedSyntax {
    match p.cur() {
        T![prefix] => parse_prefix_declaration(p),
        T![base] => parse_base_declaration(p),
        SPARQL_PREFIX_KW => parse_sparql_prefix(p),
        SPARQL_BASE_KW => parse_sparql_base(p),
        _ if p.at_ts(SUBJECT_START) => parse_triples(p),
        _ => Absent,
    }
}

fn parse_prefix_declaration(p: &mut TurtleParser) -> ParsedSyntax {
    if !p.at(T![prefix]) {
        return Absent;
    }
    let m = p.start();
    p.bump(T![prefix]);
    p.expect(TURTLE_PNAME_NS_LITERAL);
    p.expect(TURTLE_IRIREF_LITERAL);
    p.expect(T![.]);
    Present(m.complete(p, TURTLE_PREFIX_DECLARATION))
}

fn parse_base_declaration(p: &mut TurtleParser) -> ParsedSyntax {
    if !p.at(T![base]) {
        return Absent;
    }
    let m = p.start();
    p.bump(T![base]);
    p.expect(TURTLE_IRIREF_LITERAL);
    p.expect(T![.]);
    Present(m.complete(p, TURTLE_BASE_DECLARATION))
}

fn parse_sparql_prefix(p: &mut TurtleParser) -> ParsedSyntax {
    if !p.at(SPARQL_PREFIX_KW) {
        return Absent;
    }
    let m = p.start();
    p.bump(SPARQL_PREFIX_KW);
    p.expect(TURTLE_PNAME_NS_LITERAL);
    p.expect(TURTLE_IRIREF_LITERAL);
    Present(m.complete(p, TURTLE_SPARQL_PREFIX_DECLARATION))
}

fn parse_sparql_base(p: &mut TurtleParser) -> ParsedSyntax {
    if !p.at(SPARQL_BASE_KW) {
        return Absent;
    }
    let m = p.start();
    p.bump(SPARQL_BASE_KW);
    p.expect(TURTLE_IRIREF_LITERAL);
    Present(m.complete(p, TURTLE_SPARQL_BASE_DECLARATION))
}

fn parse_triples(p: &mut TurtleParser) -> ParsedSyntax {
    if !p.at_ts(SUBJECT_START) {
        return Absent;
    }

    let m = p.start();

    // Parse subject
    let subject_m = p.start();
    parse_subject_value(p);
    subject_m.complete(p, TURTLE_SUBJECT);

    // Parse predicate-object list
    parse_predicate_object_list(p);

    p.expect(T![.]);

    Present(m.complete(p, TURTLE_TRIPLES))
}

fn parse_subject_value(p: &mut TurtleParser) {
    match p.cur() {
        TURTLE_IRIREF_LITERAL => {
            parse_iri(p);
        }
        TURTLE_PNAME_LN_LITERAL | TURTLE_PNAME_NS_LITERAL => {
            parse_iri(p);
        }
        TURTLE_BLANK_NODE_LABEL_LITERAL | TURTLE_ANON_TOKEN => {
            parse_blank_node(p);
        }
        T!['['] => {
            parse_blank_node_property_list(p);
        }
        T!['('] => {
            parse_collection(p);
        }
        _ => {
            p.error(expected_subject(p, p.cur_range()));
        }
    }
}

fn parse_predicate_object_list(p: &mut TurtleParser) {
    let m = p.start();
    PredicateObjectPairList.parse_list(p);
    m.complete(p, TURTLE_PREDICATE_OBJECT_LIST);
}

struct PredicateObjectPairList;

impl ParseSeparatedList for PredicateObjectPairList {
    type Kind = TurtleSyntaxKind;
    type Parser<'source> = TurtleParser<'source>;

    const LIST_KIND: Self::Kind = TURTLE_PREDICATE_OBJECT_PAIR_LIST;

    fn parse_element(&mut self, p: &mut TurtleParser) -> ParsedSyntax {
        parse_predicate_object_pair(p)
    }

    fn is_at_list_end(&self, p: &mut TurtleParser) -> bool {
        p.at(T![.]) || p.at(T![']']) || p.at(EOF)
    }

    fn recover(
        &mut self,
        p: &mut TurtleParser,
        parsed_element: ParsedSyntax,
    ) -> biome_parser::parse_recovery::RecoveryResult {
        parsed_element.or_recover_with_token_set(
            p,
            &ParseRecoveryTokenSet::new(TURTLE_BOGUS, VERB_START.union(token_set![T![.], T![']']])),
            expected_predicate,
        )
    }

    fn separating_element_kind(&mut self) -> TurtleSyntaxKind {
        T![;]
    }

    fn allow_trailing_separating_element(&self) -> bool {
        true
    }
}

fn parse_predicate_object_pair(p: &mut TurtleParser) -> ParsedSyntax {
    if !p.at_ts(VERB_START) {
        return Absent;
    }

    let m = p.start();

    // Parse verb
    let verb_m = p.start();
    if p.at(T![a]) {
        p.bump(T![a]);
    } else {
        parse_iri(p);
    }
    verb_m.complete(p, TURTLE_VERB);

    // Parse object list
    ObjectList.parse_list(p);

    Present(m.complete(p, TURTLE_PREDICATE_OBJECT_PAIR))
}

struct ObjectList;

impl ParseSeparatedList for ObjectList {
    type Kind = TurtleSyntaxKind;
    type Parser<'source> = TurtleParser<'source>;

    const LIST_KIND: Self::Kind = TURTLE_OBJECT_LIST;

    fn parse_element(&mut self, p: &mut TurtleParser) -> ParsedSyntax {
        parse_object(p)
    }

    fn is_at_list_end(&self, p: &mut TurtleParser) -> bool {
        p.at(T![;]) || p.at(T![.]) || p.at(T![']']) || p.at(EOF)
    }

    fn recover(
        &mut self,
        p: &mut TurtleParser,
        parsed_element: ParsedSyntax,
    ) -> biome_parser::parse_recovery::RecoveryResult {
        parsed_element.or_recover_with_token_set(
            p,
            &ParseRecoveryTokenSet::new(
                TURTLE_BOGUS,
                OBJECT_START.union(token_set![T![;], T![.], T![']']]),
            ),
            expected_object,
        )
    }

    fn separating_element_kind(&mut self) -> TurtleSyntaxKind {
        T![,]
    }
}

fn parse_object(p: &mut TurtleParser) -> ParsedSyntax {
    if !p.at_ts(OBJECT_START) {
        return Absent;
    }

    let m = p.start();

    match p.cur() {
        TURTLE_IRIREF_LITERAL | TURTLE_PNAME_LN_LITERAL | TURTLE_PNAME_NS_LITERAL => {
            parse_iri(p);
        }
        TURTLE_BLANK_NODE_LABEL_LITERAL | TURTLE_ANON_TOKEN => {
            parse_blank_node(p);
        }
        T!['['] => {
            parse_blank_node_property_list(p);
        }
        T!['('] => {
            parse_collection(p);
        }
        TURTLE_STRING_LITERAL_QUOTE
        | TURTLE_STRING_LITERAL_SINGLE_QUOTE
        | TURTLE_STRING_LITERAL_LONG_QUOTE
        | TURTLE_STRING_LITERAL_LONG_SINGLE_QUOTE => {
            parse_rdf_literal(p);
        }
        TURTLE_INTEGER_LITERAL | TURTLE_DECIMAL_LITERAL | TURTLE_DOUBLE_LITERAL => {
            parse_numeric_literal(p);
        }
        T![true] | T![false] => {
            parse_boolean_literal(p);
        }
        _ => {
            p.error(expected_object(p, p.cur_range()));
        }
    }

    Present(m.complete(p, TURTLE_OBJECT))
}

fn parse_iri(p: &mut TurtleParser) {
    let m = p.start();
    match p.cur() {
        TURTLE_IRIREF_LITERAL => {
            p.bump(TURTLE_IRIREF_LITERAL);
        }
        TURTLE_PNAME_LN_LITERAL | TURTLE_PNAME_NS_LITERAL => {
            let pm = p.start();
            p.bump_any();
            pm.complete(p, TURTLE_PREFIXED_NAME);
        }
        _ => {
            p.error(expected_iri(p, p.cur_range()));
        }
    }
    m.complete(p, TURTLE_IRI);
}

fn parse_blank_node(p: &mut TurtleParser) {
    let m = p.start();
    match p.cur() {
        TURTLE_BLANK_NODE_LABEL_LITERAL => {
            p.bump(TURTLE_BLANK_NODE_LABEL_LITERAL);
        }
        TURTLE_ANON_TOKEN => {
            p.bump(TURTLE_ANON_TOKEN);
        }
        _ => {
            p.error(expected_blank_node(p, p.cur_range()));
        }
    }
    m.complete(p, TURTLE_BLANK_NODE);
}

fn parse_blank_node_property_list(p: &mut TurtleParser) {
    let m = p.start();
    p.expect(T!['[']);
    parse_predicate_object_list(p);
    p.expect(T![']']);
    m.complete(p, TURTLE_BLANK_NODE_PROPERTY_LIST);
}

fn parse_collection(p: &mut TurtleParser) {
    let m = p.start();
    p.expect(T!['(']);
    CollectionObjectList.parse_list(p);
    p.expect(T![')']);
    m.complete(p, TURTLE_COLLECTION);
}

struct CollectionObjectList;

impl ParseNodeList for CollectionObjectList {
    type Kind = TurtleSyntaxKind;
    type Parser<'source> = TurtleParser<'source>;

    const LIST_KIND: Self::Kind = TURTLE_COLLECTION_OBJECT_LIST;

    fn parse_element(&mut self, p: &mut TurtleParser) -> ParsedSyntax {
        parse_object(p)
    }

    fn is_at_list_end(&self, p: &mut TurtleParser) -> bool {
        p.at(T![')']) || p.at(EOF)
    }

    fn recover(
        &mut self,
        p: &mut TurtleParser,
        parsed_element: ParsedSyntax,
    ) -> biome_parser::parse_recovery::RecoveryResult {
        parsed_element.or_recover_with_token_set(
            p,
            &ParseRecoveryTokenSet::new(TURTLE_BOGUS, OBJECT_START.union(token_set![T![')']])),
            expected_object,
        )
    }
}

fn parse_rdf_literal(p: &mut TurtleParser) {
    let m = p.start();

    // Parse string value
    let sm = p.start();
    p.bump_any(); // string token
    sm.complete(p, TURTLE_STRING);

    // Optional language tag or datatype annotation
    if p.at(TURTLE_LANGTAG_LITERAL) {
        p.bump(TURTLE_LANGTAG_LITERAL);
    } else if p.at(T!["^^"]) {
        let dm = p.start();
        p.bump(T!["^^"]);
        parse_iri(p);
        dm.complete(p, TURTLE_DATATYPE_ANNOTATION);
    }

    m.complete(p, TURTLE_RDF_LITERAL);
}

fn parse_numeric_literal(p: &mut TurtleParser) {
    let m = p.start();
    p.bump_any(); // INTEGER, DECIMAL, or DOUBLE
    m.complete(p, TURTLE_NUMERIC_LITERAL);
}

fn parse_boolean_literal(p: &mut TurtleParser) {
    let m = p.start();
    p.bump_any(); // true or false
    m.complete(p, TURTLE_BOOLEAN_LITERAL);
}
