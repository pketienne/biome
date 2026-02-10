use crate::diagnostics::LspError;
use crate::session::Session;
use biome_lsp_converters::{from_proto, to_proto};
use biome_service::workspace;
use tower_lsp_server::ls_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
    GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents, HoverParams, Location,
    MarkupContent, MarkupKind,
};
use tracing::{info, instrument};

#[instrument(level = "debug", skip_all, err)]
pub(crate) fn hover(
    session: &Session,
    params: HoverParams,
) -> Result<Option<Hover>, LspError> {
    info!("Hover request");
    let url = params.text_document_position_params.text_document.uri;
    let path = session.file_path(&url)?;
    let Some(doc) = session.document(&url) else {
        return Ok(None);
    };

    if !session.workspace.file_exists(path.clone().into())? {
        return Ok(None);
    }

    let position_encoding = session.position_encoding();
    let offset = from_proto::offset(
        &doc.line_index,
        params.text_document_position_params.position,
        position_encoding,
    )?;

    let result = session.workspace.hover(workspace::HoverParams {
        project_key: doc.project_key,
        path,
        offset,
    })?;

    if result.content.is_empty() {
        return Ok(None);
    }

    let range = result
        .range
        .and_then(|r| to_proto::range(&doc.line_index, r, position_encoding).ok());

    Ok(Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: result.content,
        }),
        range,
    }))
}

#[instrument(level = "debug", skip_all, err)]
pub(crate) fn goto_definition(
    session: &Session,
    params: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>, LspError> {
    info!("Goto definition request");
    let url = params
        .text_document_position_params
        .text_document
        .uri
        .clone();
    let path = session.file_path(&url)?;
    let Some(doc) = session.document(&url) else {
        return Ok(None);
    };

    if !session.workspace.file_exists(path.clone().into())? {
        return Ok(None);
    }

    let position_encoding = session.position_encoding();
    let offset = from_proto::offset(
        &doc.line_index,
        params.text_document_position_params.position,
        position_encoding,
    )?;

    let result = session
        .workspace
        .goto_definition(workspace::GotoDefinitionParams {
            project_key: doc.project_key,
            path,
            offset,
        })?;

    if result.definitions.is_empty() {
        return Ok(None);
    }

    let locations: Vec<Location> = result
        .definitions
        .iter()
        .filter_map(|text_range| {
            let lsp_range =
                to_proto::range(&doc.line_index, *text_range, position_encoding).ok()?;
            Some(Location::new(url.clone(), lsp_range))
        })
        .collect();

    if locations.len() == 1 {
        Ok(Some(GotoDefinitionResponse::Scalar(locations[0].clone())))
    } else {
        Ok(Some(GotoDefinitionResponse::Array(locations)))
    }
}

#[instrument(level = "debug", skip_all, err)]
pub(crate) fn completions(
    session: &Session,
    params: CompletionParams,
) -> Result<Option<CompletionResponse>, LspError> {
    info!("Completions request");
    let url = params.text_document_position.text_document.uri;
    let path = session.file_path(&url)?;
    let Some(doc) = session.document(&url) else {
        return Ok(None);
    };

    if !session.workspace.file_exists(path.clone().into())? {
        return Ok(None);
    }

    let position_encoding = session.position_encoding();
    let offset = from_proto::offset(
        &doc.line_index,
        params.text_document_position.position,
        position_encoding,
    )?;

    let result = session
        .workspace
        .get_completions(workspace::GetCompletionsParams {
            project_key: doc.project_key,
            path,
            offset,
        })?;

    if result.items.is_empty() {
        return Ok(None);
    }

    let items: Vec<CompletionItem> = result
        .items
        .into_iter()
        .map(|item| CompletionItem {
            label: item.label,
            detail: item.detail,
            kind: Some(CompletionItemKind::REFERENCE),
            ..Default::default()
        })
        .collect();

    Ok(Some(CompletionResponse::Array(items)))
}
