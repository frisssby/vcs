use crate::errors::VcsResult;
use crate::report_printer::report_successful_commit;
use crate::vcs_manager;

pub fn run(branch: &str) -> VcsResult<String> {
    let repo = vcs_manager::find_repository(&std::env::current_dir()?)?;
    let info = vcs_manager::merge_branch(&repo, branch)?;
    let report =
        "Successfully created merge commit:\n".to_string() + &report_successful_commit(&info);
    Ok(report)
}
