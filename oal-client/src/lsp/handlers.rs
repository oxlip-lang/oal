use super::lsp_range;
use super::state::GlobalState;
use crate::utf16::char_index;
use lsp_server::{Message, RequestId, Response};
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location};
use oal_compiler::definition::Definition;
use oal_model::locator::Locator;

pub fn goto_definition(
    state: &mut GlobalState,
    id: RequestId,
    params: GotoDefinitionParams,
) -> anyhow::Result<()> {
    let mut res = GotoDefinitionResponse::Array(Vec::new());

    let pos = params.text_document_position_params.position;
    let loc = Locator::from(params.text_document_position_params.text_document.uri);

    if let Some(folder) = state.folders.iter().find(|f| f.contains(&loc)) {
        let tree = folder.module(&loc).unwrap();
        let text = state.workspace.read_file(&loc)?;
        let index = char_index(&text, pos);

        // TODO: pre-compute variable spans in each folders
        let vars: Vec<_> = tree
            .root()
            .descendants()
            .filter_map(oal_syntax::parser::Variable::cast)
            .map(|v| (v.clone(), v.node().span().unwrap()))
            .collect();

        if let Some((v, _)) = vars.iter().find(|(_, s)| s.range().contains(&index)) {
            if let Some(Definition::External(ext)) = v.node().syntax().core_ref().definition() {
                let definition = ext.node(folder.modules().unwrap());
                let span = definition.span().unwrap();
                let text = state.workspace.read_file(span.locator())?;
                let range = lsp_range(&text, span.range())?;
                res = GotoDefinitionResponse::Scalar(Location::new(
                    span.locator().url().clone(),
                    range,
                ));
            }
        }
    }

    let result = Some(res);
    let result = serde_json::to_value(&result).unwrap();
    let resp = Response {
        id,
        result: Some(result),
        error: None,
    };

    state.conn.sender.send(Message::Response(resp))?;

    Ok(())
}
