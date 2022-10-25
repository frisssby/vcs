use assert_cmd::Command;
use assert_fs::{prelude::*, TempDir};
use std::env;
use std::path::Path;

pub const COMMIT_ID_PATTERN: &str = r"[[a-f][0-9]]{40}";

pub fn get_cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

pub fn get_repo_cmd(repo: &Path) -> Command {
    let mut cmd = get_cmd();
    cmd.current_dir(repo);
    cmd
}

pub fn create_test_repo(files: &[&str], dirs: &[&str]) -> TempDir {
    let repo = assert_fs::TempDir::new().unwrap();
    for file in files {
        repo.child(file).touch().unwrap();
    }
    for dir in dirs {
        repo.child(dir).create_dir_all().unwrap();
    }
    repo
}

pub fn init_repo(path: &Path) -> String {
    let mut cmd = get_cmd();
    let output = cmd
        .arg("init")
        .arg("--path")
        .arg(path)
        .ok()
        .expect("It seems that init cmd doesn't work");
    get_commit_id(&output.stdout)
}

pub fn make_commit(repo: &Path, message: &str) -> String {
    let mut cmd = get_repo_cmd(repo);
    let output = cmd
        .arg("commit")
        .arg("--message")
        .arg(message)
        .ok()
        .expect("It seems that commit cmd doesn't work");
    get_commit_id(&output.stdout)
}

pub fn jump_to_branch(repo: &Path, name: &str) {
    let mut cmd = get_repo_cmd(repo);
    cmd.arg("jump")
        .arg("--branch")
        .arg(name)
        .ok()
        .expect("It seems that jump cmd doesn't work");
}

fn get_commit_id(output: &Vec<u8>) -> String {
    let pattern = regex::Regex::new(COMMIT_ID_PATTERN).unwrap();
    pattern
        .find(std::str::from_utf8(&output).unwrap())
        .unwrap()
        .as_str()
        .to_owned()
}

pub fn create_branch(repo: &Path, name: &str) {
    let mut cmd = get_repo_cmd(repo);
    cmd.arg("new_branch")
        .arg("--name")
        .arg(name)
        .ok()
        .expect("It seems that new_branch cmd doesn't work");
}
