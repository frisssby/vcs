use super::objects_manager::VCS_ROOT;
use crate::errors::VcsResult;

use anyhow::Context;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub fn read_file(path: &Path) -> VcsResult<Vec<u8>> {
    fs::read(path).with_context(|| format!("Failed to read {}", path.display()))
}

pub fn write_file(path: &Path, data: &Vec<u8>) -> VcsResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {} directory", parent.display()))?;
    }
    fs::write(path, data).with_context(|| format!("Failed to write to {}", path.display()))
}

pub fn is_empty_dir(directory: &Path) -> VcsResult<bool> {
    Ok(directory
        .read_dir()
        .with_context(|| format!("Failed to read {}", directory.display()))?
        .next()
        .is_none())
}

fn is_vcs_directory(entry: &DirEntry) -> bool {
    entry.path().is_dir()
        && entry
            .file_name()
            .to_str()
            .map(|s| s.eq(VCS_ROOT))
            .unwrap_or(false)
}

/// Returns a list of all files in the provided diretory ignoring .vcs folder
pub fn get_all_files(directory: &Path, recursively: bool) -> Vec<PathBuf> {
    let mut builder = WalkDir::new(directory).min_depth(1);
    if !recursively {
        builder = builder.max_depth(1);
    }
    builder
        .into_iter()
        .filter_entry(|e| !is_vcs_directory(e))
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir())
        .map(|f| f.into_path())
        .collect()
}

/// Returns a list of all entries in the provided diretory ignoring .vcs folder
pub fn get_entries(path: &Path, recursively: bool) -> Vec<PathBuf> {
    let mut builder = WalkDir::new(path).min_depth(1);
    if !recursively {
        builder = builder.max_depth(1);
    }
    builder
        .into_iter()
        .filter_entry(|e| !is_vcs_directory(e))
        .filter_map(|e| e.ok())
        .map(|f| f.into_path())
        .collect()
}

pub fn get_relative(parent: &Path, child: &Path) -> PathBuf {
    assert!(parent.is_absolute() && child.is_absolute());
    assert!(child.starts_with(parent));
    child.strip_prefix(parent).unwrap().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use std::collections::HashSet;

    #[test]
    fn test_relative_path() {
        let parent = Path::new("/home/username/path/to/repo");
        let child = Path::new("/home/username/path/to/repo/path/to/file");
        assert_eq!(get_relative(parent, child), PathBuf::from("path/to/file"));
    }

    fn assert_paths_eq(result: &[PathBuf], expected: &[PathBuf]) {
        let res_set: HashSet<&Path> = HashSet::from_iter(result.iter().map(|p| p.as_path()));
        let exp_set: HashSet<&Path> = HashSet::from_iter(expected.iter().map(|p| p.as_path()));
        assert_eq!(res_set, exp_set);
    }

    #[test]
    fn test_get_entries() {
        let tmp_dir = assert_fs::TempDir::new().unwrap();
        tmp_dir.child(".vcs").create_dir_all().unwrap();
        tmp_dir.child("file").touch().unwrap();
        tmp_dir.child("subdir").create_dir_all().unwrap();
        tmp_dir.child("subdir/file").touch().unwrap();
        tmp_dir.child("subdir/.vcs").touch().unwrap();

        let entries = get_entries(tmp_dir.path(), true);
        assert_paths_eq(
            &entries,
            &[
                tmp_dir.join("file"),
                tmp_dir.join("subdir"),
                tmp_dir.join("subdir/.vcs"),
                tmp_dir.join("subdir/file"),
            ],
        );

        let entries = get_entries(tmp_dir.path(), false);
        assert_paths_eq(&entries, &[tmp_dir.join("file"), tmp_dir.join("subdir")]);
    }

    #[test]
    fn test_get_files() {
        let tmp_dir = assert_fs::TempDir::new().unwrap();
        tmp_dir.child(".vcs").touch().unwrap();
        tmp_dir.child("file").touch().unwrap();
        tmp_dir.child("subdir").create_dir_all().unwrap();
        tmp_dir.child("subdir/file").touch().unwrap();
        tmp_dir.child("subdir/.vcs").create_dir_all().unwrap();

        let entries = get_all_files(tmp_dir.path(), true);
        assert_paths_eq(
            &entries,
            &[
                tmp_dir.join(".vcs"),
                tmp_dir.join("file"),
                tmp_dir.join("subdir/file"),
            ],
        );

        let entries = get_all_files(tmp_dir.path(), false);
        assert_paths_eq(&entries, &[tmp_dir.join(".vcs"), tmp_dir.join("file")]);
    }
}
