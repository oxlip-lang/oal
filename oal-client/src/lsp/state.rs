use super::{Folder, Workspace};
use lsp_server::Connection;

pub struct GlobalState {
    pub conn: Connection,
    pub workspace: Workspace,
    pub folders: Vec<Folder>,
    pub is_stale: bool,
}
