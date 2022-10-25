use crate::errors::VcsResult;
use crate::report_printer::{report_successful_jump_to_branch, report_successful_jump_to_commit};
use crate::vcs_manager;

pub fn to_commit(id: &str) -> VcsResult<String> {
    let repo = vcs_manager::find_repository(&std::env::current_dir()?)?;
    let state = vcs_manager::jump_to_commit(&repo, id)?;
    Ok(report_successful_jump_to_commit(&state))
}

pub fn to_branch(branch_name: &str) -> VcsResult<String> {
    let repo = vcs_manager::find_repository(&std::env::current_dir()?)?;
    let state = vcs_manager::jump_to_branch(&repo, branch_name)?;
    Ok(report_successful_jump_to_branch(&state))
}
