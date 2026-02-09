use super::{
    AnalyzerVisitorBuilder, CodeActionsParams, DocumentFileSource, EnabledForPath,
    ExtensionHandler, FixAllParams, LintParams, LintResults, ParseResult, ProcessFixAll,
    ProcessLint, SearchCapabilities,
};
use crate::WorkspaceError;
use crate::configuration::to_analyzer_rules;
use crate::file_handlers::DebugCapabilities;
use crate::file_handlers::{
    AnalyzerCapabilities, Capabilities, FormatterCapabilities, ParserCapabilities,
};
use crate::settings::{
    FormatSettings, LanguageListSettings, LanguageSettings, OverrideSettings, ServiceLanguage,
    Settings, check_feature_activity, check_override_feature_activity,
};
use crate::workspace::{CodeAction, FixFileResult, GetSyntaxTreeResult, PullActionsResult};
use biome_analyze::{AnalysisFilter, AnalyzerConfiguration, AnalyzerOptions, ControlFlow, Never};
use biome_configuration::turtle::{
    TurtleAssistConfiguration, TurtleAssistEnabled, TurtleDirectiveStyle,
    TurtleFormatterConfiguration, TurtleFormatterEnabled, TurtleLinterConfiguration,
    TurtleLinterEnabled,
};
use biome_formatter::{FormatError, IndentStyle, IndentWidth, LineEnding, LineWidth, Printed, QuoteStyle};
use biome_fs::BiomePath;
use biome_turtle_analyze::analyze;
use biome_turtle_formatter::context::TurtleFormatOptions;
use biome_turtle_formatter::format_node;
use biome_turtle_parser::parse_turtle_with_cache;
use biome_turtle_syntax::{TurtleLanguage, TurtleRoot, TurtleSyntaxNode, TextRange, TextSize};
use biome_parser::AnyParse;
use biome_rowan::{AstNode, NodeCache, TokenAtOffset};
use camino::Utf8Path;
use either::Either;
use std::borrow::Cow;
use tracing::{debug_span, error, info, trace_span};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TurtleFormatterSettings {
    pub line_ending: Option<LineEnding>,
    pub line_width: Option<LineWidth>,
    pub indent_width: Option<IndentWidth>,
    pub indent_style: Option<IndentStyle>,
    pub enabled: Option<TurtleFormatterEnabled>,
    pub quote_style: Option<QuoteStyle>,
    pub first_predicate_in_new_line: Option<bool>,
    pub directive_style: Option<TurtleDirectiveStyle>,
}

impl From<TurtleFormatterConfiguration> for TurtleFormatterSettings {
    fn from(configuration: TurtleFormatterConfiguration) -> Self {
        Self {
            line_ending: configuration.line_ending,
            line_width: configuration.line_width,
            indent_width: configuration.indent_width,
            indent_style: configuration.indent_style,
            enabled: configuration.enabled,
            quote_style: configuration.quote_style,
            first_predicate_in_new_line: configuration.first_predicate_in_new_line,
            directive_style: configuration.directive_style,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TurtleLinterSettings {
    pub enabled: Option<TurtleLinterEnabled>,
}

impl From<TurtleLinterConfiguration> for TurtleLinterSettings {
    fn from(configuration: TurtleLinterConfiguration) -> Self {
        Self {
            enabled: configuration.enabled,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TurtleAssistSettings {
    pub enabled: Option<TurtleAssistEnabled>,
}

impl From<TurtleAssistConfiguration> for TurtleAssistSettings {
    fn from(configuration: TurtleAssistConfiguration) -> Self {
        Self {
            enabled: configuration.enabled,
        }
    }
}

impl ServiceLanguage for TurtleLanguage {
    type FormatterSettings = TurtleFormatterSettings;
    type LinterSettings = TurtleLinterSettings;
    type AssistSettings = TurtleAssistSettings;
    type FormatOptions = TurtleFormatOptions;
    type ParserSettings = ();
    type ParserOptions = ();
    type EnvironmentSettings = ();

    fn lookup_settings(language: &LanguageListSettings) -> &LanguageSettings<Self> {
        &language.turtle
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
        language: &Self::FormatterSettings,
        _path: &BiomePath,
        document_file_source: &DocumentFileSource,
    ) -> Self::FormatOptions {
        let indent_style = language
            .indent_style
            .or(global.indent_style)
            .unwrap_or_default();
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

        let quote_style = language
            .quote_style
            .unwrap_or_default();
        let first_predicate_in_new_line = language
            .first_predicate_in_new_line
            .unwrap_or(true);
        let directive_style = language
            .directive_style
            .unwrap_or_default();

        TurtleFormatOptions::new(
            document_file_source
                .to_turtle_file_source()
                .unwrap_or_default(),
        )
        .with_indent_style(indent_style)
        .with_indent_width(indent_width)
        .with_line_width(line_width)
        .with_line_ending(line_ending)
        .with_quote_style(quote_style)
        .with_first_predicate_in_new_line(first_predicate_in_new_line)
        .with_directive_style(directive_style)
    }

    fn resolve_analyzer_options(
        global: &Settings,
        _language: &Self::LinterSettings,
        _environment: Option<&Self::EnvironmentSettings>,
        path: &BiomePath,
        _file_source: &DocumentFileSource,
        suppression_reason: Option<&str>,
    ) -> AnalyzerOptions {
        let configuration =
            AnalyzerConfiguration::default().with_rules(to_analyzer_rules(global, path.as_path()));
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
                        pattern.languages.turtle.linter.enabled,
                        pattern.linter.enabled,
                    )
                    .filter(|_| pattern.is_file_included(path))
                });

        overrides_activity
            .or(check_feature_activity(
                settings.languages.turtle.linter.enabled,
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
                        pattern.languages.turtle.formatter.enabled,
                        pattern.formatter.enabled,
                    )
                    .filter(|_| pattern.is_file_included(path))
                });

        overrides_activity
            .or(check_feature_activity(
                settings.languages.turtle.formatter.enabled,
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
                        pattern.languages.turtle.assist.enabled,
                        pattern.assist.enabled,
                    )
                    .filter(|_| pattern.is_file_included(path))
                });

        overrides_activity
            .or(check_feature_activity(
                settings.languages.turtle.assist.enabled,
                settings.assist.enabled,
            ))
            .unwrap_or_default()
            .into()
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct TurtleFileHandler;

impl ExtensionHandler for TurtleFileHandler {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            enabled_for_path: EnabledForPath {
                formatter: Some(formatter_enabled),
                assist: Some(assist_enabled),
                linter: Some(linter_enabled),
                search: Some(search_enabled),
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
                debug_semantic_model: Some(debug_semantic_model),
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
    settings.formatter_enabled_for_file_path::<TurtleLanguage>(path)
}

fn linter_enabled(path: &Utf8Path, settings: &Settings) -> bool {
    settings.linter_enabled_for_file_path::<TurtleLanguage>(path)
}

fn assist_enabled(path: &Utf8Path, settings: &Settings) -> bool {
    settings.assist_enabled_for_file_path::<TurtleLanguage>(path)
}

fn search_enabled(_path: &Utf8Path, _settings: &Settings) -> bool {
    true
}

fn parse(
    _biome_path: &BiomePath,
    file_source: DocumentFileSource,
    text: &str,
    _settings: &Settings,
    cache: &mut NodeCache,
) -> ParseResult {
    let parse = parse_turtle_with_cache(text, cache);

    ParseResult {
        any_parse: parse.into(),
        language: Some(file_source),
    }
}

fn debug_syntax_tree(_rome_path: &BiomePath, parse: AnyParse) -> GetSyntaxTreeResult {
    let syntax: TurtleSyntaxNode = parse.syntax();
    let tree: TurtleRoot = parse.tree();
    GetSyntaxTreeResult {
        cst: format!("{syntax:#?}"),
        ast: format!("{tree:#?}"),
    }
}

fn debug_formatter_ir(
    biome_path: &BiomePath,
    document_file_source: &DocumentFileSource,
    parse: AnyParse,
    settings: &Settings,
) -> Result<String, WorkspaceError> {
    let options = settings.format_options::<TurtleLanguage>(biome_path, document_file_source);

    let tree = parse.syntax();
    let formatted = format_node(options, &tree)?;

    let root_element = formatted.into_document();
    Ok(root_element.to_string())
}

fn debug_semantic_model(_path: &BiomePath, parse: AnyParse) -> Result<String, WorkspaceError> {
    let tree: TurtleRoot = parse.tree();
    let model = biome_turtle_semantic::semantic_model(&tree);
    Ok(model.to_string())
}

#[tracing::instrument(level = "debug", skip(parse, settings))]
fn format(
    biome_path: &BiomePath,
    document_file_source: &DocumentFileSource,
    parse: AnyParse,
    settings: &Settings,
) -> Result<Printed, WorkspaceError> {
    let options = settings.format_options::<TurtleLanguage>(biome_path, document_file_source);

    let tree = parse.syntax();
    let formatted = format_node(options, &tree)?;

    match formatted.print() {
        Ok(printed) => Ok(printed),
        Err(error) => Err(WorkspaceError::FormatError(error.into())),
    }
}

fn format_range(
    biome_path: &BiomePath,
    document_file_source: &DocumentFileSource,
    parse: AnyParse,
    settings: &Settings,
    range: TextRange,
) -> Result<Printed, WorkspaceError> {
    let options = settings.format_options::<TurtleLanguage>(biome_path, document_file_source);

    let tree = parse.syntax();
    let printed = biome_turtle_formatter::format_range(options, &tree, range)?;
    Ok(printed)
}

fn format_on_type(
    biome_path: &BiomePath,
    document_file_source: &DocumentFileSource,
    parse: AnyParse,
    settings: &Settings,
    offset: TextSize,
) -> Result<Printed, WorkspaceError> {
    let options = settings.format_options::<TurtleLanguage>(biome_path, document_file_source);

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

    let printed = biome_turtle_formatter::format_sub_tree(options, &root_node)?;
    Ok(printed)
}

fn lint(params: LintParams) -> LintResults {
    let _ = debug_span!("Linting Turtle file", path =? params.path, language =? params.language)
        .entered();
    let workspace_settings = &params.settings;
    let analyzer_options = workspace_settings.analyzer_options::<TurtleLanguage>(
        params.path,
        &params.language,
        params.suppression_reason.as_deref(),
    );
    let tree = params.parse.tree();

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

    let turtle_services = biome_turtle_analyze::TurtleAnalyzerServices {
        semantic_model: params
            .document_services
            .as_turtle_services()
            .and_then(|services| services.semantic_model.as_ref()),
    };

    let (_, analyze_diagnostics) = analyze(&tree, filter, &analyzer_options, turtle_services, |signal| {
        process_lint.process_signal(signal)
    });

    process_lint.into_result(
        params
            .parse
            .into_serde_diagnostics(params.diagnostic_offset),
        analyze_diagnostics,
    )
}

#[tracing::instrument(level = "debug", skip(params))]
pub(crate) fn code_actions(params: CodeActionsParams) -> PullActionsResult {
    let CodeActionsParams {
        parse,
        range,
        settings,
        path,
        module_graph: _,
        project_layout,
        language,
        only,
        skip,
        suppression_reason,
        enabled_rules: rules,
        plugins: _,
        categories,
        action_offset,
        document_services,
    } = params;
    let _ = debug_span!("Code actions Turtle", range =? range, path =? path).entered();
    let tree = parse.tree();
    let _ = trace_span!("Parsed file", tree =? tree).entered();
    let Some(_) = language.to_turtle_file_source() else {
        error!("Could not determine the file source of the file");
        return PullActionsResult {
            actions: Vec::new(),
        };
    };

    let analyzer_options = settings.analyzer_options::<TurtleLanguage>(
        path,
        &language,
        suppression_reason.as_deref(),
    );
    let mut actions = Vec::new();
    let (enabled_rules, disabled_rules, analyzer_options) =
        AnalyzerVisitorBuilder::new(settings, analyzer_options)
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

    info!("Turtle runs the analyzer");

    let turtle_services = biome_turtle_analyze::TurtleAnalyzerServices {
        semantic_model: document_services
            .as_turtle_services()
            .and_then(|services| services.semantic_model.as_ref()),
    };

    analyze(&tree, filter, &analyzer_options, turtle_services, |signal| {
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

/// If applies all the safe fixes to the given syntax tree.
pub(crate) fn fix_all(params: FixAllParams) -> Result<FixFileResult, WorkspaceError> {
    let mut tree: TurtleRoot = params.parse.tree();

    // Compute final rules (taking `overrides` into account)
    let rules = params.settings.as_linter_rules(params.biome_path.as_path());
    let analyzer_options = params.settings.analyzer_options::<TurtleLanguage>(
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

    let mut process_fix_all = ProcessFixAll::new(
        &params,
        rules,
        tree.syntax().text_range_with_trivia().len().into(),
    );
    loop {
        let turtle_services = biome_turtle_analyze::TurtleAnalyzerServices {
            semantic_model: params
                .document_services
                .as_turtle_services()
                .and_then(|services| services.semantic_model.as_ref()),
        };

        let (action, _) = analyze(&tree, filter, &analyzer_options, turtle_services, |signal| {
            process_fix_all.process_signal(signal)
        });

        let result = process_fix_all.process_action(action, |root| {
            tree = match TurtleRoot::cast(root) {
                Some(tree) => tree,
                None => return None,
            };
            Some(tree.syntax().text_range_with_trivia().len().into())
        })?;

        if result.is_none() {
            return process_fix_all.finish(|| {
                Ok(if params.should_format {
                    Either::Left(format_node(
                        params.settings.format_options::<TurtleLanguage>(
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
