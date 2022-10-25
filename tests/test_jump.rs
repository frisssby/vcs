mod common;
use common::*;

use assert_fs::prelude::{FileTouch, PathChild};
use predicates::prelude::*;

#[test]
fn test_jump_to_commit() {
    let repo = create_test_repo(&[], &[]);
    let initial_commit_id = init_repo(repo.path());
    repo.child("new_file").touch().unwrap();
    make_commit(repo.path(), "add new_file");

    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("jump").arg("--commit").arg(&initial_commit_id);
    cmd.assert().success();
    assert!(!repo.child("new_file").exists());
    repo.close().unwrap();
}

#[test]
fn test_jump_to_branch() {
    let repo = create_test_repo(&[], &[]);
    init_repo(repo.path());
    create_branch(repo.path(), "new_branch");

    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("status");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("On branch new_branch"));

    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("jump").arg("--branch").arg("master");
    cmd.assert().success();

    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("status");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("On branch master"));
    repo.close().unwrap();
}

#[test]
fn test_no_branch() {
    let repo = create_test_repo(&[], &[]);
    init_repo(repo.path());
    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("jump").arg("--branch").arg("not_existing_branch");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No branch"));
    repo.close().unwrap();
}

#[test]
fn test_no_commit() {
    let repo = create_test_repo(&[], &[]);
    init_repo(repo.path());
    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("jump").arg("--commit").arg("random_id");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No commit"));
    repo.close().unwrap();
}

#[test]
fn test_uncommitted_changes() {
    let repo = create_test_repo(&[], &[]);
    let initial_commit_id = init_repo(repo.path());
    repo.child("new_file").touch().unwrap();

    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("jump").arg("--commit").arg(&initial_commit_id);
    cmd.assert().failure().stderr(predicate::str::contains(
        "local changes to the following files should be commited or dropped",
    ));
}
