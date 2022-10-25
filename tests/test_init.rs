mod common;
use self::common::*;

use predicates::prelude::*;

#[test]
fn test_empty() {
    let repo = create_test_repo(&[], &[]);

    let mut cmd = get_cmd();
    cmd.arg("init").arg("--path").arg(repo.path());

    cmd.assert().success().stdout(
        predicate::str::contains(format!(
            "Initialized VCS repository in {}",
            repo.path().display()
        ))
        .and(predicate::str::contains("Created commit:"))
        .and(predicate::str::contains("Initial commit")),
    );
    repo.close().unwrap();
}

#[test]
fn test_no_path() {
    let mut cmd = get_cmd();
    cmd.arg("init");
    cmd.assert().failure();
    cmd.arg("--path");
    cmd.assert().failure();
}

#[test]
fn test_changes_add() {
    let repo = create_test_repo(&["file1", "subdir/file2"], &["empty_dir"]);

    let mut cmd = get_cmd();
    cmd.arg("init").arg("--path").arg(repo.path());

    cmd.assert().success().stdout(
        predicate::str::contains("added: file1")
            .and(predicate::str::contains("added: subdir/file2")),
    );
    repo.close().unwrap();
}

#[test]
fn test_double_init() {
    let repo = create_test_repo(&[], &[]);

    let mut cmd = get_cmd();
    cmd.arg("init").arg("--path").arg(repo.path());
    cmd.assert().success();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Already a vcs repository"));
}
