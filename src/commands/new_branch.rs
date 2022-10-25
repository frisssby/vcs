use crate::errors::VcsResult;
use crate::vcs_manager;

use crate::report_printer::report_creating_new_branch;

pub fn run(name: &str) -> VcsResult<String> {
    let repo = vcs_manager::find_repository(&std::env::current_dir()?)?;
    let info = vcs_manager::create_branch(&repo, name)?;
    Ok(report_creating_new_branch(&info))
}
