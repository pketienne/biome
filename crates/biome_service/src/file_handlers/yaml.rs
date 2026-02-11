use super::{
    AnalyzerVisitorBuilder, CodeActionsParams, DocumentFileSource, EnabledForPath,
    ExtensionHandler, ParseResult, ProcessFixAll, ProcessLint, SearchCapabilities,
};
use crate::configuration::to_analyzer_rules;
use crate::file_handlers::DebugCapabilities;
use crate::file_handlers::{
    AnalyzerCapabilities, Capabilities, FixAllParams, FormatterCapabilities, LintParams,
    LintResults, ParserCapabilities,
};
use crate::settings::{
    FormatSettings, LanguageListSettings, LanguageSettings, OverrideSettings, ServiceLanguage,
    Settings, check_feature_activity, check_override_feature_activity,
};
use crate::workspace::{CodeAction, FixFileResult, GetSyntaxTreeResult, PullActionsResult};
use crate::{WorkspaceError, extension_error};
use biome_analyze::options::PreferredQuote;
use biome_analyze::{AnalysisFilter, AnalyzerConfiguration, AnalyzerOptions, ControlFlow, Never};
use biome_configuration::yaml::{
    YamlAssistConfiguration, YamlAssistEnabled, YamlFormatterConfiguration, YamlFormatterEnabled,
    YamlLinterConfiguration, YamlLinterEnabled, YamlParserConfiguration,
};
use biome_formatter::{FormatError, IndentStyle, IndentWidth, LineEnding, LineWidth, Printed};
use biome_fs::BiomePath;
use biome_parser::AnyParse;
use biome_rowan::{AstNode, NodeCache};
use biome_rowan::{TextRange, TextSize, TokenAtOffset};
use biome_yaml_analyze::{YamlAnalyzeServices, analyze};
use biome_yaml_formatter::context::YamlFormatOptions;
use biome_yaml_formatter::format_node;
use biome_yaml_syntax::{YamlFileSource, YamlLanguage, YamlRoot, YamlSyntaxNode};
use camino::Utf8Path;
use either::Either;
use std::borrow::Cow;
use tracing::{debug_span, error, instrument};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct YamlFormatterSettings {
    pub line_ending: Option<LineEnding>,
    pub line_width: Option<LineWidth>,
    pub indent_width: Option<IndentWidth>,
    pub indent_style: Option<IndentStyle>,
    pub enabled: Option<YamlFormatterEnabled>,
}

impl From<YamlFormatterConfiguration> for YamlFormatterSettings {
    fn from(configuration: YamlFormatterConfiguration) -> Self {
        Self {
            line_ending: configuration.line_ending,
            line_width: configuration.line_width,
            indent_width: configuration.indent_width,
            indent_style: configuration.indent_style,
            enabled: configuration.enabled,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct YamlParserSettings {}

impl From<YamlParserConfiguration> for YamlParserSettings {
    fn from(_configuration: YamlParserConfiguration) -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct YamlLinterSettings {
    pub enabled: Option<YamlLinterEnabled>,
}

impl From<YamlLinterConfiguration> for YamlLinterSettings {
    fn from(configuration: YamlLinterConfiguration) -> Self {
        Self {
            enabled: configuration.enabled,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct YamlAssistSettings {
    pub enabled: Option<YamlAssistEnabled>,
}

impl From<YamlAssistConfiguration> for YamlAssistSettings {
    fn from(configuration: YamlAssistConfiguration) -> Self {
        Self {
            enabled: configuration.enabled,
        }
    }
}

impl ServiceLanguage for YamlLanguage {
    type FormatterSettings = YamlFormatterSettings;
    type LinterSettings = YamlLinterSettings;
    type AssistSettings = YamlAssistSettings;
    type FormatOptions = YamlFormatOptions;
    type ParserSettings = YamlParserSettings;
    type ParserOptions = ();
    type EnvironmentSettings = ();

    fn lookup_settings(language: &LanguageListSettings) -> &LanguageSettings<Self> {
        &language.yaml
    }

    fn resolve_environment(_settings: &Settings) -> Option<&Self::EnvironmentSettings> {
        None
    }

    fn resolve_parse_options(
        _overrides: &OverrideSettings,
        _language: &Self::ParserSettings,
        _path: &BiomePath,
        _file_source: &DocumentFileSource,
    ) -> Self::ParserOptions {
    }

    fn resolve_format_options(
        global: &FormatSettings,
        _overrides: &OverrideSettings,
        language: &YamlFormatterSettings,
        _path: &BiomePath,
        document_file_source: &DocumentFileSource,
    ) -> Self::FormatOptions {
        let indent_style = language
            .indent_style
            .or(global.indent_style)
            .unwrap_or(IndentStyle::Space);
        let line_width = language
            .line_width
            .or(global.line_width)
            .unwrap_or_default();
        let indent_width = language
            .indent_width
            .or(global.indent_width)
            .unwrap_or_default();
        let line_ending = language
            .line_ending
            .or(global.line_ending)
            .unwrap_or_default();

        let file_source = document_file_source
            .to_yaml_file_source()
            .unwrap_or_default();

        YamlFormatOptions::new(file_source)
            .with_line_ending(line_ending)
            .with_indent_style(indent_style)
            .with_indent_width(indent_width)
            .with_line_width(line_width)
    }

    fn resolve_analyzer_options(
        global: &Settings,
        _language: &Self::LinterSettings,
        _environment: Option<&Self::EnvironmentSettings>,
        path: &BiomePath,
        _file_source: &DocumentFileSource,
        suppression_reason: Option<&str>,
    ) -> AnalyzerOptions {
        let configuration = AnalyzerConfiguration::default()
            .with_rules(to_analyzer_rules(global, path.as_path()))
            .with_preferred_quote(PreferredQuote::Double);
        AnalyzerOptions::default()
            .with_file_path(path.as_path())
            .with_configuration(configuration)
            .with_suppression_reason(suppression_reason)
    }

    fn linter_enabled_for_file_path(settings: &Settings, path: &Utf8Path) -> bool {
        let overrides_activity =
            settings
                .override_settings
                .patterns
                .iter()
                .rev()
                .find_map(|pattern| {
                    check_override_feature_activity(
                        pattern.languages.yaml.linter.enabled,
                        pattern.linter.enabled,
                    )
                    .filter(|_| pattern.is_file_included(path))
                });

        overrides_activity
            .or(check_feature_activity(
                settings.languages.yaml.linter.enabled,
                settings.linter.enabled,
            ))
            .unwrap_or_default()
            .into()
    }

    fn formatter_enabled_for_file_path(settings: &Settings, path: &Utf8Path) -> bool {
        let overrides_activity =
            settings
                .override_settings
                .patterns
                .iter()
                .rev()
                .find_map(|pattern| {
                    check_override_feature_activity(
                        pattern.languages.yaml.formatter.enabled,
                        pattern.formatter.enabled,
                    )
                    .filter(|_| pattern.is_file_included(path))
                });

        overrides_activity
            .or(check_feature_activity(
                settings.languages.yaml.formatter.enabled,
                settings.formatter.enabled,
            ))
            .unwrap_or_default()
            .into()
    }

    fn assist_enabled_for_file_path(settings: &Settings, path: &Utf8Path) -> bool {
        let overrides_activity =
            settings
                .override_settings
                .patterns
                .iter()
                .rev()
                .find_map(|pattern| {
                    check_override_feature_activity(
                        pattern.languages.yaml.assist.enabled,
                        pattern.assist.enabled,
                    )
                    .filter(|_| pattern.is_file_included(path))
                });

        overrides_activity
            .or(check_feature_activity(
                settings.languages.yaml.assist.enabled,
                settings.assist.enabled,
            ))
            .unwrap_or_default()
            .into()
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct YamlFileHandler;

impl ExtensionHandler for YamlFileHandler {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            enabled_for_path: EnabledForPath {
                formatter: Some(formatter_enabled),
                search: None,
                assist: Some(assist_enabled),
                linter: Some(linter_enabled),
            },
            parser: ParserCapabilities {
                parse: Some(parse),
                parse_embedded_nodes: None,
            },
            debug: DebugCapabilities {
                debug_syntax_tree: Some(debug_syntax_tree),
                debug_control_flow: None,
                debug_formatter_ir: Some(debug_formatter_ir),
                debug_type_info: None,
                debug_registered_types: None,
                debug_semantic_model: None,
            },
            analyzer: AnalyzerCapabilities {
                lint: Some(lint),
                code_actions: Some(code_actions),
                rename: None,
                fix_all: Some(fix_all),
                update_snippets: None,
                pull_diagnostics_and_actions: None,
            },
            formatter: FormatterCapabilities {
                format: Some(format),
                format_range: Some(format_range),
                format_on_type: Some(format_on_type),
                format_embedded: None,
            },
            search: SearchCapabilities { search: None },
        }
    }
}

fn formatter_enabled(path: &Utf8Path, settings: &Settings) -> bool {
    settings.formatter_enabled_for_file_path::<YamlLanguage>(path)
}

fn linter_enabled(path: &Utf8Path, settings: &Settings) -> bool {
    settings.linter_enabled_for_file_path::<YamlLanguage>(path)
}

fn assist_enabled(path: &Utf8Path, settings: &Settings) -> bool {
    settings.assist_enabled_for_file_path::<YamlLanguage>(path)
}

fn parse(
    _biome_path: &BiomePath,
    _file_source: DocumentFileSource,
    text: &str,
    _settings: &Settings,
    cache: &mut NodeCache,
) -> ParseResult {
    let parse = biome_yaml_parser::parse_yaml_with_cache(text, cache);

    ParseResult {
        any_parse: parse.into(),
        language: Some(_file_source),
    }
}

fn debug_syntax_tree(_rome_path: &BiomePath, parse: AnyParse) -> GetSyntaxTreeResult {
    let syntax: YamlSyntaxNode = parse.syntax();
    let tree: YamlRoot = parse.tree();
    GetSyntaxTreeResult {
        cst: format!("{syntax:#?}"),
        ast: format!("{tree:#?}"),
    }
}

fn debug_formatter_ir(
    path: &BiomePath,
    document_file_source: &DocumentFileSource,
    parse: AnyParse,
    settings: &Settings,
) -> Result<String, WorkspaceError> {
    let options = settings.format_options::<YamlLanguage>(path, document_file_source);

    let tree = parse.syntax();
    let formatted = format_node(options, &tree)?;

    let root_element = formatted.into_document();
    Ok(root_element.to_string())
}

#[tracing::instrument(level = "debug", skip(parse, settings))]
fn format(
    path: &BiomePath,
    document_file_source: &DocumentFileSource,
    parse: AnyParse,
    settings: &Settings,
) -> Result<Printed, WorkspaceError> {
    let options = settings.format_options::<YamlLanguage>(path, document_file_source);

    let tree = parse.syntax();
    let formatted = format_node(options, &tree)?;

    match formatted.print() {
        Ok(printed) => Ok(printed),
        Err(error) => Err(WorkspaceError::FormatError(error.into())),
    }
}

fn format_range(
    path: &BiomePath,
    document_file_source: &DocumentFileSource,
    parse: AnyParse,
    settings: &Settings,
    range: TextRange,
) -> Result<Printed, WorkspaceError> {
    let options = settings.format_options::<YamlLanguage>(path, document_file_source);

    let tree = parse.syntax();
    let printed = biome_yaml_formatter::format_range(options, &tree, range)?;
    Ok(printed)
}

fn format_on_type(
    path: &BiomePath,
    document_file_source: &DocumentFileSource,
    parse: AnyParse,
    settings: &Settings,
    offset: TextSize,
) -> Result<Printed, WorkspaceError> {
    let options = settings.format_options::<YamlLanguage>(path, document_file_source);

    let tree = parse.syntax();

    let range = tree.text_range_with_trivia();
    if offset < range.start() || offset > range.end() {
        return Err(WorkspaceError::FormatError(FormatError::RangeError {
            input: TextRange::at(offset, TextSize::from(0)),
            tree: range,
        }));
    }

    let token = match tree.token_at_offset(offset) {
        // File is empty, do nothing
        TokenAtOffset::None => panic!("empty file"),
        TokenAtOffset::Single(token) => token,
        // The cursor should be right after the closing character that was just typed,
        // select the previous token as the correct one
        TokenAtOffset::Between(token, _) => token,
    };

    let root_node = match token.parent() {
        Some(node) => node,
        None => panic!("found a token with no parent"),
    };

    let printed = biome_yaml_formatter::format_sub_tree(options, &root_node)?;
    Ok(printed)
}

fn lint(params: LintParams) -> LintResults {
    let _ = debug_span!("Linting YAML file", path =? params.path, language =? params.language)
        .entered();
    let Some(file_source) = params
        .language
        .to_yaml_file_source()
        .or(YamlFileSource::try_from(params.path.as_path()).ok())
    else {
        return LintResults {
            errors: 0,
            diagnostics: vec![],
            skipped_diagnostics: 0,
        };
    };
    let root: YamlRoot = params.parse.tree();

    let analyzer_options = params.settings.analyzer_options::<YamlLanguage>(
        params.path,
        &params.language,
        params.suppression_reason.as_deref(),
    );

    let (enabled_rules, disabled_rules, analyzer_options) =
        AnalyzerVisitorBuilder::new(params.settings, analyzer_options)
            .with_only(params.only)
            .with_skip(params.skip)
            .with_path(params.path.as_path())
            .with_enabled_selectors(params.enabled_selectors)
            .with_project_layout(params.project_layout.clone())
            .finish();

    let filter = AnalysisFilter {
        categories: params.categories,
        enabled_rules: Some(enabled_rules.as_slice()),
        disabled_rules: &disabled_rules,
        range: None,
    };

    let mut process_lint = ProcessLint::new(&params);
    let services = YamlAnalyzeServices { file_source };
    let (_, analyze_diagnostics) = analyze(&root, filter, &analyzer_options, services, |signal| {
        process_lint.process_signal(signal)
    });

    let diagnostics = params
        .parse
        .into_serde_diagnostics(params.diagnostic_offset);

    process_lint.into_result(diagnostics, analyze_diagnostics)
}

fn code_actions(params: CodeActionsParams) -> PullActionsResult {
    let CodeActionsParams {
        parse,
        range,
        settings: workspace,
        path,
        module_graph: _,
        project_layout,
        language,
        skip,
        only,
        enabled_rules: rules,
        suppression_reason,
        plugins: _,
        categories,
        action_offset,
        document_services: _,
    } = params;

    let _ = debug_span!("Code actions YAML", range =? range, path =? path).entered();
    let tree: YamlRoot = parse.tree();
    let analyzer_options = workspace.analyzer_options::<YamlLanguage>(
        params.path,
        &params.language,
        suppression_reason.as_deref(),
    );
    let mut actions = Vec::new();
    let (enabled_rules, disabled_rules, analyzer_options) =
        AnalyzerVisitorBuilder::new(params.settings, analyzer_options)
            .with_only(only)
            .with_skip(skip)
            .with_path(path.as_path())
            .with_enabled_selectors(rules)
            .with_project_layout(project_layout)
            .finish();

    let filter = AnalysisFilter {
        categories,
        enabled_rules: Some(enabled_rules.as_slice()),
        disabled_rules: &disabled_rules,
        range,
    };

    let Some(file_source) = language.to_yaml_file_source() else {
        error!("Could not determine the file source of the file");
        return PullActionsResult { actions: vec![] };
    };
    let services = YamlAnalyzeServices { file_source };
    analyze(&tree, filter, &analyzer_options, services, |signal| {
        actions.extend(signal.actions().into_code_action_iter().map(|item| {
            CodeAction {
                category: item.category.clone(),
                rule_name: item
                    .rule_name
                    .map(|(group, name)| (Cow::Borrowed(group), Cow::Borrowed(name))),
                suggestion: item.suggestion,
                offset: action_offset,
            }
        }));

        ControlFlow::<Never>::Continue(())
    });

    PullActionsResult { actions }
}

#[instrument(level = "debug", skip(params))]
fn fix_all(params: FixAllParams) -> Result<FixFileResult, WorkspaceError> {
    let mut tree: YamlRoot = params.parse.tree();

    // Compute final rules (taking `overrides` into account)
    let rules = params.settings.as_linter_rules(params.biome_path.as_path());
    let analyzer_options = params.settings.analyzer_options::<YamlLanguage>(
        params.biome_path,
        &params.document_file_source,
        params.suppression_reason.as_deref(),
    );
    let (enabled_rules, disabled_rules, analyzer_options) =
        AnalyzerVisitorBuilder::new(params.settings, analyzer_options)
            .with_only(params.only)
            .with_skip(params.skip)
            .with_path(params.biome_path.as_path())
            .with_enabled_selectors(params.enabled_rules)
            .with_project_layout(params.project_layout.clone())
            .finish();

    let filter = AnalysisFilter {
        categories: params.rule_categories,
        enabled_rules: Some(enabled_rules.as_slice()),
        disabled_rules: &disabled_rules,
        range: None,
    };

    let Some(file_source) = params
        .document_file_source
        .to_yaml_file_source()
        .or(YamlFileSource::try_from(params.biome_path.as_path()).ok())
    else {
        return Err(extension_error(params.biome_path));
    };

    let mut process_fix_all = ProcessFixAll::new(
        &params,
        rules,
        tree.syntax().text_range_with_trivia().len().into(),
    );
    loop {
        let services = YamlAnalyzeServices { file_source };
        let (action, _) = analyze(&tree, filter, &analyzer_options, services, |signal| {
            process_fix_all.process_signal(signal)
        });

        let result = process_fix_all.process_action(action, |root| {
            tree = match YamlRoot::cast(root) {
                Some(tree) => tree,
                None => return None,
            };
            Some(tree.syntax().text_range_with_trivia().len().into())
        })?;

        if result.is_none() {
            return process_fix_all.finish(|| {
                Ok(if params.should_format {
                    Either::Left(format_node(
                        params.settings.format_options::<YamlLanguage>(
                            params.biome_path,
                            &params.document_file_source,
                        ),
                        tree.syntax(),
                    ))
                } else {
                    Either::Right(tree.syntax().to_string())
                })
            });
        }
    }
}
