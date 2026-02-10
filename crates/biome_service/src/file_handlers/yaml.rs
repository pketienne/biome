use super::{
    AnalyzerCapabilities, AnalyzerVisitorBuilder, Capabilities, CodeActionsParams,
    DebugCapabilities, DocumentFileSource, EnabledForPath, ExtensionHandler, FixAllParams,
    FormatterCapabilities, LintParams, LintResults, ParseResult, ParserCapabilities, ProcessFixAll,
    ProcessLint, SearchCapabilities,
};
use crate::configuration::to_analyzer_rules;
use crate::settings::{OverrideSettings, check_feature_activity};
use crate::workspace::{FixFileResult, PullActionsResult, RenameResult};
use crate::{
    WorkspaceError,
    settings::{ServiceLanguage, Settings},
    workspace::GetSyntaxTreeResult,
};
use biome_analyze::{AnalysisFilter, AnalyzerConfiguration, AnalyzerOptions, ControlFlow, Never};
use biome_configuration::yaml::{
    YamlAssistConfiguration, YamlAssistEnabled, YamlFormatterConfiguration,
    YamlFormatterEnabled, YamlLinterConfiguration, YamlLinterEnabled,
};
use biome_formatter::{Expand, IndentStyle, IndentWidth, LineEnding, LineWidth, Printed, QuoteStyle};
use biome_fs::BiomePath;
use biome_text_edit::TextEdit;
use biome_yaml_analyze::analyze;
use biome_yaml_formatter::format_node;
use biome_yaml_parser::parse_yaml_with_cache;
use biome_yaml_syntax::{YamlLanguage, YamlSyntaxKind, YamlSyntaxNode, YamlRoot};
use biome_parser::AnyParse;
use biome_rowan::{AstNode, Direction, NodeCache, TextRange, TextSize};
use camino::Utf8Path;
use either::Either;
use std::borrow::Cow;
use tracing::{debug_span, trace_span};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct YamlFormatterSettings {
    pub enabled: Option<YamlFormatterEnabled>,
    pub indent_style: Option<IndentStyle>,
    pub indent_width: Option<IndentWidth>,
    pub line_ending: Option<LineEnding>,
    pub line_width: Option<LineWidth>,
    pub quote_style: Option<QuoteStyle>,
    pub expand: Option<Expand>,
}

impl From<YamlFormatterConfiguration> for YamlFormatterSettings {
    fn from(config: YamlFormatterConfiguration) -> Self {
        Self {
            enabled: config.enabled,
            indent_style: config.indent_style,
            indent_width: config.indent_width,
            line_ending: config.line_ending,
            line_width: config.line_width,
            quote_style: config.quote_style,
            expand: config.expand,
        }
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

pub use biome_yaml_formatter::context::YamlFormatOptions;

impl ServiceLanguage for YamlLanguage {
    type FormatterSettings = YamlFormatterSettings;
    type LinterSettings = YamlLinterSettings;
    type FormatOptions = YamlFormatOptions;
    type ParserSettings = ();
    type EnvironmentSettings = ();
    type AssistSettings = YamlAssistSettings;
    type ParserOptions = ();

    fn lookup_settings(
        languages: &crate::settings::LanguageListSettings,
    ) -> &crate::settings::LanguageSettings<Self> {
        &languages.yaml
    }

    fn resolve_format_options(
        global: &crate::settings::FormatSettings,
        overrides: &OverrideSettings,
        language: &Self::FormatterSettings,
        path: &BiomePath,
        _file_source: &DocumentFileSource,
    ) -> Self::FormatOptions {
        let indent_style = language
            .indent_style
            .or(global.indent_style)
            .unwrap_or(IndentStyle::Space);
        let indent_width = language
            .indent_width
            .or(global.indent_width)
            .unwrap_or_default();
        let line_width = language
            .line_width
            .or(global.line_width)
            .unwrap_or_default();
        let line_ending = language
            .line_ending
            .or(global.line_ending)
            .unwrap_or_default();
        let quote_style = language.quote_style.unwrap_or_default();
        let expand = language.expand.or(global.expand).unwrap_or_default();

        let mut options = YamlFormatOptions::default()
            .with_indent_style(indent_style)
            .with_indent_width(indent_width)
            .with_line_width(line_width)
            .with_line_ending(line_ending)
            .with_quote_style(quote_style)
            .with_expand(expand);

        overrides.apply_override_yaml_format_options(path, &mut options);

        options
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

    fn formatter_enabled_for_file_path(settings: &Settings, _path: &Utf8Path) -> bool {
        check_feature_activity(
            settings.languages.yaml.formatter.enabled,
            settings.formatter.enabled,
        )
        .unwrap_or_default()
        .into()
    }

    fn assist_enabled_for_file_path(settings: &Settings, _path: &Utf8Path) -> bool {
        check_feature_activity(
            settings.languages.yaml.assist.enabled,
            settings.assist.enabled,
        )
        .unwrap_or_default()
        .into()
    }

    fn linter_enabled_for_file_path(settings: &Settings, _path: &Utf8Path) -> bool {
        check_feature_activity(
            settings.languages.yaml.linter.enabled,
            settings.linter.enabled,
        )
        .unwrap_or_default()
        .into()
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
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct YamlFileHandler;

impl ExtensionHandler for YamlFileHandler {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            enabled_for_path: EnabledForPath {
                formatter: Some(formatter_enabled),
                linter: Some(linter_enabled),
                assist: Some(assist_enabled),
                search: Some(search_enabled),
            },
            parser: ParserCapabilities {
                parse: Some(parse),
                parse_embedded_nodes: None,
            },
            debug: DebugCapabilities {
                debug_syntax_tree: Some(debug_syntax_tree),
                debug_control_flow: None,
                debug_formatter_ir: None,
                debug_type_info: None,
                debug_registered_types: None,
                debug_semantic_model: None,
            },
            analyzer: AnalyzerCapabilities {
                lint: Some(lint),
                code_actions: Some(code_actions),
                rename: Some(rename),
                fix_all: Some(fix_all),
                update_snippets: None,
                pull_diagnostics_and_actions: None,
            },
            formatter: FormatterCapabilities {
                format: Some(format),
                format_range: Some(format_range),
                format_on_type: None,
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
    let parse = parse_yaml_with_cache(text, cache);

    ParseResult {
        any_parse: parse.into(),
        language: Some(file_source),
    }
}

fn debug_syntax_tree(_biome_path: &BiomePath, parse: AnyParse) -> GetSyntaxTreeResult {
    let syntax: YamlSyntaxNode = parse.syntax();
    let tree: YamlRoot = parse.tree();
    GetSyntaxTreeResult {
        cst: format!("{syntax:#?}"),
        ast: format!("{tree:#?}"),
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

#[tracing::instrument(level = "debug", skip(params))]
fn lint(params: LintParams) -> LintResults {
    let workspace_settings = &params.settings;
    let analyzer_options = workspace_settings.analyzer_options::<YamlLanguage>(
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

    let (_, analyze_diagnostics) = analyze(&tree, filter, &analyzer_options, |signal| {
        process_lint.process_signal(signal)
    });

    process_lint.into_result(
        params
            .parse
            .into_serde_diagnostics(params.diagnostic_offset),
        analyze_diagnostics,
    )
}

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
        document_services: _,
    } = params;
    let _ = debug_span!("Code actions YAML", range =? range, path =? path).entered();
    let tree = parse.tree();
    let _ = trace_span!("Parsed file", tree =? tree).entered();
    let analyzer_options = settings.analyzer_options::<YamlLanguage>(
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

    analyze(&tree, filter, &analyzer_options, |signal| {
        actions.extend(signal.actions().into_code_action_iter().map(|item| {
            crate::workspace::CodeAction {
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

#[tracing::instrument(level = "debug", skip(params))]
pub(crate) fn fix_all(params: FixAllParams) -> Result<FixFileResult, WorkspaceError> {
    let mut tree: YamlRoot = params.parse.tree();

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

    let mut process_fix_all = ProcessFixAll::new(
        &params,
        rules,
        tree.syntax().text_range_with_trivia().len().into(),
    );

    loop {
        let (action, _) = analyze(&tree, filter, &analyzer_options, |signal| {
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
                Ok(Either::<biome_formatter::FormatResult<biome_formatter::Formatted<biome_formatter::SimpleFormatContext>>, String>::Right(tree.syntax().to_string()))
            });
        }
    }
}

fn rename(
    _path: &BiomePath,
    parse: AnyParse,
    symbol_at: TextSize,
    new_name: String,
) -> Result<RenameResult, WorkspaceError> {
    let syntax: YamlSyntaxNode = parse.syntax();

    // Find the token at the cursor position
    let token = syntax
        .descendants_tokens(Direction::Next)
        .find(|token| token.text_range().contains(symbol_at));

    let Some(token) = token else {
        return Err(WorkspaceError::RenameError(
            biome_js_analyze::utils::rename::RenameError::CannotFindDeclaration(new_name),
        ));
    };

    // Check if the token is an anchor or alias
    let prefix = match token.kind() {
        YamlSyntaxKind::ANCHOR_PROPERTY_LITERAL => '&',
        YamlSyntaxKind::ALIAS_LITERAL => '*',
        _ => {
            return Err(WorkspaceError::RenameError(
                biome_js_analyze::utils::rename::RenameError::CannotBeRenamed {
                    original_name: token.text_trimmed().to_string(),
                    original_range: token.text_range(),
                    new_name,
                },
            ));
        }
    };

    // Extract the bare anchor/alias name (without & or * prefix)
    let text = token.text_trimmed();
    let old_name = text
        .strip_prefix(prefix)
        .unwrap_or(text)
        .to_string();

    // Find the containing YAML document (for multi-doc scoping)
    let document_node = token
        .parent()
        .and_then(|node| {
            node.ancestors()
                .find(|n| n.kind() == YamlSyntaxKind::YAML_DOCUMENT)
        });

    // Search within the document (or root if no document found)
    let search_root = document_node.unwrap_or_else(|| syntax.clone());

    // Collect all anchor and alias tokens with the matching name
    let mut edits: Vec<(TextRange, String)> = Vec::new();

    for descendant_token in search_root.descendants_tokens(Direction::Next) {
        let kind = descendant_token.kind();
        let (pfx, is_anchor_or_alias) = match kind {
            YamlSyntaxKind::ANCHOR_PROPERTY_LITERAL => ('&', true),
            YamlSyntaxKind::ALIAS_LITERAL => ('*', true),
            _ => ('\0', false),
        };

        if !is_anchor_or_alias {
            continue;
        }

        let dt_text = descendant_token.text_trimmed();
        let dt_name = dt_text.strip_prefix(pfx).unwrap_or(dt_text);

        if dt_name == old_name {
            // Compute range of just the name portion (after the prefix character)
            let full_range = descendant_token.text_trimmed_range();
            let name_start = full_range.start() + TextSize::from(1); // skip & or *
            let name_range = TextRange::new(name_start, full_range.end());
            edits.push((name_range, new_name.clone()));
        }
    }

    if edits.is_empty() {
        return Err(WorkspaceError::RenameError(
            biome_js_analyze::utils::rename::RenameError::CannotFindDeclaration(new_name),
        ));
    }

    // Build the new source text with all replacements applied
    let source = syntax.to_string();
    let mut new_source = String::with_capacity(source.len());
    let mut last_offset = TextSize::from(0);

    // Sort edits by start position
    let mut edits = edits;
    edits.sort_by_key(|(range, _)| range.start());

    for (range, replacement) in &edits {
        let start: usize = range.start().into();
        let end: usize = range.end().into();
        let last: usize = last_offset.into();
        new_source.push_str(&source[last..start]);
        new_source.push_str(replacement);
        last_offset = TextSize::from(end as u32);
    }
    let last: usize = last_offset.into();
    new_source.push_str(&source[last..]);

    // Compute the overall affected range
    let first_start = edits.first().unwrap().0.start();
    let last_end = edits.last().unwrap().0.end();
    // Adjust last_end for any length difference from replacements
    let total_old_len: usize = edits.iter().map(|(r, _)| usize::from(r.len())).sum::<usize>();
    let total_new_len: usize = edits.len() * new_name.len();
    let len_diff = total_new_len as i64 - total_old_len as i64;
    let adjusted_end = (u32::from(last_end) as i64 + len_diff) as u32;
    let result_range = TextRange::new(first_start, TextSize::from(adjusted_end));

    let indels = TextEdit::from_unicode_words(&source, &new_source);

    Ok(RenameResult {
        range: result_range,
        indels,
    })
}
