mod common;
use common::*;

use assert_fs::prelude::{FileTouch, FileWriteStr, PathAssert, PathChild};
use predicates::prelude::*;

#[test]
fn test_simple() {
    let repo = create_test_repo(&[], &[]);
    init_repo(repo.path());

    repo.child("new_file").touch().unwrap();
    make_commit(repo.path(), "add new_file");

    create_branch(repo.path(), "feature branch");

    repo.child("new_file").write_str("hello").unwrap();
    make_commit(repo.path(), "add hello to new_file");

    jump_to_branch(repo.path(), "master");

    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("merge").arg("--branch").arg("feature branch");
    cmd.assert().success().stdout(predicate::str::contains(
        "Successfully created merge commit",
    ));

    repo.child("new_file").assert("hello");
    repo.close().unwrap();
}

#[test]
fn test_merge_conflict() {
    let repo = create_test_repo(&[], &[]);
    init_repo(repo.path());

    repo.child("new_file").touch().unwrap();
    make_commit(repo.path(), "add new_file");

    create_branch(repo.path(), "feature_branch");

    repo.child("new_file").write_str("hello").unwrap();
    make_commit(repo.path(), "add hello to new_file");

    jump_to_branch(repo.path(), "master");

    repo.child("new_file").write_str("goodbye").unwrap();
    make_commit(repo.path(), "add goodbye to new_file");

    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("merge").arg("--branch").arg("feature_branch");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Merge conflict"));

    repo.child("new_file").assert("goodbye");
    repo.close().unwrap();
}

#[test]
fn test_merge_from_side_branch() {
    let repo = create_test_repo(&[], &[]);
    init_repo(repo.path());

    repo.child("new_file").touch().unwrap();
    make_commit(repo.path(), "add new_file");

    create_branch(repo.path(), "feature_branch");
    jump_to_branch(repo.path(), "master");

    repo.child("new_file").write_str("hello").unwrap();
    make_commit(repo.path(), "add hello to new_file");
    jump_to_branch(repo.path(), "feature_branch");

    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("merge").arg("--branch").arg("master");
    cmd.assert().failure().stderr(predicate::str::contains(
        "merge is possible only when you are in the last commit in master",
    ));
}

#[test]
fn test_no_branch() {
    let repo = create_test_repo(&[], &[]);
    init_repo(repo.path());
    let mut cmd = get_repo_cmd(repo.path());
    cmd.arg("merge").arg("--branch").arg("non_existing_branch");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No branch"));
}
