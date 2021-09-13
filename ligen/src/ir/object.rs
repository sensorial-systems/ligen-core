//! Object representation with its structure and implementation items.

use crate::ir::{Implementation, Path, TypeDefinition};

/// Object representation.
/// This is a conventional structure created from a composition of a struct definition and impl
/// blocks.
#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    // FIXME: Path should be replaced by Identifier. The Path can be constructed by iterating on its parents.
    /// Object path.
    pub path: Path,
    /// Object type definition.
    pub definition: TypeDefinition,
    /// Object implementations blocks.
    pub implementations: Vec<Implementation>
}
