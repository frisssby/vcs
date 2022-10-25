use crate::report_printer::report_changes;
use crate::vcs_manager::FileChanges;

use thiserror::Error;

pub type VcsResult<T> = Result<T, anyhow::Error>;

#[derive(Debug, Error)]
pub enum VcsError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(serde_json::Error),
    #[error("Not a vcs repository")]
    NotVcsRepository,
    #[error("Already a vcs repository")]
    AlreadyVcsRepository,
    #[error("No changes to be committed")]
    NoChanges,
    #[error(
        "You can create a new commit only from last one.\n\
        Aborting..."
    )]
    CommitFromNonHead,
    #[error(
        "Creating a new branch is possible only when you are in the master branch.\n\
        Aborting..."
    )]
    BranchOffNonMaster,
    #[error(
        "Branch {0} already exists.\n\
        Aborting..."
    )]
    BranchAlreadyExists(String),
    #[error(
        "Your local changes to the following files should be commited or dropped:\n\
            {}\
        Please commit your changes or drop them before you jump.\n\
        Aborting...",
        report_changes(changes)
    )]
    UncomittedChanges { changes: FileChanges },
    #[error(
        "No commit with hash {0} exists.\n\
            Aborting..."
    )]
    NoCommit(String),
    #[error(
        "No branch <branch_name> exists.\n\
            Aborting..."
    )]
    NoBranch(String),
    #[error(
        "Merge conflict: file(s) has been changed both in master and branch\n\
        {}\
        Aborting...",
        report_changes(both_changed)
    )]
    MergeConflict { both_changed: FileChanges },
    #[error(
        "The merge is possible only when you are in the last commit in master.\n\
        Aborting..."
    )]
    MergeFromNotMasterHead,
}

impl From<serde_json::Error> for VcsError {
    fn from(err: serde_json::Error) -> VcsError {
        use serde_json::error::Category;
        match err.classify() {
            Category::Io => VcsError::Io(err.into()),
            _ => VcsError::Json(err),
        }
    }
}
