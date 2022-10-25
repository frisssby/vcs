use crate::errors::VcsResult;
use crate::vcs_manager::*;

use super::{file_manager, objects::*, traits::VcsSerialize};

use std::fs;
use std::path::{Path, PathBuf};

pub const VCS_ROOT: &str = ".vcs";
const VCS_INDEX: &str = "index.json";
const VCS_HEADS: &str = "refs/heads.json";
const VCS_STATE: &str = "STATE.json";
const VCS_OBJECTS: &str = "objects.json";
pub const MASTER_BRANCH: &str = "master";

pub fn get_vcs_root(repo: &Path) -> PathBuf {
    repo.join(VCS_ROOT)
}

fn get_vcs_entry(repo: &Path, relative_path: impl AsRef<Path>) -> PathBuf {
    get_vcs_root(repo).join(relative_path)
}

pub fn get_vcs_state_path(repo: &Path) -> PathBuf {
    get_vcs_entry(repo, VCS_STATE)
}

pub fn get_vcs_heads_path(repo: &Path) -> PathBuf {
    get_vcs_entry(repo, VCS_HEADS)
}

pub fn get_vcs_index_path(repo: &Path) -> PathBuf {
    get_vcs_entry(repo, VCS_INDEX)
}

pub fn get_vsc_object_path(repo: &Path, id: &VcsObjectId) -> PathBuf {
    let human_id = get_human_id(id);
    let relative_path = PathBuf::from(&human_id[..2]).join(&human_id[2..]);
    get_vcs_entry(repo, VCS_OBJECTS).join(relative_path)
}

pub fn init_state(repo: &Path) -> VcsResult<()> {
    let state = VcsRepositoryState {
        current_commit: None,
        current_branch: MASTER_BRANCH.to_owned(),
    };
    state.save(&get_vcs_state_path(repo))?;
    Ok(())
}

pub fn init_index(repo: &Path) -> VcsResult<()> {
    Index::new().save(&get_vcs_index_path(repo))?;
    Ok(())
}

pub fn init_heads(repo: &Path) -> VcsResult<()> {
    RefStorage::new().save(&get_vcs_heads_path(repo))?;
    Ok(())
}

/// Creates a tree and writes it to the objects database using data contained in
/// the index.
pub fn build_tree(repo: &Path, directory: &Path, index: &Index) -> VcsResult<VcsObjectId> {
    let mut tree = Tree::new();
    for entry in file_manager::get_entries(directory, false).into_iter() {
        let (is_blob, id) = if entry.is_dir() {
            if file_manager::is_empty_dir(&entry)? {
                continue;
            }
            (false, build_tree(repo, &entry, index)?)
        } else {
            let id = index.get_id(&entry);
            (true, *id)
        };
        tree.add_node(TreeNode::new(id, entry, is_blob));
    }
    let content = VcsObjects::Tree(tree).get_content()?;
    let tree_id = get_vcs_object_id(&content);
    file_manager::write_file(&get_vsc_object_path(repo, &tree_id), &content)?;
    Ok(tree_id)
}

/// Creates a blob mathching the given file and writes in to the objects
/// database.
pub fn add_blob(repo: &Path, file_path: &Path, index: &mut Index) -> VcsResult<VcsObjectId> {
    let blob = VcsObjects::Blob(make_blob(repo, file_path)?);
    let content = blob.get_content()?;
    let id = get_vcs_object_id(&content);
    index.update(file_path.to_path_buf(), id);

    let blob_path = get_vsc_object_path(repo, &id);
    file_manager::write_file(&blob_path, &content)?;
    Ok(id)
}

/// Forms a Blob object from file name and its contents.
fn make_blob(repo: &Path, file_path: &Path) -> VcsResult<Blob> {
    let data = file_manager::read_file(file_path)?;
    let file_name = file_manager::get_relative(repo, file_path)
        .into_os_string()
        .into_string()
        .unwrap();
    Ok(Blob { file_name, data })
}

/// Writes the Commit object to the objects database.
pub fn record_commit(repo: &Path, commit: &Commit) -> VcsResult<VcsObjectId> {
    let commit_content = VcsObjects::Commit(commit.clone()).get_content()?;
    let commit_id = get_vcs_object_id(&commit_content);
    file_manager::write_file(&get_vsc_object_path(repo, &commit_id), &commit_content)?;
    Ok(commit_id)
}

/// Compares two trees by recursively traversing them and returns all the file
/// changes in the second tree in relation to the first tree.
pub fn compare_trees(repo: &Path, first: &Tree, second: &Tree) -> VcsResult<FileChanges> {
    let mut changes = Vec::new();
    for second_node in second.iter() {
        let first_node = first.find(&second_node.path, second_node.is_blob());
        if let Some(first_node) = first_node {
            if first_node.is_tree() {
                let first_subtree =
                    VcsObjects::load(&get_vsc_object_path(repo, &first_node.id))?.tree();
                let second_subtree =
                    VcsObjects::load(&get_vsc_object_path(repo, &second_node.id))?.tree();
                changes.append(&mut compare_trees(repo, &first_subtree, &second_subtree)?);
            } else if first_node.id != second_node.id {
                changes.push((FileStatus::Modified, first_node.path.to_owned()))
            }
        } else if second_node.is_blob() {
            changes.push((FileStatus::Added, second_node.path.to_owned()));
        } else {
            let subtree = VcsObjects::load(&get_vsc_object_path(repo, &second_node.id))?.tree();
            changes.append(&mut get_tree_files(repo, &subtree)?);
        }
    }
    Ok(changes)
}

pub fn get_tree_files(repo: &Path, tree: &Tree) -> VcsResult<FileChanges> {
    let mut files = Vec::new();
    for node in tree.iter() {
        if node.is_blob() {
            files.push((FileStatus::Added, node.path.to_owned()))
        } else {
            let subtree = VcsObjects::load(&get_vsc_object_path(repo, &node.id))?.tree();
            files.append(&mut get_tree_files(repo, &subtree)?);
        }
    }
    Ok(files)
}

/// Returns the paths of the files whose contents do not match blobs recorded in
/// the index and the type of change that has happened to them.
pub fn get_changed_files(repo: &Path, index: &Index) -> VcsResult<FileChanges> {
    let files = file_manager::get_all_files(repo, true);
    files
        .into_iter()
        .map(|p| Ok((get_file_status(repo, &p, index)?, p)))
        .filter_map(|f| match f {
            Ok((st, _)) if st == FileStatus::Unchanged => None,
            f => Some(f),
        })
        .collect()
}

/// Updates working tree so that it matches the tree provided
pub fn load_from_tree(repo: &Path, tree: &Tree, index: &mut Index) -> VcsResult<()> {
    index.clear();
    for child in tree.iter() {
        if child.is_blob() {
            let blob = VcsObjects::load(&get_vsc_object_path(repo, &child.id))?.blob();
            file_manager::write_file(&child.path, &blob.data)?;
            index.update(child.path.to_path_buf(), child.id);
        } else {
            let subtree = VcsObjects::load(&get_vsc_object_path(repo, &child.id))?.tree();
            load_from_tree(repo, &subtree, index)?;
        }
    }
    index.save(&get_vcs_index_path(repo))?;
    Ok(())
}

/// Removes entries that are not present in the index from the working tree.
pub fn remove_extra_entries(directory: &Path, index: &Index) -> VcsResult<()> {
    for entry in file_manager::get_entries(directory, false) {
        if entry.is_dir() {
            remove_extra_entries(&entry, index)?;
            if file_manager::is_empty_dir(&entry)? {
                fs::remove_dir(&entry)?;
            }
        } else if !index.contains(&entry) {
            fs::remove_file(&entry)?;
        }
    }
    Ok(())
}

/// Get the file status in the working tree in relation to the current index.
fn get_file_status(repo: &Path, file_path: &Path, index: &Index) -> VcsResult<FileStatus> {
    assert!(!file_path.is_dir());
    if !index.contains(file_path) {
        Ok(FileStatus::Added)
    } else {
        let index_id = index.get_id(file_path);
        let blob = VcsObjects::Blob(make_blob(repo, file_path)?);
        let id = get_vcs_object_id(&blob.get_content()?);
        if id.eq(index_id) {
            Ok(FileStatus::Unchanged)
        } else {
            Ok(FileStatus::Modified)
        }
    }
}

/// Merges the source tree into the destination one and returns the resulting
/// tree.
pub fn merge_trees(source: Tree, mut destination: Tree) -> Tree {
    for src_node in source.into_iter() {
        let dst_node = destination.find_mut(&src_node.path, src_node.is_blob());
        if let Some(dst_node) = dst_node {
            dst_node.id = src_node.id;
        } else {
            destination.add_node(src_node);
        }
    }
    destination
}

#[test]
fn test_merge_trees() {
    let (file1, file2, dir1, dir2) = (
        PathBuf::from("path/to/file1"),
        PathBuf::from("path/to/file2"),
        PathBuf::from("path/to/dir1"),
        PathBuf::from("path/to/dir2"),
    );
    let mut source = Tree::new();
    source.add_node(TreeNode::new([1; 20], file1.to_owned(), true));
    source.add_node(TreeNode::new([2; 20], file2.to_owned(), true));
    source.add_node(TreeNode::new([3; 20], dir1.to_owned(), false));

    let mut destination = Tree::new();
    destination.add_node(TreeNode::new([4; 20], file1.to_owned(), true));
    destination.add_node(TreeNode::new([5; 20], dir2.to_owned(), false));

    let mut merged = Tree::new();
    merged.add_node(TreeNode::new([1; 20], file1, true));
    merged.add_node(TreeNode::new([5; 20], dir2, false));
    merged.add_node(TreeNode::new([2; 20], file2, true));
    merged.add_node(TreeNode::new([3; 20], dir1, false));

    assert_eq!(merge_trees(source, destination), merged);
}

pub fn get_branch_root(repo: &Path, branch_name: &str, mut head: Commit) -> VcsResult<Commit> {
    while head.branch == branch_name {
        let parent = head.parent;
        head = if let Some(parent) = &parent {
            VcsObjects::load(&get_vsc_object_path(repo, parent))?.commit()
        } else {
            return Ok(head);
        };
    }
    Ok(head)
}
