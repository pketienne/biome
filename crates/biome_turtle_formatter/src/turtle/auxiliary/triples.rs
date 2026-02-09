use crate::prelude::*;
use biome_formatter::write;
use biome_turtle_syntax::TurtleTriples;
#[derive(Debug, Clone, Default)]
pub(crate) struct FormatTurtleTriples;
impl FormatNodeRule<TurtleTriples> for FormatTurtleTriples {
    fn fmt_fields(&self, node: &TurtleTriples, f: &mut TurtleFormatter) -> FormatResult<()> {
        let subject = node.subject()?;
        let predicates = node.predicates()?;
        let dot = node.dot_token()?;

        let first_predicate_in_new_line = f.options().first_predicate_in_new_line();

        if first_predicate_in_new_line {
            write!(
                f,
                [
                    subject.format(),
                    indent(&biome_formatter::format_args!(
                        hard_line_break(),
                        predicates.format(),
                        space(),
                        dot.format(),
                    )),
                ]
            )
        } else {
            write!(
                f,
                [
                    subject.format(),
                    space(),
                    indent(&biome_formatter::format_args!(
                        predicates.format(),
                        space(),
                        dot.format(),
                    )),
                ]
            )
        }
    }
}
