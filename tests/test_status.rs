mod common;
use common::*;

use assert_fs::prelude::{FileWriteStr, PathChild, PathCreateDir};
use predicates::prelude::*;

#[test]
fn test_show_branch() {
    let repo = create_test_repo(&[], &[]);
    init_repo(repo.path());
    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("On branch master"));
    repo.close().unwrap();
}

#[test]
fn test_no_changes() {
    let repo = create_test_repo(&[], &[]);
    init_repo(repo.path());
    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("status");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No changes"));
    repo.close().unwrap();
}

#[test]
fn test_changes() {
    let repo = create_test_repo(&["file1"], &[]);
    init_repo(repo.path());

    repo.child("file1").write_str("hello world").unwrap();
    repo.child("empty_dir").create_dir_all().unwrap();

    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("status");
    cmd.assert().success().stdout(
        predicate::str::contains("Changes to be committed")
            .and(predicate::str::contains("modified: file1"))
            .and(predicate::str::contains("empty_dir").not()),
    );
    repo.close().unwrap();
}
