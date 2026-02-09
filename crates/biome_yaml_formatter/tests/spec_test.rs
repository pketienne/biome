use biome_formatter_test::spec::{SpecSnapshot, SpecTestFile};
use biome_yaml_formatter::YamlFormatLanguage;
use biome_yaml_syntax::YamlLanguage;
use biome_test_utils::create_formatting_options;
use camino::Utf8Path;

mod language {
    include!("language.rs");
}

/// [insta.rs](https://insta.rs/docs) snapshot testing
///
/// For better development workflow, run
/// `cargo watch -i '*.new' -x 'test -p biome_yaml_formatter formatter'`
///
/// To review and commit the snapshots, `cargo install cargo-insta`, and run
/// `cargo insta review` or `cargo insta accept`
///
/// The input and the expected output are stored as dedicated files in the `tests/specs` directory where
/// the input file name is `{spec_name}.yaml` and the output file name is `{spec_name}.yaml.snap`.
///
/// Specs can be grouped in directories by specifying the directory name in the spec name. Examples:
///
/// # Examples
///
/// * `yaml/mapping/simple` -> input: `tests/specs/yaml/mapping/simple.yaml`, expected output: `tests/specs/yaml/mapping/simple.yaml.snap`
pub fn run(spec_input_file: &str, _expected_file: &str, test_directory: &str, _file_type: &str) {
    let root_path = Utf8Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/specs/"));

    let Some(test_file) = SpecTestFile::try_from_file(spec_input_file, root_path, |_| None) else {
        return;
    };
    let mut diagnostics = vec![];

    let options =
        create_formatting_options::<YamlLanguage>(test_file.input_file(), &mut diagnostics);

    let language = language::YamlTestFormatLanguage::default();

    let snapshot = SpecSnapshot::new(
        test_file,
        test_directory,
        language,
        YamlFormatLanguage::new(options),
    );

    snapshot.test()
}
