use std::path::Path;

use crate::errors::VcsResult;
use crate::report_printer::{report_successful_commit, report_successful_init};
use crate::vcs_manager;

pub fn run(path: &Path) -> VcsResult<String> {
    let path = path.to_path_buf().canonicalize()?;
    let mut report = match vcs_manager::init_vcs_directory(&path) {
        Ok(_) => report_successful_init(path.to_str().unwrap()),
        Err(err) => return Err(err),
    };
    let info = vcs_manager::make_commit(&path, "Initial commit")?;
    report += "Created commit:\n";
    report += &report_successful_commit(&info);
    Ok(report)
}
