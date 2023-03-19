use anyhow::anyhow;
use lsp_server::{Connection, ExtractError, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, PublishDiagnostics,
};
use lsp_types::request::GotoDefinition;
use lsp_types::OneOf;
use lsp_types::{
    GotoDefinitionParams, GotoDefinitionResponse, InitializeParams, PositionEncodingKind,
    PublishDiagnosticsParams, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
};
use oal_client::lsp::{Folder, Workspace};
use oal_model::locator::Locator;

fn main() -> anyhow::Result<()> {
    // Note that we must have our logging only write out to stderr.
    eprintln!("starting OpenAPI Lang server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (conn, threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        position_encoding: Some(PositionEncodingKind::UTF16),
        definition_provider: Some(OneOf::Left(true)),
        ..Default::default()
    })
    .unwrap();

    let params = conn.initialize(server_capabilities)?;
    let params: InitializeParams = serde_json::from_value(params).unwrap();

    params
        .capabilities
        .general
        .and_then(|c| c.position_encodings)
        .and_then(|e| e.contains(&PositionEncodingKind::UTF16).then_some(()))
        .ok_or_else(|| anyhow!("UTF-16 not supported by client"))?;

    let folders = params
        .workspace_folders
        .unwrap_or_default()
        .into_iter()
        .flat_map(Folder::new)
        .collect::<Vec<_>>();

    let workspace = Workspace::default();

    main_loop(conn, workspace, folders)?;

    threads.join()?;

    // Shut down gracefully.
    eprintln!("shutting down OpenAPI Lang server");
    Ok(())
}

/// Refreshes the folders state following a workspace event.
/// Publishes the diagnostics to the LSP client. 
fn refresh(
    conn: &Connection,
    ws: &Workspace,
    folders: &mut [Folder],
    loc: Option<&Locator>,
) -> anyhow::Result<()> {
    for f in folders.iter_mut() {
        if loc.map(|l| f.contains(l)).unwrap_or(true) {
            f.eval(ws);
            let diags = ws.diagnostics()?;
            for (loc, diagnostics) in diags.into_iter() {
                let info = notify::<PublishDiagnostics>(PublishDiagnosticsParams {
                    uri: loc.url().clone(),
                    diagnostics,
                    version: None,
                });
                conn.sender.send(Message::Notification(info))?;
            }
        }
    }
    Ok(())
}

fn main_loop(conn: Connection, ws: Workspace, mut folders: Vec<Folder>) -> anyhow::Result<()> {
    refresh(&conn, &ws, &mut folders, None)?;

    for msg in &conn.receiver {
        match msg {
            Message::Request(req) => {
                if conn.handle_shutdown(&req)? {
                    return Ok(());
                }
                match cast_request::<GotoDefinition>(req) {
                    Ok((id, params)) => {
                        goto_definition(&conn, id, params)?;
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:#?}"),
                    Err(ExtractError::MethodMismatch(_)) => (),
                };
            }
            Message::Response(_resp) => {}
            Message::Notification(not) => {
                let not = match cast_notification::<DidOpenTextDocument>(not) {
                    Ok(params) => {
                        let loc = ws.open(params)?;
                        refresh(&conn, &ws, &mut folders, Some(&loc))?;
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:#?}"),
                    Err(ExtractError::MethodMismatch(not)) => not,
                };
                let not = match cast_notification::<DidCloseTextDocument>(not) {
                    Ok(params) => {
                        let _loc = ws.close(params)?;
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:#?}"),
                    Err(ExtractError::MethodMismatch(not)) => not,
                };
                let _not = match cast_notification::<DidChangeTextDocument>(not) {
                    Ok(params) => {
                        let loc = ws.change(params)?;
                        refresh(&conn, &ws, &mut folders, Some(&loc))?;
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:#?}"),
                    Err(ExtractError::MethodMismatch(not)) => not,
                };
            }
        }
    }
    Ok(())
}

fn cast_request<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

fn cast_notification<N>(not: Notification) -> Result<N::Params, ExtractError<Notification>>
where
    N: lsp_types::notification::Notification,
    N::Params: serde::de::DeserializeOwned,
{
    not.extract(N::METHOD)
}

fn notify<N>(p: N::Params) -> Notification
where
    N: lsp_types::notification::Notification,
{
    Notification::new(N::METHOD.to_owned(), p)
}

fn goto_definition(
    conn: &Connection,
    id: RequestId,
    _params: GotoDefinitionParams,
) -> anyhow::Result<()> {
    let result = Some(GotoDefinitionResponse::Array(Vec::new()));
    let result = serde_json::to_value(&result).unwrap();
    let resp = Response {
        id,
        result: Some(result),
        error: None,
    };
    conn.sender.send(Message::Response(resp))?;
    Ok(())
}
