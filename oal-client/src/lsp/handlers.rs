use super::state::GlobalState;
use lsp_server::{Message, RequestId, Response};
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse};

pub fn goto_definition(
    state: &mut GlobalState,
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
    state.conn.sender.send(Message::Response(resp))?;
    Ok(())
}
