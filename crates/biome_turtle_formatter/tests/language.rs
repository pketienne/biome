use biome_formatter_test::TestFormatLanguage;
use biome_fs::BiomePath;
use biome_turtle_formatter::TurtleFormatLanguage;
use biome_turtle_formatter::context::TurtleFormatContext;
use biome_turtle_parser::parse_turtle;
use biome_turtle_syntax::{TurtleFileSource, TurtleLanguage};
use biome_parser::AnyParse;
use biome_service::{
    settings::{ServiceLanguage, Settings},
    workspace::DocumentFileSource,
};

#[derive(Default)]
pub struct TurtleTestFormatLanguage {
    _source_type: TurtleFileSource,
}

impl TestFormatLanguage for TurtleTestFormatLanguage {
    type ServiceLanguage = TurtleLanguage;
    type Context = TurtleFormatContext;
    type FormatLanguage = TurtleFormatLanguage;

    fn parse(&self, text: &str) -> AnyParse {
        let parse = parse_turtle(text);
        parse.into()
    }

    fn to_format_language(
        &self,
        settings: &Settings,
        file_source: &DocumentFileSource,
    ) -> Self::FormatLanguage {
        let language_settings = &settings.languages.turtle.formatter;
        let options = Self::ServiceLanguage::resolve_format_options(
            &settings.formatter,
            &settings.override_settings,
            language_settings,
            &BiomePath::new(""),
            file_source,
        );
        TurtleFormatLanguage::new(options)
    }
}
