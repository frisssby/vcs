use crate::errors::VcsResult;
use crate::report_printer::report_successful_commit;
use crate::vcs_manager;

pub fn run(message: &str) -> VcsResult<String> {
    let repo = vcs_manager::find_repository(&std::env::current_dir()?)?;
    let info = vcs_manager::make_commit(&repo, message)?;
    Ok(report_successful_commit(&info))
}
