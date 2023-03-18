// Base example taken from:
// https://github.com/rust-lang/rust-analyzer/blob/master/lib/lsp-server/examples/goto_def.rs

//! A minimal example LSP server that can only respond to the `gotoDefinition` request. To use
//! this example, execute it and then send an `initialize` request.
//!
//! ```no_run
//! Content-Length: 85
//!
//! {"jsonrpc": "2.0", "method": "initialize", "id": 1, "params": {"capabilities": {}}}
//! ```
//!
//! This will respond with a server response. Then send it a `initialized` notification which will
//! have no response.
//!
//! ```no_run
//! Content-Length: 59
//!
//! {"jsonrpc": "2.0", "method": "initialized", "params": {}}
//! ```
//!
//! Once these two are sent, then we enter the main loop of the server. The only request this
//! example can handle is `gotoDefinition`:
//!
//! ```no_run
//! Content-Length: 159
//!
//! {"jsonrpc": "2.0", "method": "textDocument/definition", "id": 2, "params": {"textDocument": {"uri": "file://temp"}, "position": {"line": 1, "character": 1}}}
//! ```
//!
//! To finish up without errors, send a shutdown request:
//!
//! ```no_run
//! Content-Length: 67
//!
//! {"jsonrpc": "2.0", "method": "shutdown", "id": 3, "params": null}
//! ```
//!
//! The server will exit the main loop and finally we send a `shutdown` notification to stop
//! the server.
//!
//! ```
//! Content-Length: 54
//!
//! {"jsonrpc": "2.0", "method": "exit", "params": null}
//! ```
use anyhow::anyhow;
use lsp_server::{Connection, ExtractError, Message, Request, RequestId, Response};
use lsp_types::OneOf;
use lsp_types::{
    request::GotoDefinition, GotoDefinitionResponse, InitializeParams, ServerCapabilities,
};
use oal_client::config::Config;
use oal_client::Context;
use oal_compiler::module::ModuleSet;
use oal_compiler::spec::Spec;
use url::Url;

fn main() -> anyhow::Result<()> {
    // Note that  we must have our logging only write out to stderr.
    eprintln!("starting generic LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        definition_provider: Some(OneOf::Left(true)),
        ..Default::default()
    })
    .unwrap();
    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    // Shut down gracefully.
    eprintln!("shutting down server");
    Ok(())
}

#[derive(Debug)]
struct Folder {
    uri: Url,
    config: Config,
    mods: Option<ModuleSet>,
    spec: Option<Spec>,
}

impl Folder {
    fn new(folder: lsp_types::WorkspaceFolder) -> anyhow::Result<Self> {
        const DEFAULT_CONFIG_FILE: &str = "oal.toml";
        if folder.uri.scheme() != "file" {
            Err(anyhow!("not a file"))
        } else {
            let mut uri = folder.uri;
            // The original URL can be a base so path_segments_mut should never fail.
            uri.path_segments_mut().unwrap().push(DEFAULT_CONFIG_FILE);
            let path = uri.to_file_path().map_err(|_| anyhow!("not a path"))?;
            let config = Config::new(Some(path.as_path()))?;
            let mods = None;
            let spec = None;
            Ok(Folder {
                uri,
                config,
                mods,
                spec,
            })
        }
    }
}

fn main_loop(connection: Connection, params: serde_json::Value) -> anyhow::Result<()> {
    let params: InitializeParams = serde_json::from_value(params).unwrap();

    eprintln!("starting main loop with: {params:#?}");

    let mut ctx = Context::new(std::io::stderr());

    let mut folders = params
        .workspace_folders
        .unwrap_or_default()
        .into_iter()
        .map(Folder::new)
        .collect::<anyhow::Result<Vec<_>>>()?;

    for f in folders.iter_mut() {
        eprintln!("loading configuration at {}", f.uri);
        let main = f.config.main()?;
        f.mods = oal_compiler::module::load(&mut ctx, &main).ok();
        if let Some(ref m) = f.mods {
            f.spec = ctx.eval(m).ok();
        }
        eprintln!("done");
    }

    for msg in &connection.receiver {
        eprintln!("got msg: {msg:?}");
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                eprintln!("got request: {req:#?}");
                match cast::<GotoDefinition>(req) {
                    Ok((id, params)) => {
                        eprintln!("got gotoDefinition request #{id}: {params:#?}");
                        let result = Some(GotoDefinitionResponse::Array(Vec::new()));
                        let result = serde_json::to_value(&result).unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:#?}"),
                    Err(ExtractError::MethodMismatch(req)) => req,
                };
                // ...
            }
            Message::Response(resp) => {
                eprintln!("got response: {resp:#?}");
            }
            Message::Notification(not) => {
                eprintln!("got notification: {not:#?}");
            }
        }
    }
    Ok(())
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}
