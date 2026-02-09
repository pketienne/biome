use biome_formatter::{IndentStyle, LineWidth};
use biome_formatter_test::check_reformat::CheckReformat;
use biome_turtle_formatter::context::TurtleFormatOptions;
use biome_turtle_formatter::{TurtleFormatLanguage, format_node};
use biome_turtle_parser::parse_turtle;

mod language {
    include!("language.rs");
}

#[ignore]
#[test]
// use this test check if your snippet prints as you wish, without using a snapshot
fn quick_test() {
    let src = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

<http://example.org/alice> a foaf:Person ;
    foaf:name "Alice" .
"#;
    let parse = parse_turtle(src);
    println!("{parse:#?}");

    let options = TurtleFormatOptions::default()
        .with_line_width(LineWidth::try_from(80).unwrap())
        .with_indent_style(IndentStyle::Space);
    let doc = format_node(options.clone(), &parse.syntax()).unwrap();
    let result = doc.print().unwrap();

    let root = &parse.syntax();
    let language = language::TurtleTestFormatLanguage::default();

    println!("{}", doc.into_document());
    eprintln!("{}", result.as_code());

    CheckReformat::new(
        root,
        result.as_code(),
        "quick_test",
        &language,
        TurtleFormatLanguage::new(options),
    )
    .check_reformat();
}
