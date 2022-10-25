use super::file_manager;
use super::objects::{VcsObjectId, VcsRepositoryState};
use crate::errors::{VcsError, VcsResult};
use chrono::{offset::Local, DateTime};
use std::path::{Path, PathBuf};

pub struct NewCommitInfo {
    pub human_id: String,
    pub branch: String,
    pub changes: FileChanges,
    pub message: String,
}

pub struct CommitLog {
    pub human_id: String,
    pub changes: FileChanges,
    pub message: String,
    pub time: DateTime<Local>,
}

pub struct StateInfo {
    pub commit: Option<String>,
    pub branch: String,
}

impl From<VcsRepositoryState> for StateInfo {
    fn from(state: VcsRepositoryState) -> Self {
        Self {
            commit: state.current_commit.map(|id| get_human_id(&id)),
            branch: state.current_branch,
        }
    }
}

impl From<&VcsRepositoryState> for StateInfo {
    fn from(state: &VcsRepositoryState) -> Self {
        Self {
            commit: state.current_commit.map(|id| get_human_id(&id)),
            branch: state.current_branch.clone(),
        }
    }
}

/// Transforms the inner representation of an object id into the public one.
pub fn get_human_id(id: &VcsObjectId) -> String {
    hex::encode(id)
}

/// Transforms the public representation of an object id into the inner one.
pub fn get_inner_id(id: &str) -> VcsResult<VcsObjectId> {
    let mut bytes = VcsObjectId::default();
    hex::decode_to_slice(id, &mut bytes).map_err(|_| VcsError::NoCommit(id.to_owned()))?;
    Ok(bytes)
}

#[test]
fn get_inner_id_works() {
    let human_id = "4c6f72656d20697073756d20677261766964612e";
    let inner_id = get_inner_id(human_id);
    assert!(inner_id.is_ok());
    assert_eq!(&inner_id.unwrap(), b"Lorem ipsum gravida.");
}

#[test]
fn incorrect_id() {
    let human_id = "41656e65616e2e0a0a";
    let inner_id = get_inner_id(human_id);
    assert!(inner_id.is_err());
}

/// Represents file statuses in the working tree in relation to the current
/// index state.  
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    Modified,
    Added,
    Unchanged,
}

pub type FileChanges = Vec<(FileStatus, PathBuf)>;

/// Transforms file paths so that they are relative to the repository root.
pub fn into_pathspec(repo: &Path, changes: FileChanges) -> FileChanges {
    changes
        .into_iter()
        .map(|(st, p)| (st, file_manager::get_relative(repo, &p)))
        .collect()
}
