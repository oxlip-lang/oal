use anyhow::anyhow;
use crossbeam_channel::select;
use lsp_server::{Connection, Message, Notification};
use lsp_types::notification::{
    DidChangeTextDocument, DidChangeWorkspaceFolders, DidCloseTextDocument, DidOpenTextDocument,
    PublishDiagnostics,
};
use lsp_types::request::{GotoDefinition, PrepareRenameRequest, References, Rename};
use lsp_types::{
    InitializeParams, PositionEncodingKind, PublishDiagnosticsParams, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, WorkspaceFileOperationsServerCapabilities,
    WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities,
};
use lsp_types::{OneOf, RenameOptions};
use oal_client::lsp::dispatcher::{NotificationDispatcher, RequestDispatcher};
use oal_client::lsp::state::GlobalState;
use oal_client::lsp::{handlers, Folder, Workspace};
use std::collections::HashMap;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // Note that we must have our logging only write out to stderr.
    eprintln!("starting Oxlip API Language server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (conn, threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        position_encoding: Some(PositionEncodingKind::UTF16),
        definition_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        rename_provider: Some(OneOf::Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: Default::default(),
        })),
        workspace: Some(WorkspaceServerCapabilities {
            workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(OneOf::Left(true)),
            }),
            file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                // TODO: register for operations on configuration files
                ..Default::default()
            }),
        }),
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

    let mut folders = HashMap::new();
    for f in params.workspace_folders.unwrap_or_default().into_iter() {
        let uri = f.uri.clone();
        if let Ok(folder) = Folder::new(f) {
            folders.insert(uri, folder);
        }
    }

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
    eprintln!("shutting down Oxlip API Language server");
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
    for (_, f) in state.folders.iter_mut() {
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
                        refresh(state)?;
                        RequestDispatcher::new(state, req)
                        .on::<GotoDefinition, _>(handlers::go_to_definition)?
                        .on::<References, _>(handlers::references)?
                        .on::<PrepareRenameRequest, _>(handlers::prepare_rename)?
                        .on::<Rename, _>(handlers::rename)?;
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
                        })?
                        .on::<DidChangeWorkspaceFolders>(|state, params| {
                            for f in params.event.removed {
                                state.folders.remove(&f.uri);
                            }
                            for f in params.event.added {
                                let uri = f.uri.clone();
                                if let Ok(folder) = Folder::new(f) {
                                    state.folders.insert(uri, folder);
                                }
                            }
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
