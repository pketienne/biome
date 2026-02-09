use biome_formatter_test::check_reformat::CheckReformat;
use biome_yaml_formatter::format_node;
use biome_yaml_formatter::context::YamlFormatOptions;
use biome_yaml_formatter::YamlFormatLanguage;
use biome_yaml_parser::parse_yaml;

mod language {
    include!("language.rs");
}

#[ignore]
#[test]
// use this test check if your snippet prints as you wish, without using a snapshot
fn quick_test() {
    let src = r#"key: value
"#;
    let parse = parse_yaml(src);
    let options = YamlFormatOptions::default();
    let result = format_node(options.clone(), &parse.syntax())
        .unwrap()
        .print()
        .unwrap();

    let root = &parse.syntax();
    let language = language::YamlTestFormatLanguage::default();

    let check_reformat = CheckReformat::new(
        root,
        result.as_code(),
        "quick_test",
        &language,
        YamlFormatLanguage::new(options),
    );
    check_reformat.check_reformat();

    assert_eq!(
        result.as_code(),
        r#"key: value
"#
    );
}
