use crate::parser::TurtleParser;
use biome_parser::diagnostic::{ParseDiagnostic, expected_node};
use biome_rowan::TextRange;

pub(crate) fn expected_statement(p: &TurtleParser, range: TextRange) -> ParseDiagnostic {
    expected_node("statement", range, p)
}

pub(crate) fn expected_subject(p: &TurtleParser, range: TextRange) -> ParseDiagnostic {
    expected_node("subject", range, p)
}

pub(crate) fn expected_predicate(p: &TurtleParser, range: TextRange) -> ParseDiagnostic {
    expected_node("predicate", range, p)
}

pub(crate) fn expected_object(p: &TurtleParser, range: TextRange) -> ParseDiagnostic {
    expected_node("object", range, p)
}

pub(crate) fn expected_iri(p: &TurtleParser, range: TextRange) -> ParseDiagnostic {
    expected_node("IRI", range, p)
}

pub(crate) fn expected_blank_node(p: &TurtleParser, range: TextRange) -> ParseDiagnostic {
    expected_node("blank node", range, p)
}
