use crate::vcs_manager::{CommitLog, FileStatus, NewCommitInfo, StateInfo};
use std::path::PathBuf;

pub fn report_current_branch(branch_name: &str) -> String {
    format!("On branch {branch_name}\n")
}

pub fn report_creating_new_branch(info: &StateInfo) -> String {
    let mut report = format!("Created a new branch {}", info.branch);
    if let Some(commit) = &info.commit {
        report += &format!(" from master's commit {commit}");
    };
    report + "\n"
}

pub fn report_successful_init(path: &str) -> String {
    format!("Initialized VCS repository in {path}\n")
}

pub fn report_successful_commit(info: &NewCommitInfo) -> String {
    format!("[{} {}] {}\n", info.branch, info.human_id, info.message)
        + &report_changes(&info.changes)
}

pub fn report_changes(changes: &[(FileStatus, PathBuf)]) -> String {
    let mut report = String::new();
    for (status, path) in changes.iter() {
        report += match status {
            FileStatus::Added => "  added: ",
            FileStatus::Modified => "  modified: ",
            FileStatus::Unchanged => unreachable!(),
        };
        report += &format!("{}\n", path.to_str().unwrap());
    }
    report
}

pub fn report_successful_jump_to_commit(info: &StateInfo) -> String {
    format!(
        "Successfully jumped to commit {}. Current branch: {}\n",
        info.commit.as_ref().unwrap(),
        info.branch
    )
}

pub fn report_successful_jump_to_branch(info: &StateInfo) -> String {
    let mut report = format!("Successfully jumped to branch {}.", info.branch);
    if let Some(commit) = &info.commit {
        report += &format!(" Current commit: {commit}.");
    }
    report + "\n"
}

pub fn display_logs(logs: &[CommitLog]) -> String {
    let mut iter = logs.iter().peekable();
    let mut report = String::new();
    while let Some(commit) = iter.next() {
        report += &format!(
            "commit {}\nDate: {}\nMessage: {}\n",
            commit.human_id,
            commit.time.format("%a %b %e %H:%M:%S %Y %z"),
            commit.message
        );
        if commit.changes.is_empty() {
            report += "No changes\n";
        } else {
            report += "Changes:\n";
            report += &report_changes(&commit.changes);
        }
        if iter.peek().is_some() {
            report += "\n";
        }
    }
    report
}
