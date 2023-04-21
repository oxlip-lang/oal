use super::state::GlobalState;
use super::{utf16_range, Workspace};
use crate::utf16::char_index;
use anyhow::anyhow;
use lsp_server::{Connection, Message, RequestId, Response};
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location, ReferenceParams};
use oal_compiler::definition::{Definition, External};
use oal_compiler::tree::{Core, NRef, Tree};
use oal_model::grammar::AbstractSyntaxNode;
use oal_model::locator::Locator;
use oal_syntax::parser::{Declaration, Gram, Identifier, Variable};
use serde::Serialize;

/// Returns the abstract syntax node at the given position, if any.
fn syntax_at<'a, N>(tree: &'a Tree, pos: usize) -> Option<N>
where
    N: AbstractSyntaxNode<'a, Core, Gram>,
{
    tree.root()
        .descendants()
        .filter_map(N::cast)
        .find(|i| i.node().span().unwrap().range().contains(&pos))
}

fn node_location(workspace: &mut Workspace, node: NRef) -> anyhow::Result<Location> {
    let span = node.span().unwrap();
    let text = workspace.read_file(span.locator())?;
    let range = utf16_range(&text, span.range());
    Ok(Location::new(span.locator().url().clone(), range))
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
        if let Some(v) = syntax_at::<Variable<_>>(tree, index) {
            if let Some(Definition::External(ext)) = v.node().syntax().core_ref().definition() {
                let definition = ext.node(folder.modules().unwrap());
                let location = node_location(&mut state.workspace, definition)?;
                res = GotoDefinitionResponse::Scalar(location);
            }
        }
    }

    send_response(&state.conn, id, Some(res))?;

    Ok(())
}

/// Implements the references capability.
pub fn references(
    state: &mut GlobalState,
    id: RequestId,
    params: ReferenceParams,
) -> anyhow::Result<()> {
    let mut res: Vec<Location> = Vec::new();

    let pos = params.text_document_position.position;
    let loc = Locator::from(params.text_document_position.text_document.uri);

    if let Some(folder) = state.folders.iter().find(|f| f.contains(&loc)) {
        let tree = folder.module(&loc).unwrap();
        let text = state.workspace.read_file(&loc)?;
        let index = char_index(&text, pos);
        if let Some(ident) = syntax_at::<Identifier<_>>(tree, index) {
            let parent = ident.node().ancestors().nth(1).unwrap();
            let definition = if let Some(decl) = Declaration::cast(parent) {
                Some(Definition::External(External::new(tree, decl.node())))
            } else if let Some(var) = Variable::cast(parent) {
                var.node().syntax().core_ref().definition().cloned()
            } else {
                None
            }
            .ok_or_else(|| anyhow!("neither a variable or a declaration"))?;

            for module in folder.modules().unwrap().modules() {
                for var in module.root().descendants().filter_map(Variable::cast) {
                    if definition == *var.node().syntax().core_ref().definition().unwrap() {
                        let location = node_location(&mut state.workspace, var.node())?;
                        res.push(location);
                    }
                }
            }
        }
    }

    send_response(&state.conn, id, Some(res))?;
    Ok(())
}

fn send_response<R: Serialize>(conn: &Connection, id: RequestId, result: R) -> anyhow::Result<()> {
    let value = serde_json::to_value(result).unwrap();
    let resp = Response {
        id,
        result: Some(value),
        error: None,
    };
    conn.sender.send(Message::Response(resp))?;
    Ok(())
}
