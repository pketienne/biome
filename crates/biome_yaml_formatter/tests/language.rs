use biome_formatter_test::TestFormatLanguage;
use biome_fs::BiomePath;
use biome_parser::AnyParse;
use biome_service::settings::{ServiceLanguage, Settings};
use biome_service::workspace::DocumentFileSource;
use biome_yaml_formatter::YamlFormatLanguage;
use biome_yaml_formatter::context::YamlFormatContext;
use biome_yaml_parser::parse_yaml;
use biome_yaml_syntax::YamlLanguage;

#[derive(Default)]
pub struct YamlTestFormatLanguage;

impl TestFormatLanguage for YamlTestFormatLanguage {
    type ServiceLanguage = YamlLanguage;
    type Context = YamlFormatContext;
    type FormatLanguage = YamlFormatLanguage;

    fn parse(&self, text: &str) -> AnyParse {
        parse_yaml(text).into()
    }

    fn to_format_language(
        &self,
        settings: &Settings,
        file_source: &DocumentFileSource,
    ) -> Self::FormatLanguage {
        let language_settings = &settings.languages.yaml.formatter;
        let options = Self::ServiceLanguage::resolve_format_options(
            &settings.formatter,
            &settings.override_settings,
            language_settings,
            &BiomePath::new(""),
            file_source,
        );
        YamlFormatLanguage::new(options)
    }
}
