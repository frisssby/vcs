mod common;
use common::*;

use assert_fs::prelude::{FileTouch, FileWriteStr, PathChild};
use predicates::prelude::*;

#[test]
fn test_simple() {
    let repo = create_test_repo(&[], &[]);
    init_repo(repo.path());

    repo.child("new_file").touch().unwrap();
    make_commit(repo.path(), "add new_file");

    repo.child("new_file").write_str("hello").unwrap();
    make_commit(repo.path(), "add hello to new_file");

    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("log");
    cmd.assert().success().stdout(
        predicate::str::is_match(format!(r"commit {}", COMMIT_ID_PATTERN))
            .unwrap()
            .count(3),
    );
    repo.close().unwrap();
}

#[test]
fn test_branch_logs_root() {
    let repo = create_test_repo(&[], &[]);
    init_repo(repo.path());

    repo.child("new_file").touch().unwrap();
    make_commit(repo.path(), "add new_file");

    create_branch(repo.path(), "feature branch");

    repo.child("new_file").write_str("hello").unwrap();
    make_commit(repo.path(), "add hello to new_file");

    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("log");
    cmd.assert().success().stdout(
        predicate::str::is_match(format!(r"commit {}", COMMIT_ID_PATTERN))
            .unwrap()
            .count(3),
    );
    repo.close().unwrap();
}
