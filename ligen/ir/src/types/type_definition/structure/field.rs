//! Structure field representation.

use crate::prelude::*;
use crate::{Identifier, Type, Visibility, Attributes};

/// Property representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    /// Field attributes.
    pub attributes: Attributes,
    /// Field visibility.
    pub visibility: Visibility,
    /// Field identifier.
    pub identifier: Option<Identifier>,
    /// Field type.
    pub type_: Type
}
