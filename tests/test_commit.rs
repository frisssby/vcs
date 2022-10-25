mod common;
use common::*;

use assert_fs::prelude::{FileWriteStr, PathChild, PathCreateDir};
use predicates::prelude::*;

#[test]
fn test_no_message() {
    let repo = create_test_repo(&[], &[]);
    init_repo(repo.path());
    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("commit");
    cmd.assert().failure();
    cmd.arg("--message");
    cmd.assert().failure();
    repo.close().unwrap();
}

#[test]
fn test_empty_commit() {
    let repo = create_test_repo(&[], &[]);
    init_repo(repo.path());
    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("commit").arg("--message").arg("empty commit");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No changes to be committed"));
    repo.child("empty_dir").create_dir_all().unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No changes to be committed"));
    repo.close().unwrap();
}

#[test]
fn test_successful_commit() {
    let repo = create_test_repo(&[], &[]);
    init_repo(repo.path());

    repo.child("file1").write_str("hello world").unwrap();
    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("commit").arg("--message").arg("add file1");
    cmd.assert().success().stdout(
        predicate::str::is_match(format!(
            r"^\[master {}\] add file1\n  added: file1\n$",
            COMMIT_ID_PATTERN
        ))
        .unwrap(),
    );
    repo.close().unwrap();
}
