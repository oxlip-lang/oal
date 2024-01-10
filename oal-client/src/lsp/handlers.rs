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
    definition: &Definition,
) -> anyhow::Result<Vec<Location>> {
    let mut refs = Vec::new();
    for module in folder.modules().unwrap().modules() {
        for var in module.root().descendants().filter_map(Variable::cast) {
            if definition == var.node().syntax().core_ref().definition().unwrap() {
                let location = node_location(workspace, var.identifier().node())?;
                refs.push(location);
            }
        }
    }
    Ok(refs)
}

/// Finds the folders containing the given locator.
fn find_folders<'a>(
    folders: &'a HashMap<Url, Folder>,
    loc: &'a Locator,
) -> impl Iterator<Item = &'a Folder> + 'a {
    folders
        .iter()
        .filter_map(|(_, f)| if f.contains(loc) { Some(f) } else { None })
}

/// Implements the go-to-definition capability.
pub fn go_to_definition(
    state: &mut GlobalState,
    params: GotoDefinitionParams,
) -> anyhow::Result<Option<GotoDefinitionResponse>> {
    let pos = params.text_document_position_params.position;
    let loc = Locator::from(params.text_document_position_params.text_document.uri);
    let text = state.workspace.read_file(&loc)?;
    let index = position_to_utf8(&text, pos);

    for folder in find_folders(&state.folders, &loc) {
        let tree = folder.module(&loc).unwrap();
        if let Some(v) = syntax_at::<Variable<_>>(tree, index) {
            if let Some(Definition::External(ext)) = v.node().syntax().core_ref().definition() {
                let definition = ext.node(folder.modules().unwrap());
                let location = node_location(&mut state.workspace, definition)?;
                return Ok(Some(GotoDefinitionResponse::Scalar(location)));
            }
        }
    }

    Ok(Some(GotoDefinitionResponse::Array(Vec::new())))
}

/// Implements the references capability.
pub fn references(
    state: &mut GlobalState,
    params: ReferenceParams,
) -> anyhow::Result<Option<Vec<Location>>> {
    let pos = params.text_document_position.position;
    let loc = Locator::from(params.text_document_position.text_document.uri);
    let text = state.workspace.read_file(&loc)?;
    let index = position_to_utf8(&text, pos);

    let mut refs = Vec::new();

    for folder in find_folders(&state.folders, &loc) {
        let tree = folder.module(&loc).unwrap();
        if let Some(definition) = find_definition(tree, index) {
            let r = &mut find_references(&mut state.workspace, folder, &definition)?;
            refs.append(r);
        }
    }

    Ok(Some(refs))
}

/// Implements the preparation of the identifier rename capability.
pub fn prepare_rename(
    state: &mut GlobalState,
    params: TextDocumentPositionParams,
) -> anyhow::Result<Option<Range>> {
    let pos = params.position;
    let loc = Locator::from(params.text_document.uri);
    let text = state.workspace.read_file(&loc)?;
    let index = position_to_utf8(&text, pos);

    for folder in find_folders(&state.folders, &loc) {
        let tree = folder.module(&loc).unwrap();
        if let Some(ident) = syntax_at::<Identifier<_>>(tree, index) {
            let parent = ident.node().ancestors().nth(1).unwrap();
            let node = if let Some(decl) = Declaration::cast(parent) {
                Some(decl.identifier().node())
            } else if let Some(qualifier) = Qualifier::cast(parent) {
                qualifier.identifier().map(|i| i.node())
            } else {
                // Defaults to matching the unqualified identifier of a variable reference.
                Variable::cast(parent).map(|var| var.identifier().node())
            };
            if let Some(n) = node {
                let range = utf8_range_to_position(&text, n.span().unwrap().range());
                return Ok(Some(range));
            }
        }
    }

    Ok(None)
}

/// Implements the identifier rename capability.
pub fn rename(
    state: &mut GlobalState,
    params: RenameParams,
) -> anyhow::Result<Option<WorkspaceEdit>> {
    let pos = params.text_document_position.position;
    let loc = Locator::from(params.text_document_position.text_document.uri);
    let new_name = params.new_name;
    let text = state.workspace.read_file(&loc)?;
    let index = position_to_utf8(&text, pos);

    let mut changes = HashMap::new();

    for folder in find_folders(&state.folders, &loc) {
        let tree = folder.module(&loc).unwrap();
        if let Some(definition) = find_definition(tree, index) {
            rename_variable(
                &mut state.workspace,
                folder,
                &new_name,
                definition,
                &mut changes,
            )?;
        } else if let Some(qualifier) = find_qualifier(tree, index) {
            rename_qualifier(
                &mut state.workspace,
                folder,
                &new_name,
                qualifier,
                &mut changes,
            )?;
        }
    }

    let edits = WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    };

    Ok(Some(edits))
}

/// Renames an import qualifier and all references.
fn rename_qualifier<'a>(
    workspace: &mut Workspace,
    folder: &'a Folder,
    new_name: &str,
    qualifier: Qualifier<'a, Core>,
    changes: &mut HashMap<Url, Vec<TextEdit>>,
) -> anyhow::Result<()> {
    let Some(definition) = qualifier.identifier() else {
        // Not an import qualifier definition.
        return Ok(());
    };

    // Rename the qualifier definition
    let def_location = node_location(workspace, definition.node())?;
    let def_edit = TextEdit::new(def_location.range, new_name.into());
    changes.insert(def_location.uri, vec![def_edit]);

    // Rename all references to the qualifier
    let loc = definition.node().span().unwrap().locator().clone();
    let module = folder.module(&loc).unwrap();
    for var in module.root().descendants().filter_map(Variable::cast) {
        match (var.qualifier(), qualifier.identifier()) {
            (Some(reference), Some(definition)) if reference == definition => {
                let location = node_location(workspace, reference.node())?;
                let edit = TextEdit::new(location.range, new_name.into());
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

    Ok(())
}

/// Renames a variable definition and all references.
fn rename_variable(
    workspace: &mut Workspace,
    folder: &Folder,
    new_name: &str,
    definition: Definition,
    changes: &mut HashMap<Url, Vec<TextEdit>>,
) -> anyhow::Result<()> {
    let Definition::External(ref external) = definition else {
        // Not an external definition.
        return Ok(());
    };

    // Rename the variable declaration.
    let decl = Declaration::cast(external.node(folder.modules().unwrap())).unwrap();
    let decl_location = node_location(workspace, decl.identifier().node())?;
    let decl_edit = TextEdit::new(decl_location.range, new_name.into());
    changes.insert(decl_location.uri, vec![decl_edit]);

    // Rename all references to the variable.
    for r in find_references(workspace, folder, &definition)? {
        let edit = TextEdit::new(r.range, new_name.into());
        match changes.entry(r.uri) {
            Entry::Occupied(mut e) => {
                e.get_mut().push(edit);
            }
            Entry::Vacant(e) => {
                e.insert(vec![edit]);
            }
        }
    }

    Ok(())
}
