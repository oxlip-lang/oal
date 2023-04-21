use anyhow::anyhow;
use crossbeam_channel::select;
use lsp_server::{Connection, Message, Notification};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, PublishDiagnostics,
};
use lsp_types::request::{GotoDefinition, References};
use lsp_types::OneOf;
use lsp_types::{
    InitializeParams, PositionEncodingKind, PublishDiagnosticsParams, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind,
};
use oal_client::lsp::dispatcher::{NotificationDispatcher, RequestDispatcher};
use oal_client::lsp::state::GlobalState;
use oal_client::lsp::{handlers, Folder, Workspace};
use std::time::Duration;

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
        references_provider: Some(OneOf::Left(true)),
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

    let state = &mut GlobalState {
        conn,
        workspace,
        folders,
        is_stale: true,
    };

    main_loop(state)?;

    threads.join()?;

    // Shut down gracefully.
    eprintln!("shutting down OpenAPI Lang server");
    Ok(())
}

fn notify<N>(p: N::Params) -> Notification
where
    N: lsp_types::notification::Notification,
{
    Notification::new(N::METHOD.to_owned(), p)
}

/// Refreshes the folders state following a workspace event.
/// Publishes the diagnostics to the LSP client.
fn refresh(state: &mut GlobalState) -> anyhow::Result<()> {
    if !state.is_stale {
        return Ok(());
    }
    state.is_stale = false;
    for f in state.folders.iter_mut() {
        f.eval(&mut state.workspace);
        let diags = state.workspace.diagnostics()?;
        for (loc, diagnostics) in diags {
            let info = notify::<PublishDiagnostics>(PublishDiagnosticsParams {
                uri: loc.url().clone(),
                diagnostics,
                version: None,
            });
            state.conn.sender.send(Message::Notification(info))?;
        }
    }
    Ok(())
}

fn main_loop(state: &mut GlobalState) -> anyhow::Result<()> {
    loop {
        select! {
            recv(state.conn.receiver) -> msg => {
                match msg? {
                    Message::Request(req) => {
                        if state.conn.handle_shutdown(&req)? {
                            return Ok(());
                        }
                        RequestDispatcher::new(state, req)
                        .on::<GotoDefinition>(handlers::go_to_definition)?
                        .on::<References>(handlers::references)?;
                    }
                    Message::Response(_resp) => {}
                    Message::Notification(not) => {
                        NotificationDispatcher::new(state, not)
                        .on::<DidOpenTextDocument>(|state, params| {
                            state.workspace.open(params)?;
                            state.is_stale = true;
                            Ok(())
                        })?
                        .on::<DidCloseTextDocument>(|state, params| {
                            state.workspace.close(params)?;
                            state.is_stale = true;
                            Ok(())
                        })?
                        .on::<DidChangeTextDocument>(|state, params| {
                            state.workspace.change(params)?;
                            state.is_stale = true;
                            Ok(())
                        })?;
                    }
                }
            },
            default(Duration::from_millis(1000)) => {
                refresh(state)?;
            }
        }
    }
}
