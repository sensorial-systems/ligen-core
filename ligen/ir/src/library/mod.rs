//! Library representation.

pub mod metadata;
use is_tree::IntoIterTypeMut;
use is_tree::IterTypeMut;
use is_tree::TypeIteratorMut;
pub use metadata::*;

use crate::Identifier;
use crate::Module;
use crate::Type;
use crate::prelude::*;

/// Library representation.
#[allow(missing_docs)]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Library {
    pub identifier: Identifier,
    pub metadata: Metadata,
    pub root_module: Module,
}

impl Library {
    /// Save library to file.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Load library from file.
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }
}

impl IntoIterTypeMut<Type> for Library {
    fn into_type_iterator<'a>(&'a mut self) -> TypeIteratorMut<'a, Type> {
        self.root_module.iter_type_mut::<Type>()
    }
}