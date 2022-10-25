use anyhow::Context;

use super::file_manager::{read_file, write_file};
use super::objects::Bytes;
use crate::errors::VcsResult;
use std::path::Path;

/// A trait for serializing and deserializing data structure's into json format,
/// and also writing them to (and reading from) files.
pub trait VcsSerialize: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug {
    /// Serializes the object and returns its data in the form of bytes vector
    fn get_content(&self) -> VcsResult<Bytes> {
        serde_json::ser::to_vec(self).with_context(|| format!("Failed to serialize {:?}", self))
    }
    /// Constructs an object from bytes vector in json format
    fn read_from(content: &Bytes) -> VcsResult<Self> {
        serde_json::de::from_slice(content).with_context(|| {
            format!(
                "Failed to deserialize data into {:?}",
                std::any::type_name::<Self>()
            )
        })
    }
    /// Writes the object to the file in the path provided serializing it
    /// beforehand
    fn save(&self, path: &Path) -> VcsResult<()> {
        write_file(
            path,
            &self
                .get_content()
                .with_context(|| format!("Failed to save {:?} to {}", self, path.display()))?,
        )
    }
    /// Loads an object from the file in the path provided by deserializing
    /// file's raw bytes
    fn load(path: &Path) -> VcsResult<Self> {
        Self::read_from(&read_file(path).with_context(|| {
            format!(
                "Failed to load {:?} from {}",
                std::any::type_name::<Self>(),
                path.display()
            )
        })?)
    }
}
