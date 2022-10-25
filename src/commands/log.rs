use crate::errors::VcsResult;
use crate::report_printer::display_logs;
use crate::vcs_manager;

pub fn run() -> VcsResult<String> {
    let repo = vcs_manager::find_repository(&std::env::current_dir()?)?;
    let logs = vcs_manager::get_commit_logs(&repo)?;
    Ok(display_logs(&logs))
}
