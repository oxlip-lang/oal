use super::state::GlobalState;
use super::utf16_range;
use crate::utf16::char_index;
use lsp_server::{Message, RequestId, Response};
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location};
use oal_compiler::definition::Definition;
use oal_compiler::tree::{Core, Tree};
use oal_model::locator::Locator;
use oal_syntax::parser::Variable;

/// Returns the variable syntax node at the given position, if any.
fn variable_at(tree: &Tree, pos: usize) -> Option<Variable<Core>> {
    tree.root()
        .descendants()
        .filter_map(oal_syntax::parser::Variable::cast)
        .find(|v| v.node().span().unwrap().range().contains(&pos))
}

/// Implements the go-to-definition capability.
pub fn go_to_definition(
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
        if let Some(v) = variable_at(tree, index) {
            if let Some(Definition::External(ext)) = v.node().syntax().core_ref().definition() {
                let definition = ext.node(folder.modules().unwrap());
                let span = definition.span().unwrap();
                let text = state.workspace.read_file(span.locator())?;
                let range = utf16_range(&text, span.range())?;
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
