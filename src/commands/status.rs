use crate::errors::VcsResult;
use crate::report_printer::{report_changes, report_current_branch};
use crate::vcs_manager;

pub fn run() -> VcsResult<String> {
    let repo = vcs_manager::find_repository(&std::env::current_dir()?)?;
    let state = vcs_manager::get_state(&repo)?;
    let mut report = report_current_branch(&state.branch);
    let changes = vcs_manager::get_changes(&repo)?;
    if changes.is_empty() {
        report += "No changes to be committed\n";
    } else {
        report += "Changes to be committed:\n";
        report += &report_changes(&changes);
    }
    Ok(report)
}
