mod common;
use common::*;

use predicates::prelude::*;

#[test]
fn test_successful_branching() {
    let repo = create_test_repo(&[], &[]);
    init_repo(repo.path());
    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("new_branch").arg("--name").arg("new_feature");
    cmd.assert().success();

    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("status");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("On branch new_feature"));
    repo.close().unwrap();
}

#[test]
fn test_from_side_branch() {
    let repo = create_test_repo(&[], &[]);
    init_repo(repo.path());
    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("new_branch").arg("--name").arg("new_feature");
    cmd.assert().success();

    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("new_branch").arg("--name").arg("one_more_feature");
    cmd.assert().failure().stderr(predicate::str::contains(
        "possible only when you are in the master branch",
    ));
    repo.close().unwrap();
}

#[test]
fn branch_already_exists() {
    let repo = create_test_repo(&[], &[]);
    init_repo(repo.path());
    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("new_branch").arg("--name").arg("new_feature");
    cmd.assert().success();

    jump_to_branch(repo.path(), "master");

    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("new_branch").arg("--name").arg("new_feature");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
    repo.close().unwrap();
}
