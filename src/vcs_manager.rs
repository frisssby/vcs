mod file_manager;
mod objects;
mod objects_manager;
pub mod public_info;
mod traits;

use self::objects::*;
use self::objects_manager::*;
pub use self::public_info::*;
use self::traits::VcsSerialize;
use crate::errors::{VcsError, VcsResult};

use array_tool::vec::Intersect;
use chrono::DateTime;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Creates **.vcs** directory together with all its subdirectories and files to
/// match the following structure:
///
/// .vcs
/// ├── index
/// ├── STATE
/// ├── objects
/// │     └── ...
/// └── refs
///     └── heads
pub fn init_vcs_directory(path: &Path) -> VcsResult<()> {
    assert!(path.is_absolute());
    let vcs_directory = get_vcs_root(path);
    if vcs_directory.is_dir() {
        Err(VcsError::AlreadyVcsRepository)?;
    }
    fs::create_dir(&vcs_directory)?;
    init_state(path)?;
    init_index(path)?;
    init_heads(path)?;
    Ok(())
}

/// Finds a repository root by searching for .vcs folder in the provided path
/// directory and all of its parents
pub fn find_repository(current_dir: &Path) -> VcsResult<PathBuf> {
    for path in current_dir.ancestors() {
        if path.join(VCS_ROOT).is_dir() {
            return Ok(path.to_owned());
        }
    }
    Ok(Err(VcsError::NotVcsRepository)?)
}

/// Get all the files that have been changed since the current commit. If
/// succeeds, returns `FileChanges` object, which is basically a
/// `Vec<(FileStatus, PathBuf)>`. The paths are returned
/// as relative to the repository root.
pub fn get_changes(repo: &Path) -> VcsResult<FileChanges> {
    let index = Index::load(&get_vcs_index_path(repo))?;
    Ok(into_pathspec(repo, get_changed_files(repo, &index)?))
}

// Get information about the repository STATE
pub fn get_state(repo: &Path) -> VcsResult<StateInfo> {
    let state = VcsRepositoryState::load(&get_vcs_state_path(repo))?;
    Ok(StateInfo::from(state))
}

/// Forms a new commit from all the current changes in the repository. Updates
/// STATE so that it points to the newly created commit.
pub fn make_commit(repo: &Path, message: &str) -> VcsResult<NewCommitInfo> {
    let index_path = get_vcs_index_path(repo);
    let mut index = Index::load(&index_path)?;
    let changed_files = get_changed_files(repo, &index)?;

    let state_path = get_vcs_state_path(repo);
    let mut state = VcsRepositoryState::load(&state_path)?;

    let heads_path = get_vcs_heads_path(repo);
    let heads = RefStorage::load(&heads_path)?;

    if let Some(current_commit) = &state.current_commit {
        let branch_head = heads.get_id(&state.current_branch);
        if branch_head != current_commit {
            Err(VcsError::CommitFromNonHead)?;
        }
        if changed_files.is_empty() {
            Err(VcsError::NoChanges)?;
        }
    };

    for (_status, path) in changed_files.iter() {
        add_blob(repo, path, &mut index)?;
    }
    let snapshot = build_tree(repo, repo, &index)?;
    index.save(&index_path)?;

    let commit = Commit {
        tree: snapshot,
        parent: state.current_commit,
        branch: state.current_branch.clone(),
        message: message.to_string(),
        time: SystemTime::now(),
    };
    let commit_id = record_commit(repo, &commit)?;
    state.current_commit = Some(commit_id);

    let mut heads = heads;
    heads.update(state.current_branch.clone(), commit_id);

    state.save(&state_path)?;
    heads.save(&heads_path)?;

    Ok(NewCommitInfo {
        human_id: get_human_id(&commit_id),
        branch: commit.branch,
        changes: into_pathspec(repo, changed_files),
        message: commit.message,
    })
}

/// Creates new branch with a name `branch_name` unless it already exists.
/// Checks that STATE is at the MASTER's head.
pub fn create_branch(repo: &Path, branch_name: &str) -> VcsResult<StateInfo> {
    let state_path = get_vcs_state_path(repo);
    let mut state = VcsRepositoryState::load(&state_path)?;
    if state.current_branch != MASTER_BRANCH {
        Err(VcsError::BranchOffNonMaster)?;
    }
    let heads = RefStorage::load(&get_vcs_heads_path(repo))?;
    if heads.contains(branch_name) {
        Err(VcsError::BranchAlreadyExists(branch_name.to_owned()))?;
    }
    if let Some(current_commit) = state.current_commit {
        let heads_path = get_vcs_heads_path(repo);
        let mut heads = RefStorage::load(&heads_path)?;
        heads.update(branch_name.to_owned(), current_commit);
        heads.save(&heads_path)?;
    }
    state.current_branch = branch_name.to_owned();
    state.save(&state_path)?;
    Ok(StateInfo::from(VcsRepositoryState {
        current_commit: state.current_commit,
        current_branch: state.current_branch,
    }))
}

/// Updates STATE to be on the specified commit. Updates working tree by loading
/// the tree the commit points to. Checks for uncommitted changes.
pub fn jump_to_commit(repo: &Path, commit_id: &str) -> VcsResult<StateInfo> {
    let mut index = Index::load(&get_vcs_index_path(repo))?;
    let changes = get_changed_files(repo, &index)?;
    if !changes.is_empty() {
        Err(VcsError::UncomittedChanges {
            changes: into_pathspec(repo, changes),
        })?;
    }

    let commit_id = get_inner_id(commit_id)?;
    let commit = VcsObjects::load(&get_vsc_object_path(repo, &commit_id))?.commit();
    let tree = VcsObjects::load(&get_vsc_object_path(repo, &commit.tree))?.tree();
    load_from_tree(repo, &tree, &mut index)?;
    remove_extra_entries(repo, &index)?;
    let state = VcsRepositoryState {
        current_branch: commit.branch,
        current_commit: Some(commit_id),
    };

    state.save(&get_vcs_state_path(repo))?;
    Ok(StateInfo::from(state))
}

/// Updates STATE to be on the specified branch and its head commit.
/// Updates working tree by loading the tree the branch's head commit points to.
/// Checks for uncommitted changes.
pub fn jump_to_branch(repo: &Path, branch_name: &str) -> VcsResult<StateInfo> {
    let heads = RefStorage::load(&get_vcs_heads_path(repo))?;
    if !heads.contains(branch_name) {
        Err(VcsError::NoBranch(branch_name.to_owned()))?;
    }
    let commit_id = heads.get_id(branch_name);
    jump_to_commit(repo, &get_human_id(commit_id))?;
    let state = VcsRepositoryState {
        current_branch: branch_name.to_owned(),
        current_commit: Some(*commit_id),
    };
    state.save(&get_vcs_state_path(repo))?;
    Ok(StateInfo::from(state))
}

/// Forms commit logs from the current commit to the root commit
pub fn get_commit_logs(repo: &Path) -> VcsResult<Vec<CommitLog>> {
    let state = VcsRepositoryState::load(&get_vcs_state_path(repo))?;
    let mut logs = Vec::new();
    let mut current_commit_id = state.current_commit;
    let (mut current_commit, mut current_tree) = (None, None);
    while let Some(commit_id) = current_commit_id {
        let commit = current_commit
            .unwrap_or(VcsObjects::load(&get_vsc_object_path(repo, &commit_id))?.commit());
        let tree = current_tree
            .unwrap_or(VcsObjects::load(&get_vsc_object_path(repo, &commit.tree))?.tree());
        (current_commit, current_tree) = (None, None);

        let changes = if let Some(parent) = &commit.parent {
            let parent_commit = VcsObjects::load(&get_vsc_object_path(repo, parent))?.commit();
            let parent_tree =
                VcsObjects::load(&get_vsc_object_path(repo, &parent_commit.tree))?.tree();
            let changes = compare_trees(repo, &parent_tree, &tree)?;
            current_commit = Some(parent_commit);
            current_tree = Some(parent_tree);
            changes
        } else {
            get_tree_files(repo, &tree)?
        };
        logs.push(CommitLog {
            human_id: get_human_id(&commit_id),
            changes: into_pathspec(repo, changes),
            message: commit.message,
            time: DateTime::from(commit.time),
        });
        current_commit_id = commit.parent;
    }
    Ok(logs)
}

/// Merge `branch_name` into MASTER branch. Checks for merge conflics and aborts
/// iff there are any.
pub fn merge_branch(repo: &Path, branch_name: &str) -> VcsResult<NewCommitInfo> {
    let state_path = get_vcs_state_path(repo);
    let state = VcsRepositoryState::load(&state_path)?;

    let heads_path = get_vcs_heads_path(repo);
    let heads = RefStorage::load(&heads_path)?;

    if !heads.contains(branch_name) {
        Err(VcsError::NoBranch(branch_name.to_owned()))?;
    }
    let master_head_id = heads.get_id(MASTER_BRANCH);
    if state.current_branch != MASTER_BRANCH
        || heads.contains(MASTER_BRANCH) && state.current_commit.unwrap().ne(master_head_id)
    {
        Err(VcsError::MergeFromNotMasterHead)?;
    }
    let index_path = get_vcs_index_path(repo);
    let index = Index::load(&index_path)?;
    let changes = get_changed_files(repo, &index)?;
    if !changes.is_empty() {
        Err(VcsError::UncomittedChanges {
            changes: into_pathspec(repo, changes),
        })?;
    }
    let branch_head_id = heads.get_id(branch_name);
    let branch_head = VcsObjects::load(&get_vsc_object_path(repo, branch_head_id))?.commit();
    let branch_tree = VcsObjects::load(&get_vsc_object_path(repo, &branch_head.tree))?.tree();

    let root_commit = get_branch_root(repo, branch_name, branch_head)?;
    let root_tree = VcsObjects::load(&get_vsc_object_path(repo, &root_commit.tree))?.tree();

    let master_head = VcsObjects::load(&get_vsc_object_path(repo, master_head_id))?.commit();
    let master_tree = VcsObjects::load(&get_vsc_object_path(repo, &master_head.tree))?.tree();

    let branch_changes = compare_trees(repo, &root_tree, &branch_tree)?;
    let master_changes = compare_trees(repo, &root_tree, &master_tree)?;
    let intersection = branch_changes.intersect(master_changes);
    if !intersection.is_empty() {
        Err(VcsError::MergeConflict {
            both_changed: intersection,
        })?;
    }
    let merged_tree = VcsObjects::Tree(merge_trees(branch_tree, master_tree));
    let content = merged_tree.get_content()?;
    let merged_tree_id = get_vcs_object_id(&content);
    file_manager::write_file(&get_vsc_object_path(repo, &merged_tree_id), &content)?;

    let merge_commit = Commit {
        tree: merged_tree_id,
        parent: Some(*master_head_id),
        branch: MASTER_BRANCH.to_owned(),
        message: format!("Merged branch {}", branch_name),
        time: SystemTime::now(),
    };
    let commit_id = record_commit(repo, &merge_commit)?;

    let mut heads = heads;
    heads.update(MASTER_BRANCH.to_owned(), commit_id);
    heads.save(&heads_path)?;

    let mut state = state;
    state.current_commit = Some(commit_id);
    state.save(&state_path)?;

    let mut index = index;
    load_from_tree(repo, &merged_tree.tree(), &mut index)?;
    index.save(&index_path)?;

    Ok(NewCommitInfo {
        human_id: get_human_id(&commit_id),
        branch: merge_commit.branch,
        changes: into_pathspec(repo, branch_changes),
        message: merge_commit.message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::{FileTouch, PathAssert, PathChild, PathCreateDir};

    #[test]
    fn test_init() {
        let tmp_dir = assert_fs::TempDir::new().unwrap();
        assert!(init_vcs_directory(&tmp_dir).is_ok());
    }

    #[test]
    fn test_double_init() {
        let tmp_dir = assert_fs::TempDir::new().unwrap();
        assert!(init_vcs_directory(&tmp_dir).is_ok());
        assert!(init_vcs_directory(&tmp_dir).is_err());
    }

    #[test]
    fn test_find_repo() {
        let tmp_dir = assert_fs::TempDir::new().unwrap();
        tmp_dir.child("subdir/subsubdir").create_dir_all().unwrap();
        let repo = find_repository(&tmp_dir.join("subdir/subsubdir"));
        assert!(repo.is_err());
        assert_eq!(repo.unwrap_err().to_string(), "Not a vcs repository");

        tmp_dir.child(".vcs").create_dir_all().unwrap();
        let repo = find_repository(&tmp_dir.join("subdir/subsubdir"));
        assert!(repo.is_ok());
        assert_eq!(repo.unwrap(), tmp_dir.path())
    }

    #[test]
    fn test_changes() {
        let tmp_dir = assert_fs::TempDir::new().unwrap();
        let index_path = tmp_dir.child(get_vcs_index_path(tmp_dir.path()));
        index_path.touch().unwrap();
        index_path.assert("");

        let index = Index::new();
        index.save(index_path.path()).unwrap();

        tmp_dir.child("file1").touch().unwrap();
        tmp_dir.child("subdir/file2").touch().unwrap();

        let changes = get_changes(tmp_dir.path()).unwrap();
        assert_eq!(
            changes,
            [
                (FileStatus::Added, PathBuf::from("file1")),
                (FileStatus::Added, PathBuf::from("subdir/file2"))
            ]
        )
    }
}
