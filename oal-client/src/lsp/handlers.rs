use super::state::GlobalState;
use super::unicode::position_to_utf8;
use super::{utf8_range_to_position, Folder, Workspace};
use lsp_types::{
    GotoDefinitionParams, GotoDefinitionResponse, Location, Range, ReferenceParams, RenameParams,
    TextDocumentPositionParams, TextEdit, WorkspaceEdit,
};
use oal_compiler::definition::{Definition, External};
use oal_compiler::tree::{Core, NRef, Tree};
use oal_model::grammar::AbstractSyntaxNode;
use oal_model::locator::Locator;
use oal_syntax::parser::{Declaration, Gram, Identifier, Qualifier, Variable};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use url::Url;

/// Returns the abstract syntax node at the given UTF-8 index, if any.
fn syntax_at<'a, N>(tree: &'a Tree, index: usize) -> Option<N>
where
    N: AbstractSyntaxNode<'a, Core, Gram>,
{
    tree.root()
        .descendants()
        .filter_map(N::cast)
        .find(|i| i.node().span().unwrap().range().contains(&index))
}

/// Returns the location of the given syntax node.
fn node_location(workspace: &mut Workspace, node: NRef) -> anyhow::Result<Location> {
    let span = node.span().unwrap();
    let text = workspace.read_file(span.locator())?;
    let range = utf8_range_to_position(&text, span.range());
    Ok(Location::new(span.locator().url().clone(), range))
}

/// Finds the definition of the identifier at the given location, if any.
fn find_definition(tree: &Tree, index: usize) -> Option<Definition> {
    let Some(ident) = syntax_at::<Identifier<_>>(tree, index) else {
        return None;
    };
    let parent = ident.node().ancestors().nth(1).unwrap();
    if let Some(decl) = Declaration::cast(parent) {
        Some(Definition::External(External::new(decl.node())))
    } else if let Some(var) = Variable::cast(parent) {
        var.node().syntax().core_ref().definition().cloned()
    } else {
        None
    }
}

/// Finds the qualifier at the given identifier location, if any.
fn find_qualifier(tree: &Tree, index: usize) -> Option<Qualifier<Core>> {
    let Some(ident) = syntax_at::<Identifier<_>>(tree, index) else {
        return None;
    };
    let parent = ident.node().ancestors().nth(1).unwrap();
    Qualifier::cast(parent)
}

/// Finds all references to the given definition.
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

/// Finds the folder containing the given locator, if any.
fn find_folder<'a>(folders: &'a HashMap<Url, Folder>, loc: &Locator) -> Option<&'a Folder> {
    match folders.iter().find(|(_, f)| f.contains(loc)) {
        Some((_, folder)) => Some(folder),
        _ => None,
    }
}

/// Implements the go-to-definition capability.
pub fn go_to_definition(
    state: &mut GlobalState,
    params: GotoDefinitionParams,
) -> anyhow::Result<Option<GotoDefinitionResponse>> {
    let pos = params.text_document_position_params.position;
    let loc = Locator::from(params.text_document_position_params.text_document.uri);

    let Some(folder) = find_folder(&state.folders, &loc) else {
        // Location not found in any folder.
        return Ok(None);
    };

    let tree = folder.module(&loc).unwrap();
    let text = state.workspace.read_file(&loc)?;
    let index = position_to_utf8(&text, pos);

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

    let Some(folder) = find_folder(&state.folders, &loc) else {
        // Location not found in any folder.
        return Ok(None);
    };

    let tree = folder.module(&loc).unwrap();
    let text = state.workspace.read_file(&loc)?;
    let index = position_to_utf8(&text, pos);

    let Some(definition) = find_definition(tree, index) else {
        // Not a variable or definition.
        return Ok(None);
    };

    let refs = find_references(&mut state.workspace, folder, definition)?;
    Ok(Some(refs))
}

/// Implements the preparation of the identifier rename capability.
pub fn prepare_rename(
    state: &mut GlobalState,
    params: TextDocumentPositionParams,
) -> anyhow::Result<Option<Range>> {
    let pos = params.position;
    let loc = Locator::from(params.text_document.uri);

    let Some(folder) = find_folder(&state.folders, &loc) else {
        // Location not found in any folder.
        return Ok(None);
    };

    let tree = folder.module(&loc).unwrap();
    let text = state.workspace.read_file(&loc)?;
    let index = position_to_utf8(&text, pos);

    if let Some(ident) = syntax_at::<Identifier<_>>(tree, index) {
        let parent = ident.node().ancestors().nth(1).unwrap();
        let identifier = if let Some(decl) = Declaration::cast(parent) {
            Some(decl.identifier().node())
        } else if let Some(qualifier) = Qualifier::cast(parent) {
            qualifier.identifier().map(|i| i.node())
        } else {
            // Defaults to matching the unqualified identifier of a variable reference.
            Variable::cast(parent).map(|var| var.identifier().node())
        };
        Ok(identifier.map(|n| utf8_range_to_position(&text, n.span().unwrap().range())))
    } else {
        Ok(None)
    }
}

/// Implements the identifier rename capability.
pub fn rename(
    state: &mut GlobalState,
    params: RenameParams,
) -> anyhow::Result<Option<WorkspaceEdit>> {
    let pos = params.text_document_position.position;
    let loc = Locator::from(params.text_document_position.text_document.uri);
    let new_name = params.new_name;

    let Some(folder) = find_folder(&state.folders, &loc) else {
        // Location not found in any folder.
        return Ok(None);
    };

    let tree = folder.module(&loc).unwrap();
    let text = state.workspace.read_file(&loc)?;
    let index = position_to_utf8(&text, pos);

    if let Some(definition) = find_definition(tree, index) {
        rename_variable(&mut state.workspace, folder, new_name, definition)
    } else if let Some(qualifier) = find_qualifier(tree, index) {
        rename_qualifier(&mut state.workspace, folder, new_name, qualifier)
    } else {
        // Not a variable reference, definition or qualifier.
        return Ok(None);
    }
}

/// Renames an import qualifier and all references.
fn rename_qualifier<'a>(
    workspace: &mut Workspace,
    folder: &'a Folder,
    new_name: String,
    qualifier: Qualifier<'a, Core>,
) -> anyhow::Result<Option<WorkspaceEdit>> {
    let Some(definition) = qualifier.identifier() else {
        return Ok(None);
    };

    let mut changes = HashMap::new();

    // Rename the qualifier definition
    let def_location = node_location(workspace, definition.node())?;
    let def_edit = TextEdit::new(def_location.range, new_name.clone());
    changes.insert(def_location.uri, vec![def_edit]);

    // Rename all references to the qualifier
    let loc = definition.node().span().unwrap().locator().clone();
    let module = folder.module(&loc).unwrap();
    for var in module.root().descendants().filter_map(Variable::cast) {
        match (var.qualifier(), qualifier.identifier()) {
            (Some(reference), Some(definition)) if reference == definition => {
                let location = node_location(workspace, reference.node())?;
                let edit = TextEdit::new(location.range, new_name.clone());
                match changes.entry(location.uri) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().push(edit);
                    }
                    Entry::Vacant(e) => {
                        e.insert(vec![edit]);
                    }
                }
            }
            _ => {}
        }
    }

    let edits = WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    };

    Ok(Some(edits))
}

/// Renames a variable definition and all references.
fn rename_variable(
    workspace: &mut Workspace,
    folder: &Folder,
    new_name: String,
    definition: Definition,
) -> anyhow::Result<Option<WorkspaceEdit>> {
    let Definition::External(ref external) = definition else {
        // Not an external definition.
        return Ok(None);
    };

    let mut changes = HashMap::new();

    let decl = Declaration::cast(external.node(folder.modules().unwrap())).unwrap();
    let decl_loc = node_location(workspace, decl.identifier().node())?;
    let decl_edit = TextEdit::new(decl_loc.range, new_name.clone());

    changes.insert(decl_loc.uri, vec![decl_edit]);

    for r in find_references(workspace, folder, definition)? {
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
