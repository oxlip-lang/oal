use super::state::GlobalState;
use super::{utf16_range, Folder, Workspace};
use crate::utf16::char_index;
use lsp_types::{
    GotoDefinitionParams, GotoDefinitionResponse, Location, Position, Range, ReferenceParams,
    RenameParams, TextDocumentPositionParams, TextEdit, WorkspaceEdit,
};
use oal_compiler::definition::{Definition, External};
use oal_compiler::tree::{Core, NRef, Tree};
use oal_model::grammar::AbstractSyntaxNode;
use oal_model::locator::Locator;
use oal_syntax::parser::{Declaration, Gram, Identifier, Variable};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

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

// Returns the location of the given syntax node.
fn node_location(workspace: &mut Workspace, node: NRef) -> anyhow::Result<Location> {
    let span = node.span().unwrap();
    let text = workspace.read_file(span.locator())?;
    let range = utf16_range(&text, span.range());
    Ok(Location::new(span.locator().url().clone(), range))
}

// Finds the definition of the identifier at the given location, if any.
fn find_definition(
    workspace: &mut Workspace,
    folder: &Folder,
    loc: &Locator,
    pos: Position,
) -> anyhow::Result<Option<Definition>> {
    let tree = folder.module(loc).unwrap();
    let text = workspace.read_file(loc)?;
    let index = char_index(&text, pos);
    if let Some(ident) = syntax_at::<Identifier<_>>(tree, index) {
        let parent = ident.node().ancestors().nth(1).unwrap();
        let definition = if let Some(decl) = Declaration::cast(parent) {
            Some(Definition::External(External::new(tree, decl.node())))
        } else if let Some(var) = Variable::cast(parent) {
            var.node().syntax().core_ref().definition().cloned()
        } else {
            None
        };
        return Ok(definition);
    }
    Ok(None)
}

// Finds all references to the given definition.
fn find_references(
    workspace: &mut Workspace,
    folder: &Folder,
    definition: Definition,
) -> anyhow::Result<Vec<Location>> {
    let mut refs: Vec<Location> = Vec::new();
    for module in folder.modules().unwrap().modules() {
        for var in module.root().descendants().filter_map(Variable::cast) {
            if definition == *var.node().syntax().core_ref().definition().unwrap() {
                let location = node_location(workspace, var.identifier().node())?;
                refs.push(location);
            }
        }
    }
    Ok(refs)
}

/// Implements the go-to-definition capability.
pub fn go_to_definition(
    state: &mut GlobalState,
    params: GotoDefinitionParams,
) -> anyhow::Result<Option<GotoDefinitionResponse>> {
    let pos = params.text_document_position_params.position;
    let loc = Locator::from(params.text_document_position_params.text_document.uri);

    let Some(folder) = state.folders.iter().find(|f| f.contains(&loc)) else {
        // Location not found in any folder.
        return Ok(None)
    };

    let tree = folder.module(&loc).unwrap();
    let text = state.workspace.read_file(&loc)?;
    let index = char_index(&text, pos);

    let mut res = GotoDefinitionResponse::Array(Vec::new());

    if let Some(v) = syntax_at::<Variable<_>>(tree, index) {
        if let Some(Definition::External(ext)) = v.node().syntax().core_ref().definition() {
            let definition = ext.node(folder.modules().unwrap());
            let location = node_location(&mut state.workspace, definition)?;
            res = GotoDefinitionResponse::Scalar(location);
        }
    }

    Ok(Some(res))
}

/// Implements the references capability.
pub fn references(
    state: &mut GlobalState,
    params: ReferenceParams,
) -> anyhow::Result<Option<Vec<Location>>> {
    let pos = params.text_document_position.position;
    let loc = Locator::from(params.text_document_position.text_document.uri);

    let Some(folder) = state.folders.iter().find(|f| f.contains(&loc)) else {
        // Location not found in any folder.
        return Ok(None)
    };

    let Some(definition) = find_definition(&mut state.workspace, folder, &loc, pos)? else {
        // Not a variable or definition.
        return Ok(None)
    };

    let refs = find_references(&mut state.workspace, folder, definition)?;
    Ok(Some(refs))
}

// Implements the preparation of the identifier rename capability.
pub fn prepare_rename(
    state: &mut GlobalState,
    params: TextDocumentPositionParams,
) -> anyhow::Result<Option<Range>> {
    let pos = params.position;
    let loc = Locator::from(params.text_document.uri);

    let Some(folder) = state.folders.iter().find(|f| f.contains(&loc)) else {
        // Location not found in any folder.
        return Ok(None)
    };

    let tree = folder.module(&loc).unwrap();
    let text = state.workspace.read_file(&loc)?;
    let index = char_index(&text, pos);

    if let Some(ident) = syntax_at::<Identifier<_>>(tree, index) {
        // Get the unqualified identifier from either a declaration or a variable.
        // TODO: add support for module alias rename
        let parent = ident.node().ancestors().nth(1).unwrap();
        let identifier = if let Some(decl) = Declaration::cast(parent) {
            Some(decl.identifier().node())
        } else {
            Variable::cast(parent).map(|var| var.identifier().node())
        };
        Ok(identifier.map(|n| utf16_range(&text, n.span().unwrap().range())))
    } else {
        Ok(None)
    }
}

// Implements the identifier rename capability.
pub fn rename(
    state: &mut GlobalState,
    params: RenameParams,
) -> anyhow::Result<Option<WorkspaceEdit>> {
    let pos = params.text_document_position.position;
    let loc = Locator::from(params.text_document_position.text_document.uri);
    let new_name = params.new_name;

    let Some(folder) = state.folders.iter().find(|f| f.contains(&loc)) else {
        // Location not found in any folder.
        return Ok(None)
    };

    let Some(definition) = find_definition(&mut state.workspace, folder, &loc, pos)? else {
        // Not a variable or definition.
        return Ok(None)
    };

    let Definition::External(ref external) = definition else {
        // Not an external definition.
        return Ok(None)
    };

    let mut changes = HashMap::new();

    let decl = Declaration::cast(external.node(folder.modules().unwrap())).unwrap();
    let decl_loc = node_location(&mut state.workspace, decl.identifier().node())?;
    let decl_edit = TextEdit::new(decl_loc.range, new_name.clone());

    changes.insert(decl_loc.uri, vec![decl_edit]);

    for r in find_references(&mut state.workspace, folder, definition)? {
        let edit = TextEdit::new(r.range, new_name.clone());
        match changes.entry(r.uri) {
            Entry::Occupied(mut e) => {
                e.get_mut().push(edit);
            }
            Entry::Vacant(e) => {
                e.insert(vec![edit]);
            }
        }
    }

    let edits = WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    };

    Ok(Some(edits))
}
