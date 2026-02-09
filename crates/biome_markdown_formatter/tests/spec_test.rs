use biome_formatter_test::spec::{SpecSnapshot, SpecTestFile};
use biome_markdown_formatter::MarkdownFormatLanguage;
use biome_markdown_syntax::MarkdownLanguage;
use biome_test_utils::create_formatting_options;
use camino::Utf8Path;

mod language {
    include!("language.rs");
}

/// [insta.rs](https://insta.rs/docs) snapshot testing
///
/// For better development workflow, run
/// `cargo watch -i '*.new' -x 'test -p biome_markdown_formatter formatter'`
///
/// To review and commit the snapshots, `cargo install cargo-insta`, and run
/// `cargo insta review` or `cargo insta accept`
///
/// The input and the expected output are stored as dedicated files in the `tests/specs` directory where
/// the input file name is `{spec_name}.md` and the output file name is `{spec_name}.md.snap`.
///
/// Specs can be grouped in directories by specifying the directory name in the spec name. Examples:
///
/// # Examples
///
/// * `md/headings` -> input: `tests/specs/md/headings.md`, expected output: `tests/specs/md/headings.md.snap`
/// * `headings` -> input: `tests/specs/headings.md`, expected output: `tests/specs/headings.md.snap`
pub fn run(spec_input_file: &str, _expected_file: &str, test_directory: &str, _file_type: &str) {
    let root_path = Utf8Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/specs/"));

    let Some(test_file) = SpecTestFile::try_from_file(spec_input_file, root_path, |_| None) else {
        return;
    };
    let mut diagnostics = vec![];

    let options =
        create_formatting_options::<MarkdownLanguage>(test_file.input_file(), &mut diagnostics);

    let language = language::MarkdownTestFormatLanguage::default();

    let snapshot = SpecSnapshot::new(
        test_file,
        test_directory,
        language,
        MarkdownFormatLanguage::new(options),
    );

    snapshot.test()
}
