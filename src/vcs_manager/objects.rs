use super::traits::VcsSerialize;

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Vcs object unique id.
pub type VcsObjectId = [u8; 20];

pub fn get_vcs_object_id(content: &Bytes) -> VcsObjectId {
    let mut hasher = Sha1::new();
    hasher.update(content);
    hasher.finalize().try_into().unwrap()
}

/// Inner representation of all Vcs structures and objects.
pub type Bytes = Vec<u8>;

/// Enum whose variants are all possible types of objects in Vcs.
#[derive(Debug, Serialize, Deserialize)]
pub enum VcsObjects {
    Commit(Commit),
    Blob(Blob),
    Tree(Tree),
}
impl VcsSerialize for VcsObjects {}
impl VcsObjects {
    pub fn commit(self) -> Commit {
        if let VcsObjects::Commit(commit) = self {
            commit
        } else {
            panic!("Not a Commit object")
        }
    }
    pub fn blob(self) -> Blob {
        if let VcsObjects::Blob(blob) = self {
            blob
        } else {
            panic!("Not a Blob object")
        }
    }
    pub fn tree(self) -> Tree {
        if let VcsObjects::Tree(tree) = self {
            tree
        } else {
            panic!("Not a Tree object")
        }
    }
}

/// A Commit object representing commits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub tree: VcsObjectId,
    pub parent: Option<VcsObjectId>,
    pub branch: String,
    pub time: SystemTime,
    pub message: String,
}

/// A Blob object representing files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blob {
    pub file_name: String,
    pub data: Bytes,
}

/// A Tree object representing directories.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tree(Vec<TreeNode>);

impl Tree {
    /// Creates an empty tree.
    pub fn new() -> Self {
        Self(Vec::new())
    }
    /// Adds a new node to the tree.
    pub fn add_node(&mut self, node: TreeNode) {
        self.0.push(node);
    }
    /// If the tree root contains a node with the given path and type, returns
    /// Some(&TreeNode). Otherwise, returns None.
    pub fn find(&self, path: &Path, is_blob: bool) -> Option<&TreeNode> {
        self.0
            .iter()
            .find(|node| node.path == path && node.is_blob() == is_blob)
    }
    /// If the tree root contains a node with the given path and type, returns
    /// Some(&mut TreeNode). Otherwise, returns None.
    pub fn find_mut(&mut self, path: &Path, is_blob: bool) -> Option<&mut TreeNode> {
        self.0
            .iter_mut()
            .find(|node| node.path == path && node.is_blob() == is_blob)
    }
    /// Gets an iterator that visits the tree's direct children.
    pub fn iter(&self) -> std::slice::Iter<'_, TreeNode> {
        self.0.iter()
    }
    /// Creates and iterator from the tree's direct children values.
    pub fn into_iter(self) -> std::vec::IntoIter<TreeNode> {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreeNode {
    object_type: TreeNodeType,
    pub id: VcsObjectId,
    pub path: PathBuf,
}
impl TreeNode {
    pub fn new(id: VcsObjectId, path: PathBuf, is_blob: bool) -> Self {
        Self {
            object_type: if is_blob {
                TreeNodeType::Blob
            } else {
                TreeNodeType::Tree
            },
            id,
            path,
        }
    }
    /// Returns true if the node corresponds to a Blob object.
    pub fn is_blob(&self) -> bool {
        matches!(self.object_type, TreeNodeType::Blob)
    }
    /// Returns true if the node corresponds to a Tree object.
    pub fn is_tree(&self) -> bool {
        !self.is_blob()
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum TreeNodeType {
    Tree,
    Blob,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VcsRepositoryState {
    pub current_commit: Option<VcsObjectId>,
    pub current_branch: String,
}
impl VcsSerialize for VcsRepositoryState {}

/// Stores current state of files (their Blob ids) in the working directory.
#[derive(Debug, Serialize, Deserialize)]
pub struct Index(BTreeMap<PathBuf, VcsObjectId>);
impl VcsSerialize for Index {}
impl Index {
    /// Creates an empty index.
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
    /// Get current object's id corresponding to the path provided.
    pub fn get_id(&self, path: &Path) -> &VcsObjectId {
        self.0.get(path).unwrap()
    }
    pub fn contains(&self, path: &Path) -> bool {
        self.0.contains_key(path)
    }
    pub fn update(&mut self, path: PathBuf, id: VcsObjectId) {
        self.0.insert(path, id);
    }
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

/// Stores vcs objects references
#[derive(Debug, Serialize, Deserialize)]
pub struct RefStorage(BTreeMap<String, VcsObjectId>);
impl VcsSerialize for RefStorage {}
impl RefStorage {
    /// Creates an empty storage.
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
    /// Get object's id by the name of the reference.
    pub fn get_id(&self, name: &str) -> &VcsObjectId {
        self.0.get(name).unwrap()
    }
    /// Returns true if the storage contains a reference with the name <name>.
    pub fn contains(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }
    /// Adds a new reference if it is not in the storage yet. Otherwise, updates
    /// object's id that the reference points to.
    pub fn update(&mut self, name: String, id: VcsObjectId) {
        self.0.insert(name, id);
    }
}
