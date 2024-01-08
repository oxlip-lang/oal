use super::{Folder, Workspace};
use lsp_server::Connection;
use std::collections::HashMap;
use url::Url;

pub struct GlobalState {
    pub conn: Connection,
    pub workspace: Workspace,
    pub folders: HashMap<Url, Folder>,
    pub is_stale: bool,
}
